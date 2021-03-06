mod websocket_client;

use crate::websocket_client::WsClient;
use chrono_tz::Europe::Berlin as TzBerlin;
use error_chain::{quick_main, ChainedError};
use log::{debug, error, warn};
use mattermost_structs::{
    api::{ChannelType, Client, CreatePostRequest},
    websocket::{Events, Message, Status},
    Result,
};
use serde::{Deserialize, Serialize};
use std::{
    ffi::{OsStr, OsString},
    fs::File,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use structopt::StructOpt;
use url::Url;
use ws::connect;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Config {
    signal_phone_number: String,
    servers: Vec<ServerConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    // #[serde(with = "url_serde")]
    // base_url: Url,
    base_url: String,
    token: String,
    servername: String,
}

/// Mattermost to Signal Bridge
#[derive(Debug, StructOpt)]
#[structopt(
    author = "",
    raw(setting = "structopt::clap::AppSettings::ColoredHelp")
)]
struct CliArgs {
    /// Sets a custom config file
    #[structopt(
        short = "c",
        long = "config",
        parse(from_os_str),
        raw(validator_os = "path_is_file")
    )]
    config: PathBuf,
}

fn path_is_file(value: &OsStr) -> std::result::Result<(), OsString> {
    let path = Path::new(value);
    if !path.exists() {
        return Err("Config file does not exist".into());
    }
    if !path.is_file() {
        return Err("Config file must be a file".into());
    }
    Ok(())
}

quick_main!(run);

fn run() -> Result<()> {
    // Setup logging
    env_logger::init();
    // this fixes connection problems with openssl
    // it set some environment variables to the correct value for the current system
    openssl_probe::init_ssl_cert_env_vars();

    // parse arguments
    let args = CliArgs::from_args();

    let config: Config = serde_yaml::from_reader(File::open(args.config)?)?;

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
            } else {
                error!("Invalid token for {}", server_config.servername);
            }
        } else {
            error!("Could not connect to server '{}'", server_config.servername);
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
    fn handle_server(
        serverconfig: ServerConfig,
        mobile_number: String,
        serverstate: Arc<Mutex<Status>>,
    ) -> thread::JoinHandle<Result<()>> {
        thread::spawn(move || {
            let mut url = Url::parse(&*serverconfig.base_url)?;
            url.set_scheme("wss")
                .expect("Setting the scheme to wss must always work");
            let url = url.join("/api/v4/websocket")?;

            // Connect to the url and call the closure
            if let Err(error) = connect(url.as_str(), move |out| {
                // Queue a message to be sent when the WebSocket is open
                if out
                    .send(format!(
                        r#"
                    {{
                        "seq": 1,
                        "action": "authentication_challenge",
                        "data": {{
                            "token": "{}"
                        }}
                    }}
                "#,
                        serverconfig.token
                    ))
                    .is_err()
                {
                    error!("Websocket couldn't queue an initial message.")
                }

                WsClient {
                    ws: out,
                    timeout: None,
                    own_id: None,
                    mobile_number: mobile_number.clone(),
                    serverconfig: serverconfig.clone(),
                    serverstate: serverstate.clone(),
                }
            }) {
                // Inform the user of failure
                error!("Failed to create WebSocket due to: {:?}", error);
            }
            Ok(())
        })
    };

    let serverstate = Arc::new(Mutex::new(Status::Online));
    // the websocket client can die, e.g., if the Internet connection fails or
    // mattermost fails for some time
    // Therefore, make sure to restart the handle if it fails
    thread::spawn(move || loop {
        let serverstate = serverstate.clone();
        let serverconfig = server_config.clone();
        let mobile_number = mobile_number.clone();

        match handle_server(serverconfig, mobile_number, serverstate).join() {
            Ok(Err(err)) => warn!(
                "Websocket connection to \"{}\" failed:\n{}",
                server_config.servername, err
            ),
            Err(_) => warn!("Thread for \"{}\" paniced!", server_config.servername),
            _ => {}
        }
        thread::sleep(Duration::from_secs(5));
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
    if let Ok(Message::Push(msg)) = serde_json::from_str::<Message>(message) {
        debug!("Received message:\n{:?}", msg);

        use crate::Events::*;
        match msg.event {
            Hello { .. } => {
                client.own_id = Some(msg.broadcast.user_id);
            }

            // Track the servers/users status to not send any notifications while in Do Not Disturb mode
            StatusChange { status, .. } => {
                let mut serverstate = client.serverstate.lock().unwrap();
                *serverstate = status;
            }

            Posted {
                channel_display_name,
                sender_name,
                post,
                channel_type,
                mentions,
                ..
            } => {
                // React to some messages
                if client.own_id == Some(post.user_id) && post.message.starts_with("@me") {
                    let client = Client::new(
                        client.serverconfig.base_url.clone(),
                        client.serverconfig.token.clone(),
                    );
                    if let Ok(client) = client {
                        // if the message we receive has a root_id, then we are already in a thread, so further use that
                        // otherwise use the post id
                        let root_id = if !post.root_id.is_empty() {
                            post.root_id.clone()
                        } else {
                            post.id.clone()
                        };

                        let _ = client.create_post(&CreatePostRequest {
                            channel_id: post.channel_id.clone(),
                            message: "Hi!".to_string(),
                            root_id: Some(root_id),
                            ..CreatePostRequest::default()
                        });
                    }
                }

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

                // only send push notification when we are mentioned
                // Also check that the status is anything but do not disturb
                if let Some(mentions) = mentions {
                    let status = client.serverstate.lock().unwrap();
                    if *status != Status::DoNotDisturb
                        && mentions.contains(client.own_id.as_ref().unwrap())
                    {
                        let localtime = post.create_at.with_timezone(&TzBerlin).format("%H:%M:%S");
                        let testmessage = match channel_type {
                            ChannelType::DirectMessage | ChannelType::Group => format!(
                                "{server} {sender}:\n{message}\n@{time}",
                                message = post.message,
                                sender = sender_name,
                                server = client.serverconfig.servername,
                                time = localtime,
                            ),
                            ChannelType::Open | ChannelType::Private => format!(
                                "{server} {sender} in {channel}:\n{message}\n@{time}",
                                message = post.message,
                                sender = sender_name,
                                server = client.serverconfig.servername,
                                channel = channel_display_name,
                                time = localtime,
                            ),
                            ChannelType::Internal => {
                                // Ignore this type.
                                // I don't know what exactly this type even is
                                return;
                            }
                        };
                        let mobile_number = client.mobile_number.clone();
                        thread::spawn(move || {
                            send_android_notification(&mobile_number, &testmessage)
                        });
                    }
                }
            }

            // do nothing for other patterns
            _ => {}
        }
    } else {
        warn!("Could not parse the following message:");
        warn!("{}", message);
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
