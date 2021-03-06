mod cache;
mod controllers;
mod error;
mod filters;
mod http;
mod routes;

use clap::{App, Arg};

use vaulty::config;

#[tokio::main]
async fn main() {
    // Init logger
    env_logger::builder().format_timestamp_micros().init();

    // CLI
    let matches = App::new("vaulty_server")
        .version("0.1")
        .author("Assil Ksiksi")
        .arg(
            Arg::with_name("config_path")
                .short("c")
                .long("config-path")
                .help("Path to Vaulty config")
                .value_name("CONFIG_PATH")
                .default_value(vaulty::config::DEFAULT_CONFIG_PATH)
                .takes_value(true),
        )
        .get_matches();

    // Load config
    let config_path = matches.value_of("config_path");
    let arg = config::Config::load(config_path);
    log::info!("Loaded config from {:?}", config_path);

    log::info!("Starting vaulty_server...");

    http::run(arg).await;
}
