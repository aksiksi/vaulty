use std::io::{Read};

use clap::{Arg, App};
use hyper::{Body, Client, Request};

async fn transmit_email(receiver: String, sender: String,
                        email: String) {
    // Send information to mgr server via API
    let client = Client::new();

    let req = Request::builder()
        .method("POST")
        .uri("http://httpbin.org/post")
        .body(Body::from(email))
        .expect("Failed to build request");

    let future = client.request(req);
    let resp = future.await.expect("Failed request!");

    assert!(resp.status().is_success());

    let body = hyper::body::to_bytes(resp.into_body()).await.unwrap();
    println!("{:?}", body);
}

#[tokio::main]
async fn main() {
    let matches = App::new("vaulty_filter")
                  .version("1.0")
                  .author("Assil Ksiksi")
                  .arg(Arg::with_name("receiver")
                       .short("r")
                       .long("receiver")
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
                  .get_matches();

    let receiver_address = matches.value_of("receiver").unwrap();
    let sender_address = matches.value_of("sender").unwrap();

    println!("Receiver: {}, Sender: {}", receiver_address, sender_address);

    // Get message body from stdin
    let mut email = String::new();
    std::io::stdin().read_to_string(&mut email)
                    .expect("Failed to read email body from stdin!");

    // Send out the email info to remote server
    // TODO: Do all processing in vaulty_filter
    transmit_email(String::from(receiver_address),
                   String::from(sender_address), email).await;
}
