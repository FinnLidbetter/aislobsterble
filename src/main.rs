mod slobsterble_client;
mod controller;
mod models;
mod utilities;

use log;
use std::path::PathBuf;
use std::process;

use configparser::ini::Ini;
use crate::controller::Controller;


fn main() {
    // Parse configuration.
    let mut config_ini = Ini::new();
    env_logger::init();

    let config_path = get_config_path();
    if let Err(failure_reason) = config_ini.load(config_path) {
        log::error!("Failed to load config: {}", failure_reason);
        process::exit(1);
    }
    let config = models::config_models::Config::new(config_ini);
    let mut controller = Controller::new(config);
    controller.run();
}



/// Get a path to the configuration file.
fn get_config_path() -> PathBuf {
    let mut default_config_path = PathBuf::new();
    default_config_path.push("defaults.conf");
    let home = dirs::home_dir();
    let prod_config_dir = match home {
        None => {
            let mut default_dir = PathBuf::new();
            default_dir.push("/home/ubuntu");
            default_dir
        },
        Some(val) => val,
    };
    let prod_config_path = prod_config_dir.as_path().join(".aislobsterble.conf");
    let mut config_path = prod_config_path;
    if !config_path.as_path().exists() {
        config_path = default_config_path;
    }
    config_path
}