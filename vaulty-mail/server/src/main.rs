mod config;
mod filter;
mod http;
mod routes;

use clap::{Arg, App, SubCommand};

#[tokio::main]
async fn main() {
    // Init logger
    env_logger::builder().format_timestamp_micros().init();

    // CLI
    let matches = App::new("vaulty_server")
                  .version("0.1")
                  .author("Assil Ksiksi")
                  .subcommand(SubCommand::with_name("filter")
                      .arg(Arg::with_name("recipient")
                           .short("r")
                           .long("recipient")
                           .required(true)
                           .help("Receiver email address")
                           .value_name("EMAIL")
                           .takes_value(true))
                      .arg(Arg::with_name("sender")
                           .short("s")
                           .long("sender")
                           .required(true)
                           .help("Sender email address")
                           .value_name("EMAIL")
                           .takes_value(true))
                  )
                  .subcommand(SubCommand::with_name("http")
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
                           .takes_value(true))
                  )
                  .get_matches();

    // TODO: Only bring up Tokio runtime for HTTP server
    // Makes no sense to add startup time overhead for the filter, since
    // the expectation that it will called once per email by Postfix
    if let Some(matches) = matches.subcommand_matches("http") {
        let arg = config::HttpArg {
            port: matches.value_of("port").unwrap().parse::<u16>().unwrap(),
            mailgun_key: matches.value_of("mailgun_key")
        };

        http::run(&arg).await;
    } else if let Some(matches) = matches.subcommand_matches("filter") {
        let arg = config::FilterArg {
            recipient: matches.value_of("recipient").unwrap(),
            sender: matches.value_of("sender").unwrap(),
        };

        filter::filter(&arg).await;
    }
}
