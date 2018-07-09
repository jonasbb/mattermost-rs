extern crate env_logger;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate mattermost_structs;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
#[macro_use]
extern crate structopt;
extern crate url;
extern crate ws;

use mattermost_structs::Result;
use std::{
    ffi::{OsStr, OsString},
    fs::File,
    path::{Path, PathBuf},
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
struct ServerConfig {
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

    /// Select server if multiple server are present in the configuration. Start with 1
    #[structopt(short = "s", long = "server")]
    server: Option<usize>,
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

    // Parse arguments
    let args = CliArgs::from_args();

    let config: Config = serde_yaml::from_reader(File::open(args.config)?)?;

    let server_config;
    if config.servers.is_empty() {
        bail!("You need to configure at least one server.")
    } else if config.servers.len() > 1 && args.server.is_none() {
        bail!("Multiple servers are configured, you need to select which one to use.")
    } else {
        // select first entry if len is 1
        let idx = match args.server {
            Some(0) => bail!("There is no server with id 0. Indexing starts with 1."),
            Some(n) => n - 1,
            // there is exactly one server
            None => 0,
        };

        if idx > config.servers.len() {
            bail!(
                "Server {} selected but only {} are configured.",
                idx + 1,
                config.servers.len()
            );
        }

        server_config = &config.servers[idx];
    }

    info!("Connecting to server '{}'", server_config.servername);
    let mut url = Url::parse(&server_config.base_url)?;
    url.set_scheme("wss")
        .expect("Setting the scheme to wss must always work");
    let url = url.join("/api/v4/websocket")?;

    // Connect to the url and call the closure
    if let Err(error) = connect(url.as_str(), |out| {
        // Queue a message to be sent when the WebSocket is open
        if out
            .send(&*format!(
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
            ))
            .is_err()
        {
            error!("Websocket couldn't queue an initial message.")
        }

        // The handler needs to take ownership of out, so we use move
        move |msg| {
            // Handle messages received on this connection
            println!("{}", msg);
            Ok(())
        }
    }) {
        // Inform the user of failure
        error!("Failed to create WebSocket due to: {:?}", error);
    }
    Ok(())
}
