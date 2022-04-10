use configparser::ini::Ini;
use std::path::PathBuf;
use std::{thread, time};


fn main() {
    println!("Hello, world!");
    // Parse configuration.
    let mut config = Ini::new();

    let config_path = get_config_path();
    let _settings = match config.load(config_path) {
        Err(failure_reason) => panic!("{}", failure_reason),
        Ok(inner) => inner,
    };
    let poll_interval_seconds = config.getint("lobster", "poll_interval_seconds").unwrap().unwrap();
    let poll_interval_duration = time::Duration::from_secs(poll_interval_seconds as u64);

    let connection_data = ConnectionData {
        root_url: String::from(config.get("slobsterble", "root_url").unwrap().unwrap());
        username: String::from(config.get("lobster", "username").unwrap().unwrap());
        password: String::from(config.get("lobster", "password").unwrap().unwrap());
    }

    // Start polling.
    loop {
        let games_to_play = get_games_to_play(&connection_data);
        for game_id in games_to_play {
            play_turn(&connection_data, game_id);
        }
        thread::sleep(poll_interval_duration);
    }
}

/// Structure for storing server path and login credentials.
struct ConnectionData {
    root_url: String,
    username: String,
    password: String,
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
    let prod_config_path = prod_config_dir.as_path().join(".lobster.conf");
    let mut config_path = prod_config_path;
    if !config_path.as_path().exists() {
        config_path = default_config_path;
    }
    config_path
}