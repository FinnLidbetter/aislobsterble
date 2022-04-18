use chrono::DateTime;
use chrono::Utc;
use log::{error};
use reqwest::header::{AUTHORIZATION};
use std::collections::HashMap;

use crate::models::{Config, GameSerializer, GameInfo, TokenPair, Token};



#[derive(Debug)]
pub struct SlobsterbleClient {
    client: reqwest::blocking::Client,
    tokens: TokenPair,
    config: Config,
}

impl SlobsterbleClient {

    /// Initialize a new client but with expired JWTs.
    pub fn new(config: Config) -> SlobsterbleClient {
        let client = reqwest::blocking::Client::new();
        let tokens = TokenPair::default();
        SlobsterbleClient{ client, tokens, config }
    }

    /// Get a list of active or recently completed games for the player.
    pub fn list_games(&mut self) -> Result<Vec<GameInfo>, reqwest::Error> {
        if self.tokens.get_access_token_ref().is_almost_expired() {
            self.renew_access_token();
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

    pub fn get_game(&mut self, game_id: &str) -> Result<GameSerializer, reqwest::Error> {
        let mut game_path = String::from(&self.config.root_url);
        game_path.push_str("api/game/");
        game_path.push_str(game_id);
        if self.tokens.get_access_token_ref().is_almost_expired() {
            self.renew_access_token();
        }
        let request = self.client.get(game_path)
            .header(AUTHORIZATION, self.get_access_auth_header());
        let response = request.send()?;
        match response.error_for_status() {
            Ok(response) => response.json::<GameSerializer>(),
            Err(err) => Err(err),
        }
    }

    /// Renew the refresh token for the client if it has expired or will expire soon.
    pub fn renew_refresh_token(&mut self) {
        if !self.tokens.get_refresh_token_ref().is_almost_expired() {
            ()
        }
        let tokens = self.get_new_refresh_token();
        match tokens {
            Ok(tokens) => {
                self.tokens = tokens;
                ()
            },
            Err(err) => {
                error!("Failed to renew refresh token: {}", err);
                ()
            }
        }
    }

    /// Get a new refresh token, access token pair.
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

    /// Renew the access token if it is expired or will expire soon.
    fn renew_access_token(&mut self) {
        if !self.tokens.get_access_token_ref().is_almost_expired() {
            ()
        }
        if self.tokens.get_refresh_token_ref().is_almost_expired() {
            self.renew_refresh_token()
        }
        let access_token = self.get_new_access_token();
        match access_token {
            Ok(access_token) => {
                let tokens = TokenPair::new(self.tokens.get_refresh_token_ref().clone(), access_token);
                self.tokens = tokens;
                ()
                // SlobsterbleClient{ client: self.client, tokens, config: self.config }
            },
            Err(err) => {
                error!("Failed to renew access token: {}", err);
                ()
            }
        }
    }

    /// Get a new access token.
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

    /// Get the authorization header using the access token.
    fn get_access_auth_header(&self) -> String {
        let mut auth_header = String::from("Bearer ");
        auth_header.push_str(&self.tokens.get_access_token_ref().token());
        auth_header
    }

    /// Get the authorization header using the refresh token.
    fn get_refresh_auth_header(&self) -> String {
        let mut auth_header = String::from("Bearer ");
        auth_header.push_str(&self.tokens.get_refresh_token_ref().token());
        auth_header
    }
}
