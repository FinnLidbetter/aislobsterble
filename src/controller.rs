use log::{error};
use std::thread;
use std::time::Duration;

use crate::models::{Config, GameInfo, GameSerializer, Rack, GameBoard};
use crate::slobsterble_client::{SlobsterbleClient};

pub struct Controller {
    client: SlobsterbleClient,
    config: Config,
}

impl Controller {

    pub fn new(config: Config) -> Controller {
        Controller{ client: SlobsterbleClient::new(config.clone()), config }
    }

    fn poll(&mut self) {
        let games = match self.client.list_games() {
            Ok(games) => games,
            Err(e) => {
                error!("Error fetching games list: {}", e);
                Vec::new()
            }
        };
        let active_games = Controller::filter_active_games(games);
        let potential_ai_turn_games = self.filter_by_ai_name(active_games);
        for game in potential_ai_turn_games.into_iter() {
            let game_state = match self.client.get_game(&game.id) {
                Ok(game_state) => game_state,
                Err(e) => {
                    error!("Error fetching game state for game {}: {}", &game.id, e);
                    continue;
                },
            };
            if Controller::is_ai_turn(&game_state) {
                let game_board = GameBoard::new(&game_state);
                let rack = Rack::new(&game_state);
                self.play_turn(game_board, rack);
            }
        }
        ()
    }

    /// Filter a list of games down to those that are not completed.
    fn filter_active_games(games: Vec<GameInfo>) -> Vec<GameInfo> {
        games.into_iter().filter(|game| game.completed.is_none()).collect()
    }

    /// Filter a list of games down to those whose turn name matches the configuration.
    ///
    /// Due to a weakness of the API, this is not sufficient to identify games in which
    /// it is the AI player's turn. The list_games API should be updated to provide a
    /// unique identifier for the player whose turn it is. Nevertheless, for now this
    /// will work in all cases where someone is not trying to test the limits and mimic
    /// the display name of the AI player.
    fn filter_by_ai_name(&self, games: Vec<GameInfo>) -> Vec<GameInfo> {
        games.into_iter().filter(|game|
            game.whose_turn_name.eq(&self.config.ai_display_name)
        ).collect()
    }

    /// Return True iff it is the AI player's turn.
    fn is_ai_turn(game_state: &GameSerializer) -> bool {
        let num_players = game_state.game_players.len() as i32;
        let turn_order_to_match = game_state.turn_number % num_players;
        match game_state.game_players.iter().find(|game_player|
            game_player.turn_order == turn_order_to_match
        ) {
            Some(game_player) => game_player.player.id == game_state.fetcher_player_id,
            None => false,
        }
    }

    fn play_turn(&self, game_board: GameBoard, rack: Rack) {

    }

    pub fn run(&mut self) {
        let sleep_duration = Duration::from_secs(self.config.poll_interval_seconds as u64);
        loop {
            self.poll();
            thread::sleep(sleep_duration);
        }
    }
}