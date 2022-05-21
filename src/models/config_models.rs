use chrono::DateTime;
use chrono::Utc;
use configparser::ini::Ini;
use serde::Deserialize;
use serde_with::{formats::Flexible, TimestampSeconds};
const ALMOST_EXPIRED_THRESHOLD_SECONDS: i64 = 20;

#[derive(Debug)]
#[derive(Clone)]
pub struct Config {
    pub root_url: String,
    pub ai_display_name: String,
    pub check_score: bool,
    pub poll_interval_seconds: u32,
    pub log_level: String,
    pub auth_data: AuthData,
}

impl Config {
    pub fn new(config_ini: Ini) -> Config {
        let root_url = config_ini.get("slobsterble", "root_url").unwrap();
        let username = config_ini.get("aislobsterble", "username").unwrap();
        let password = config_ini.get("aislobsterble", "password").unwrap();
        let check_score = config_ini.getboolcoerce("aislobsterble", "check_score")
            .unwrap_or(Some(false)).unwrap_or(false);
        let ai_display_name = config_ini.get("aislobsterble", "display_name").unwrap();
        let poll_interval_seconds = config_ini
            .getint("aislobsterble", "poll_interval_seconds")
            .unwrap().unwrap() as u32;
        let auth_data = AuthData { username, password };
        let log_level = config_ini.get("aislobsterble", "log_level").unwrap();
        Config { root_url, ai_display_name, check_score, poll_interval_seconds, log_level, auth_data }
    }
}

#[derive(Debug)]
#[derive(Clone)]
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
