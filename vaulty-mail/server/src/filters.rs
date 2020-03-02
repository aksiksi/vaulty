use std::sync::Arc;

use super::error::Error;

use vaulty::config::Config;

use warp::{filters::BoxedFilter, Filter};

/// Simple filter for HTTP Basic Authentication
///
/// User and pass checked against those set in config file
pub fn basic_auth(config: Arc<Config>) -> BoxedFilter<()> {
    warp::header::<String>("Authorization")
        .and(warp::any().map(move || config.clone()))
        .and_then(|auth: String, config: Arc<Config>| async move {
            let user = &config.auth_user;
            let pass = &config.auth_pass;

            let full = format!("{}:{}", user, pass);

            if !auth.contains(&base64::encode(&full)) {
                Err(warp::reject::custom(Error::Unauthorized))
            } else {
                Ok(())
            }
        })
        .untuple_one()
        .boxed()
}
