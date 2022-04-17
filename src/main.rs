mod slobsterble_client;
mod models;

use log::{info, error};
use std::path::PathBuf;
use std::thread;
use std::time;
use std::process;

use configparser::ini::Ini;



fn main() {
    // Parse configuration.
    let mut config_ini = Ini::new();
    env_logger::init();

    let config_path = get_config_path();
    if let Err(failure_reason) = config_ini.load(config_path) {
        error!("Failed to load config: {}", failure_reason);
        process::exit(1);
    }
    let config = models::Config::new(config_ini);

    let poll_interval_duration = time::Duration::from_secs(config.poll_interval_seconds as u64);

    let client = reqwest::blocking::Client::new();
    let mut my_client = slobsterble_client::SlobsterbleClient::new(config);
    my_client.renew_refresh_token();
    let games = my_client.list_games();
    match games {
        Ok(games) => {
            println!("{:?}", games);
        },
        Err(e) => {
            println!("{:?}", e);
        }
    }
    println!("{:?}", my_client);
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