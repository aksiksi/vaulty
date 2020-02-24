use std::collections::HashMap;

pub const DEFAULT_PATH: &str = "/etc/vaulty/vaulty.toml";
const ENV_PREFIX: &str = "VAULTY_";

/// Loads Vaulty config from filesystem and merges it with any
/// environment variables prefixed with VAULTY_.
///
/// This function will panic on error.
///
/// See sample config file in `examples` for valid keys.
pub fn load_config(path: Option<&str>) -> HashMap<String, String> {
    let mut settings = config::Config::default();

    settings
        .merge(config::File::with_name(path.unwrap_or(DEFAULT_PATH)))
        .unwrap()
        .merge(config::Environment::with_prefix(ENV_PREFIX))
        .unwrap();

    settings.try_into::<HashMap<String, String>>().unwrap()
}
