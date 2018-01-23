extern crate chrono;
extern crate chrono_tz;
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate mattermost_structs;
extern crate openssl_probe;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde_yaml;
extern crate url;
extern crate ws;

mod websocket_client;

use chrono_tz::Europe::Berlin as TzBerlin;
use clap::{App, Arg};
use error_chain::ChainedError;
use mattermost_structs::*;
use mattermost_structs::api::*;
use mattermost_structs::websocket::*;
use std::fs::File;
use std::path::Path;
use std::thread;
use std::time::Duration;
use url::Url;
use websocket_client::WsClient;
use ws::connect;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Config {
    signal_phone_number: String,
    servers: Vec<ServerConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ServerConfig {
    // #[serde(with = "url_serde")]
    // base_url: Url,
    base_url: String,
    token: String,
    servername: String,
}

quick_main!(run);

fn run() -> Result<()> {
    // Setup logging
    env_logger::init();
    // this fixes connection problems with openssl
    // it set some environment variables to the correct value for the current system
    openssl_probe::init_ssl_cert_env_vars();

    let matches = App::new("Mattermost to Signal Bridge")
                        .version("0.1")
                        // .about("Does awesome things")
                        .arg(Arg::with_name("config")
                            .short("c")
                            .long("config")
                            .value_name("FILE")
                            .required(true)
                            .validator_os(|value| {
                                let path  = Path::new(value);
                                if !path.exists() {
                                    return Err("Config file does not exist".into())
                                }
                                if !path.is_file() {
                                    return Err("Config file must be a file".into())
                                }
                                Ok(())
                            })
                            .help("Sets a custom config file")
                            .takes_value(true))
                        .get_matches();

    let config: Config = serde_yaml::from_reader(File::open(
        matches
            .value_of_os("config")
            .expect("config file will exist"),
    )?)?;

    // spawn a thread for each server
    let mut thread_handles = Vec::new();
    // Check connectivity and validity of credentials
    for server_config in config.servers {
        println!("Check connectivity for: {}", server_config.servername);
        let client = Client::new(server_config.base_url.clone(), server_config.token.clone());
        if let Ok(client) = client {
            // check internet connectivity
            if client.is_token_valid() {
                thread_handles.push(spawn_server_handle_thread(
                    server_config.clone(),
                    config.signal_phone_number.clone(),
                ));
                thread_handles.push(spawn_server_watchdog(
                    server_config,
                    config.signal_phone_number.clone(),
                ));
            }
        } else {
            eprintln!("Could not connect to server '{}'", server_config.servername);
        }
    }

    for handle in thread_handles {
        handle.join().unwrap()?;
    }

    Ok(())
}

fn spawn_server_handle_thread(
    server_config: ServerConfig,
    mobile_number: String,
) -> thread::JoinHandle<Result<()>> {
    thread::spawn(move || {
        let mut url = Url::parse(&server_config.base_url)?;
        url.set_scheme("wss")
            .expect("Setting the scheme to wss must always work");
        let url = url.join("/api/v4/websocket")?;

        // Connect to the url and call the closure
        if let Err(error) = connect(url.as_str(), move |out| {
            // Queue a message to be sent when the WebSocket is open
            if let Err(_) = out.send(format!(
                r#"
                {{
                    "seq": 1,
                    "action": "authentication_challenge",
                    "data": {{
                        "token": "{}"
                    }}
                }}
            "#,
                server_config.token
            )) {
                eprintln!("Websocket couldn't queue an initial message.")
            }

            WsClient {
                ws: out,
                timeout: None,
                own_id: None,
                token: server_config.token.clone(),
                servername: server_config.servername.clone(),
                mobile_number: mobile_number.clone(),
            }
        }) {
            // Inform the user of failure
            eprintln!("Failed to create WebSocket due to: {:?}", error);
        }
        Ok(())
    })
}

fn spawn_server_watchdog(
    server_config: ServerConfig,
    mobile_number: String,
) -> thread::JoinHandle<Result<()>> {
    thread::spawn(move || {
        let client = Client::new(server_config.base_url, server_config.token)?;
        loop {
            if !client.is_token_valid() {
                let msg = format!(
                    "Token for {server} expired!",
                    server = server_config.servername,
                );
                if let Err(e) = send_android_notification(&mobile_number, &msg) {
                    warn!("{}", e.display_chain().to_string());
                }
            }
            thread::sleep(Duration::new(60 * 60 * 6, 0)); // 6 hours
        }
    })
}

fn react_to_message(client: &mut WsClient, message: &str) {
    if let Ok(msg) = serde_json::from_str::<Message>(message) {
        // ignore broadcast events which cover us
        if let Some(ref own_id) = client.own_id {
            if let Some(ref omit_users) = msg.broadcast.omit_users {
                if let Some(omit_me) = omit_users.get(own_id) {
                    if *omit_me {
                        return;
                    }
                }
            }
        }

        use Events::*;
        match msg.event {
            Hello { .. } => {
                client.own_id = Some(msg.broadcast.user_id);
            }

            Posted {
                channel_display_name,
                sender_name,
                post,
                channel_type,
                mentions,
                ..
            } => {
                eprintln!("{:?}", post);

                // only send push notification when we are mentioned
                if let Some(mentions) = mentions {
                    if mentions.contains(client.own_id.as_ref().unwrap()) {
                        // if true {
                        //     if true {
                        use std::thread;
                        let localtime = post.create_at.with_timezone(&TzBerlin).format("%H:%M:%S");
                        let testmessage = match channel_type {
                            ChannelType::DirectMessage => format!(
                                "{server} {sender}:\n{message}\n@{time}",
                                message = post.message,
                                sender = sender_name,
                                server = client.servername,
                                time = localtime,
                            ),
                            ChannelType::Open | ChannelType::Private => format!(
                                "{server} {sender} in {channel}:\n{message}\n@{time}",
                                message = post.message,
                                sender = sender_name,
                                server = client.servername,
                                channel = channel_display_name,
                                time = localtime,
                            ),
                        };
                        let mobile_number = client.mobile_number.clone();
                        thread::spawn(move || {
                            send_android_notification(&mobile_number, &testmessage)
                        });
                    }
                }
            }

            // PostEdited {
            //     post,
            // } => {
            //     eprintln!("{:?}", post);

            //     // don't send push notifications of myself
            //     if Some(&post.user_id) != client.own_id.as_ref() {
            //         return;
            //     }

            //     use std::thread;
            //     let localtime = post.edit_at.with_timezone(&TzBerlin).format("%H:%M:%S");
            //     let sender_name = client.userid_to_username
            //         .entry(post.user_id.clone())
            //         .or_insert_with({
            //             let token = client.token.clone();
            //             let id = post.user_id.clone();
            //             move || fetch_username(token, id)
            //         });
            //     let channel_name = client.channelid_to_channelname
            //         .entry(post.channel_id.clone())
            //         .or_insert_with({
            //             let token = client.token.clone();
            //             let id = post.channel_id.clone();
            //             move || fetch_channelname(token, id)
            //         });
            //     let testmessage = format!("",
            //         message=post.message,
            //         sender=*sender_name,
            //         server="CISPA",
            //         channel=*channel_name,
            //         time=localtime,
            //     );
            //     thread::spawn(move || send_android_notification(&testmessage));
            // }

            // PostDeleted {
            //     post,
            // } => {
            //     eprintln!("{:?}", post);

            //     // don't send push notifications of myself
            //     if Some(&post.user_id) != client.own_id.as_ref() {
            //         return;
            //     }

            //     use std::thread;
            //     // the `delete_at` timestamp value is unusable
            //     let localtime = Utc::now().with_timezone(&TzBerlin).format("%H:%M:%S");
            //     let sender_name = client.userid_to_username
            //         .entry(post.user_id.clone())
            //         .or_insert_with({
            //             let token = client.token.clone();
            //             let id = post.user_id.clone();
            //             move || fetch_username(token, id)
            //         });
            //     let channel_name = client.channelid_to_channelname
            //         .entry(post.channel_id.clone())
            //         .or_insert_with({
            //             let token = client.token.clone();
            //             let id = post.channel_id.clone();
            //             move || fetch_channelname(token, id)
            //         });
            //     let testmessage = format!("",
            //         message=post.message,
            //         sender=*sender_name,
            //         server="CISPA",
            //         channel=*channel_name,
            //         time=localtime,
            //     );
            //     thread::spawn(move || send_android_notification(&testmessage));
            // }

            // do nothing for other patterns
            _ => {}
        }
    } else {
        eprintln!("Could not parse the following message:");
        eprintln!("{}", message);
    }
}

fn send_android_notification(mobile_number: &str, message: &str) -> Result<()> {
    use std::process::Command;
    let mut child = Command::new("signal-cli")
        .arg("-u")
        .arg(mobile_number)
        .arg("send")
        .arg("-m")
        .arg(message)
        .arg(mobile_number)
        .spawn()?;
    child.wait()?;
    Ok(())
}
