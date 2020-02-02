use std::io::Read;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "vaulty-filter", about = "Vaulty filter for Postfix incoming mail.")]
struct Opt {
    #[structopt(short, long)]
    sender: String,

    #[structopt(short, long)]
    recipient: String,
}

// Example
async fn transmit_email(_recipient: String, _sender: String,
                        email: String) {
    let client = reqwest::Client::new();

    let req = client
        .post("http://httpbin.org/post")
        .body(reqwest::Body::from(email));

    let resp = req.send().await.expect("Failed request!");

    assert!(resp.status().is_success());

    let body = resp.text().await.unwrap();
    println!("{}", body);
}

#[tokio::main]
async fn main() {
    // Init logger
    env_logger::builder().format_timestamp_micros().init();

    let opt = Opt::from_args();

    println!("{:?}", opt);

    // Get message body from stdin
    let mut email = String::new();
    std::io::stdin().read_to_string(&mut email)
                    .expect("Failed to read email body from stdin!");

    transmit_email(opt.sender, opt.recipient, email).await;
}
