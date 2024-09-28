use config::builder::DefaultState;
use config::{Config, ConfigBuilder};

use std::env;
use std::path::Path;

fn read_one_config(var: &str, cfg: ConfigBuilder<DefaultState>) -> ConfigBuilder<DefaultState> {
    if let Ok(x) = env::var(var) {
        log::debug!("Config found in {var}.");

        let config_path = Path::new(&x).join("baldr");

        cfg.add_source(
            config::File::with_name(config_path.to_str().expect("Non UTF-8 string in path")
        ).required(false))
    }
    else {
        log::debug!("{var} is not defined.");
        cfg
    }
}

/// Read configuration from environment variables and files.
///
/// Files are looked in the following directories:
/// * XDG_CONFIG_HOME
/// * HOME
/// * Current working directory
///
/// # Errors
///
/// Returns an error if config files exist but cannot be read or the configuration is invalid.
pub fn read_config(config_override: &Option<String>) -> Result<Config, config::ConfigError> {
    let mut config = Config::builder()
        .add_source(config::Environment::with_prefix("BALDR"));

    config = match config_override {
        Some(x) => {
            config.add_source(config::File::with_name(x.as_str()))
        },
        None => {
            config = read_one_config("XDG_CONFIG_HOME", config);
            config = read_one_config("HOME", config);
            config.add_source(config::File::with_name("./baldr").required(false))
        }
    };

    config.build()
}

pub fn get_cc(cfg: &Config) -> Option<String> {
    match cfg.get_string("compiler.cc") {
        Ok(x) => Some(x),
        Err(_) => None,
    }
}

pub fn get_cxx(cfg: &Config) -> Option<String> {
    match cfg.get_string("compiler.cxx") {
        Ok(x) => Some(x),
        Err(_) => None,
    }
}

pub fn get_cmake_definitions(cfg: &Config) -> Vec<String> {
    match cfg.get_array("cmake.definitions") {
        Ok(x) =>
            x.iter()
                .filter_map(|x| x.clone().into_string().ok())
                .collect(),
        Err(_) => [].to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> Config {
        Config::builder().add_source(
            config::File::with_name(
                Path::new(".")
                    .join("examples")
                    .join("baldr")
                    .to_str()
                    .unwrap()
            )
        ).build().unwrap()
    }

    #[test]
    fn cfg_cc() {
        assert_eq!(get_cc(&config()), Some("gcc".into()));
    }

    #[test]
    fn cfg_cxx() {
        assert_eq!(get_cxx(&config()), Some("g++".into()));
    }

    #[test]
    fn cfg_cmake_definitions() {
        assert_eq!(
            get_cmake_definitions(&config()),
            vec![
                "CFG1=cfg1",
                "CFG2=cfg2",
            ]
        );
    }
}
