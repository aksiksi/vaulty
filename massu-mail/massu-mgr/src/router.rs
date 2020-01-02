use rouille::router;

use super::controllers;

pub fn handle_request(request: &rouille::Request) -> rouille::Response {
    router!(request,
        (GET) (/) => {
            controllers::index(&request)
        },

        (POST) (/mailgun) => {
            controllers::mailgun_post(&request)
        },

        _ => rouille::Response::empty_404()
    )
}
