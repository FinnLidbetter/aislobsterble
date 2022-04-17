use chrono::DateTime;
use chrono::Utc;
use log::{error};
use reqwest::header::{AUTHORIZATION};
use serde::Deserialize;
use serde_with::{formats::Flexible, TimestampSeconds};
use std::collections::HashMap;

use crate::Config;

const ALMOST_EXPIRED_THRESHOLD_SECONDS: i64 = 20;


#[derive(Debug)]
pub struct SlobsterbleClient {
    client: reqwest::blocking::Client,
    tokens: TokenPair,
    config: Config,
}

impl SlobsterbleClient {

    pub fn new(config: Config) -> SlobsterbleClient {
        let client = reqwest::blocking::Client::new();
        let tokens = TokenPair::default();
        SlobsterbleClient{ client, tokens, config }
    }

    pub fn list_games(&self) -> Result<Vec<GameInfo>, reqwest::Error> {
        if self.tokens.access_token.is_expired() {
            // Return an error to indicate that access token should be refreshed.

        }
        let mut games_path = String::from(&self.config.root_url);
        games_path.push_str("api/games");
        let request = self.client.get(games_path)
            .header(AUTHORIZATION, self.get_access_auth_header());
        let response = request.send()?;
        match response.error_for_status() {
            Ok(response) => response.json::<Vec<GameInfo>>(),
            Err(err) => Err(err),
        }
    }

    pub fn renew_refresh_token(self) -> SlobsterbleClient {
        if !self.tokens.refresh_token.is_almost_expired() {
            return self;
        }
        let tokens = self.get_new_refresh_token();
        match tokens {
            Ok(tokens) => {
                SlobsterbleClient{ client: self.client, tokens, config: self.config }
            },
            Err(err) => {
                error!("Failed to renew refresh token: {}", err);
                self
            }
        }
    }

    fn get_new_refresh_token(&self) -> Result<TokenPair, reqwest::Error> {
        let mut auth_path = String::from(&self.config.root_url);
        auth_path.push_str("api/login");
        let mut map = HashMap::new();
        map.insert("username", &self.config.auth_data.username);
        map.insert("password", &self.config.auth_data.password);
        let response = self.client.post(auth_path).json(&map).send()?;
        match response.error_for_status() {
            Ok(response) => {
                response.json::<TokenPair>()
            },
            Err(err) => Err(err),
        }
    }

    fn renew_access_token(self) -> SlobsterbleClient {
        if !self.tokens.access_token.is_almost_expired() {
            return self;
        }
        if self.tokens.refresh_token.is_almost_expired() {
            return self.renew_refresh_token();
        }
        let access_token = self.get_new_access_token();
        match access_token {
            Ok(access_token) => {
                let tokens = TokenPair{ refresh_token: self.tokens.refresh_token, access_token };
                SlobsterbleClient{ client: self.client, tokens, config: self.config }
            },
            Err(err) => {
                error!("Failed to renew access token: {}", err);
                self
            }
        }
    }

    fn get_new_access_token(&self) -> Result<Token, reqwest::Error> {
        let mut renew_path = String::from(&self.config.root_url);
        renew_path.push_str("api/refresh-access");
        let request = self.client
            .post(renew_path)
            .header(AUTHORIZATION, self.get_refresh_auth_header());
        let response = request.send()?;
        match response.error_for_status() {
            Ok(response) => {
                response.json::<Token>()
            },
            Err(err) => Err(err),
        }
    }

    fn get_access_auth_header(&self) -> String {
        let mut auth_header = String::from("Bearer ");
        auth_header.push_str(&self.tokens.access_token.token);
        auth_header
    }

    fn get_refresh_auth_header(&self) -> String {
        let mut auth_header = String::from("Bearer ");
        auth_header.push_str(&self.tokens.refresh_token.token);
        auth_header
    }
}


#[derive(Debug)]
#[derive(Deserialize)]
struct TokenPair {
    access_token: Token,
    refresh_token: Token,
}

impl TokenPair {
    fn default() -> TokenPair {
        TokenPair { access_token: Token::default(), refresh_token: Token::default() }
    }
}


#[derive(Debug)]
#[serde_with::serde_as]
#[derive(Deserialize)]
struct Token {
    token: String,
    #[serde_as(as = "TimestampSeconds<String, Flexible>")]
    expiration_date: DateTime<Utc>,
}


impl Token {

    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now();
        self.expiration_date < now
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