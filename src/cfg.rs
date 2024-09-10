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
