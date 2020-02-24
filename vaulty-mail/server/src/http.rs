use warp::{self, Filter};

use super::config;
use super::errors;
use super::routes;

pub async fn get_db_pool(arg: &config::Config) -> sqlx::PgPool {
    let db_host = &arg.db_host;
    let db_name = &arg.db_name;
    let db_user = &arg.db_user;

    let db_path = if arg.db_password.is_some() {
        let db_password = arg.db_password.as_ref().unwrap();
        format!(
            "postgres://{}:{}@{}/{}",
            db_user, db_password, db_host, db_name
        )
    } else {
        format!("postgres://{}@{}/{}", db_user, db_host, db_name)
    };

    sqlx::PgPool::new(&db_path).await.unwrap()
}

pub async fn run(arg: config::Config) {
    let pool = get_db_pool(&arg).await;
    log::info!("Connected to Postgres DB: {}/{}", arg.db_host, arg.db_name);

    let mailgun = routes::mailgun(arg.mailgun_key);
    let postfix = routes::postfix(pool.clone());
    let index = routes::index();

    let get = warp::get().and(index);
    let post = warp::post().and(mailgun.or(postfix));

    let router = get.or(post).recover(errors::handle_rejection);

    let port = arg.port;

    log::info!("Starting HTTP server at 0.0.0.0:{}...", port);
    warp::serve(router).run(([0, 0, 0, 0], port)).await;
}
