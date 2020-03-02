use super::config;
use super::error::Error;

use warp::{Filter, Rejection};

/// Simple filter for HTTP Basic Authentication
/// Currently just checks against a static user/pass
pub fn basic_auth() -> impl Filter<Extract = (), Error = Rejection> + Clone {
    warp::header::<String>("Authorization")
        .and_then(|auth: String| async move {
            let full = format!("{}:{}", config::VAULTY_USER, config::VAULTY_PASS);

            if !auth.contains(&base64::encode(&full)) {
                Err(warp::reject::custom(Error::Unauthorized))
            } else {
                Ok(())
            }
        })
        .untuple_one()
}
