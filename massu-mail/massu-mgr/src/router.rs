use rouille::router;

use super::controllers;

pub fn handle_request(request: &rouille::Request) -> rouille::Response {
    router!(request,
        (GET) (/) => {
            controllers::index(&request)
        },
        _ => rouille::Response::empty_404()
    )
}
