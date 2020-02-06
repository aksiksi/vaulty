mod config;
mod controllers;
mod http;
mod routes;

use clap::{Arg, App};

#[tokio::main]
async fn main() {
    // Init logger
    env_logger::builder().format_timestamp_micros().init();

    // CLI
    let matches = App::new("vaulty_server")
                  .version("0.1")
                  .author("Assil Ksiksi")
                  .arg(Arg::with_name("port")
                       .short("p")
                       .long("port")
                       .help("HTTP server port")
                       .value_name("PORT")
                       .default_value("7777")
                       .takes_value(true))
                  .arg(Arg::with_name("mailgun_key")
                       .short("m")
                       .long("mailgun-key")
                       .help("Mailgun API key")
                       .value_name("KEY")
                       .default_value("NONE")
                       .takes_value(true))
                  .get_matches();

    // TODO: Only bring up Tokio runtime for HTTP server
    // Makes no sense to add startup time overhead for the filter, since
    // the expectation that it will called once per email by Postfix
    let arg = config::HttpArg {
        port: matches.value_of("port").unwrap().parse::<u16>().unwrap(),
        mailgun_key: matches.value_of("mailgun_key").map(|a| a.to_string()),
    };

    log::info!("Starting vaulty_server...");

    http::run(arg).await;
}
