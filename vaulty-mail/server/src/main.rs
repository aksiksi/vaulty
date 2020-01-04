use warp::{self, Filter};

mod routes;

#[tokio::main]
async fn main() {
    env_logger::builder().format_timestamp_micros().init();

    let _ = std::env::var("MAILGUN_API_KEY")
        .expect("MAILGUN_API_KEY not set in env");

    log::info!("Starting server...");

    let get = warp::get().and(routes::index());
    let post = warp::post().and(routes::mailgun());

    let router = get.or(post);

    warp::serve(router).run(([127, 0, 0, 1], 3030)).await;
}
