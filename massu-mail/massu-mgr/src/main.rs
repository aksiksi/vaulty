mod controllers;
mod email;
mod router;

fn main() {
    rouille::start_server("0.0.0.0:7777", move |request| {
        router::handle_request(&request)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_response() {
    }
}
