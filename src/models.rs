use chrono::DateTime;
use chrono::Utc;
use configparser::ini::Ini;
use serde::Deserialize;
use serde_with::{formats::Flexible, TimestampSeconds};


const ALMOST_EXPIRED_THRESHOLD_SECONDS: i64 = 20;

#[derive(Debug)]
pub struct Config {
    pub root_url: String,
    pub poll_interval_seconds: u32,
    pub auth_data: AuthData,
}

impl Config {
    pub fn new(config_ini: Ini) -> Config {
        let root_url = config_ini.get("slobsterble", "root_url").unwrap().clone();
        let username = config_ini.get("aislobsterble", "username").unwrap().clone();
        let password = config_ini.get("aislobsterble", "password").unwrap().clone();
        let poll_interval_seconds = config_ini
            .getint("aislobsterble", "poll_interval_seconds")
            .unwrap().unwrap() as u32;
        let auth_data = AuthData { username, password };
        Config { root_url, poll_interval_seconds, auth_data }
    }
}

#[derive(Debug)]
pub struct AuthData {
    pub username: String,
    pub password: String,
}


#[derive(Debug)]
#[derive(Clone, Deserialize)]
pub struct TokenPair {
    access_token: Token,
    refresh_token: Token,
}

impl TokenPair {
    pub fn default() -> TokenPair {
        TokenPair { access_token: Token::default(), refresh_token: Token::default() }
    }

    pub fn new(refresh_token: Token, access_token: Token) -> TokenPair {
        TokenPair{ refresh_token, access_token }
    }

    pub fn get_refresh_token_ref(&self) -> &Token {
        &self.refresh_token
    }

    pub fn get_access_token_ref(&self) -> &Token {
        &self.access_token
    }

    pub fn set_refresh_token(&mut self, refresh_token: Token) {
        self.refresh_token = refresh_token;
    }
    pub fn set_access_token(&mut self, access_token: Token) {
        self.access_token = access_token;
    }
}


#[derive(Debug)]
#[serde_with::serde_as]
#[derive(Clone, Deserialize)]
pub struct Token {
    token: String,
    #[serde_as(as = "TimestampSeconds<String, Flexible>")]
    expiration_date: DateTime<Utc>,
}


impl Token {

    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now();
        self.expiration_date < now
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn is_almost_expired(&self) -> bool {
        let now = chrono::Utc::now();
        let almost_expired_threshold_duration = chrono::Duration::seconds(ALMOST_EXPIRED_THRESHOLD_SECONDS);
        self.expiration_date < now + almost_expired_threshold_duration
    }

    fn default() -> Token {
        let epoch = chrono::DateTime::<Utc>::from(std::time::UNIX_EPOCH);
        Token { token: String::from(""), expiration_date: epoch }
    }
}

#[derive(Debug)]
#[serde_with::serde_as]
#[derive(Deserialize)]
pub struct GameInfo {
    #[serde_as(as = "TimestampSeconds<String, Flexible>")]
    started: DateTime<Utc>,
    #[serde_as(as = "Option<TimestampSeconds<String, Flexible>>")]
    completed: Option<DateTime<Utc>>,
    whose_turn_name: String,
    game_players: GamePlayerInfo,
    id: String,
}

#[derive(Debug)]
#[derive(Deserialize)]
struct GamePlayerInfo {
    score: i32,
    player: PlayerInfo,
    turn_order: i32,
}
#[derive(Debug)]
#[derive(Deserialize)]
struct PlayerInfo {
    id: String,
    display_name: String,
}

#[derive(Deserialize)]
pub struct Game {

}