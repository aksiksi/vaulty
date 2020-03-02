use std::sync::Arc;

use warp::{self, Filter};

use super::error;
use super::routes;

use vaulty::config::Config;

pub async fn get_db_pool(config: &Config) -> sqlx::PgPool {
    let db_host = &config.db_host;
    let db_name = &config.db_name;
    let db_user = &config.db_user;

    let db_path = if config.db_password.is_some() {
        let db_password = config.db_password.as_ref().unwrap();
        format!(
            "postgres://{}:{}@{}/{}",
            db_user, db_password, db_host, db_name
        )
    } else {
        format!("postgres://{}@{}/{}", db_user, db_host, db_name)
    };

    sqlx::PgPool::new(&db_path).await.unwrap()
}

pub async fn run(arg: Config) {
    let pool = get_db_pool(&arg).await;
    log::info!("Connected to Postgres DB: {}/{}", arg.db_host, arg.db_name);

    // Use Arc to share config across threads on server
    let config = Arc::new(arg);

    let mailgun = routes::mailgun(config.clone());
    let postfix = routes::postfix(pool.clone(), config.clone());
    let index = routes::index();

    let get = warp::get().and(index);
    let post = warp::post().and(mailgun.or(postfix));

    let router = get.or(post).recover(error::handle_rejection);

    let port = config.port;

    log::info!("Starting HTTP server at 0.0.0.0:{}...", port);
    warp::serve(router).run(([0, 0, 0, 0], port)).await;
}
