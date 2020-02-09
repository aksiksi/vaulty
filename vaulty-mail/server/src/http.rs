use warp::{self, Filter};

use super::config;
use super::errors;
use super::routes;

pub async fn run(arg: config::HttpArg) {
    log::info!("Starting HTTP server at 0.0.0.0:{}...", arg.port);

    let mailgun = routes::mailgun(arg.mailgun_key);
    let postfix = routes::postfix();
    let index = routes::index();

    let get = warp::get().and(index);
    let post = warp::post().and(mailgun.or(postfix));

    let router = get.or(post).recover(errors::handle_rejection);

    warp::serve(router).run(([0, 0, 0, 0], arg.port)).await;
}
