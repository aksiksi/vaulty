mod controllers;
mod mailgun;
mod router;

fn main() {
    env_logger::builder().format_timestamp_micros().init();

    let _ = std::env::var("MAILGUN_API_KEY").expect("MAILGUN_API_KEY not set in env");

    log::info!("Starting server...");

    // Pool of 8 * NUM_CPU threads
    rouille::start_server_with_pool("0.0.0.0:7777", None, move |request| {
        router::handle_request(&request)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_response() {}
}
