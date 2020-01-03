use std::io::Read;

use vaulty::mailgun;

use rouille::{Request, Response};

pub fn index(_request: &Request) -> Response {
    Response::text("Hello, world!")
}

pub fn mailgun_post(request: &Request) -> Response {
    let mut data = request.data().expect("Request data already retrieved");

    log::info!(
        "Received request from: {}",
        request.remote_addr().to_string()
    );

    let mut body = String::new();
    match data.read_to_string(&mut body) {
        Ok(s) => log::info!("Read {} bytes from body", s),
        Err(_) => return Response::text("Failed to read request body").with_status_code(500),
    };

    // Ensure correct content type
    let content_type = match request.header("Content-Type") {
        Some(t) => t,
        None => return Response::text("No content type set!").with_status_code(500),
    };

    let mut mail = match mailgun::Email::from_body(&body, &content_type) {
        Ok(m) => m,
        Err(e) => {
            log::error!("{:?}", e);
            return Response::text(e.to_string()).with_status_code(500);
        }
    };

    if let Err(e) = mail.fetch_attachments() {
        return Response::text(e.to_string()).with_status_code(500);
    };

    log::info!("Fetched all attachments successfully!");

    let mut handler = vaulty::EmailHandler::new();
    let email: vaulty::email::Email = mail.into();

    log::info!("Uploaded {} attachments to Dropbox", email.attachments.len());

    if let Err(e) = handler.handle(email) {
        return Response::text(e.to_string()).with_status_code(500);
    }

    Response::text("Success")
}
