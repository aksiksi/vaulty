use std::io::Read;

use super::config;

// Example
// In reality, we should use the Vaulty lib
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

pub async fn filter(arg: &config::FilterArg<'_>) {
    // Get message body from stdin
    let mut email = String::new();
    std::io::stdin().read_to_string(&mut email)
                    .expect("Failed to read email body from stdin!");

    // Send out the email info to remote server
    // TODO: Do all processing in vaulty_filter
    transmit_email(String::from(arg.recipient),
                   String::from(arg.sender), email).await;
}
