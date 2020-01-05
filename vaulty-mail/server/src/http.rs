use warp::{self, Filter};

use super::config;
use super::routes;

pub async fn run(arg: config::HttpArg) {
    // TODO: Log values from config
    log::info!("Starting HTTP server at 0.0.0.0:{}...", arg.port);

    let get = warp::get().and(routes::index());
    let post = warp::post().and(routes::mailgun(arg.mailgun_key));

    let router = get.or(post);

    warp::serve(router).run(([0, 0, 0, 0], 7777)).await;
}
