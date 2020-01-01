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
        Ok(s) => println!("Read {} bytes from body", s),
        Err(_) => return rouille::Response::text("Failed to read request body")
                                           .with_status_code(500)
    };

    // Ensure correct content type
    let content_type = match request.header("Content-Type") {
        Some(t) => t,
        None => return rouille::Response::text("No content type set!")
                                         .with_status_code(500)
    };

    // Parse mail based on content-type
    let mail = match mail::Mail::from_body(&body, &content_type) {
        Ok(m) => m,
        Err(e) => return rouille::Response::text(e.to_string()).with_status_code(500)
    };

    println!("{:?}", mail);

    rouille::Response::text("Success")
}
