use config::Config;

pub fn read_config(config_override: &Option<String>) -> Result<Config, config::ConfigError> {
    let mut config = Config::builder()
        .add_source(config::Environment::with_prefix("BALDR"));

    if config_override.is_some() {
        config = config.add_source(config::File::with_name(config_override.as_ref().unwrap().as_str()));
    }
    else {
        let xdg_home = std::env::var("XDG_CONFIG_HOME");
        if xdg_home.is_ok() {
            let xdg_config = std::path::Path::new(&xdg_home.unwrap()).join("baldr");
            config = config.add_source(config::File::with_name(xdg_config.to_str().unwrap()).required(false));
        }

        let home = std::env::var("HOME").expect("HOME env variable is not defined!");
        let home_config = std::path::Path::new(&home).join("baldr");

        config = config
            .add_source(config::File::with_name(home_config.to_str().unwrap()).required(false))
            .add_source(config::File::with_name("./baldr").required(false));
    }

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
                std::path::Path::new(".")
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
