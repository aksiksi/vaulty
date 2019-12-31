use std::io::Read;

use super::mail;

pub fn index(request: &rouille::Request) -> rouille::Response {
    rouille::Response::text("Hello, world!")
}

pub fn mailgun(request: &rouille::Request) -> rouille::Response {
    let mut data = request.data()
                          .expect("Request data already retrieved");

    let mut body = String::new();
    match data.read_to_string(&mut body) {
        Ok(s) => println!("Read {} bytes", s),
        Err(_) => return rouille::Response::text("Failed to read request body")
                                           .with_status_code(500)
    };

    let mail: mail::Mail = serde_json::from_str(&body)
                                      .expect("Failed to parse JSON");

    println!("{:?}", mail);

    rouille::Response::text("Success")
}
