use std::cmp;
use std::fs;
use std::thread;
use std::time::Duration;
use std::collections::HashSet;

use log;

use crate::models::config_models::Config;
use crate::models::game_models::{Axis, Coordinates, GameBoard, PlayedTile, Rack, Tile};
use crate::models::serializers::{FlatPlayedTileSerializer, GameInfo, GameSerializer, PlayedTileSerializer, TileSerializer};
use crate::slobsterble_client::{SlobsterbleClient};
use crate::utilities::{next_combination, next_permutation};


const PLAY_ATTEMPTS_LIMIT: u32 = 10;
const BLANK_FILLERS: [char; 5] = ['S', 'E', 'R', 'A', 'T'];

pub struct Controller {
    client: SlobsterbleClient,
    config: Config,
    dictionary: HashSet<String>,
}

impl Controller {

    pub fn new(config: Config) -> Controller {
        let dictionary = load_dictionary();
        Controller{ client: SlobsterbleClient::new(config.clone()), config, dictionary }
    }

    fn poll(&mut self) {
        log::debug!("Polling games.");
        let games = match self.client.list_games() {
            Ok(games) => games,
            Err(e) => {
                log::error!("Error fetching games list: {}", e);
                Vec::new()
            }
        };
        let active_games = Controller::filter_active_games(games);
        let potential_ai_turn_games = self.filter_by_ai_name(active_games);
        for game in potential_ai_turn_games.into_iter() {
            let game_state = match self.client.get_game(&game.id.to_string()) {
                Ok(game_state) => game_state,
                Err(e) => {
                    log::error!("Error fetching game state for game {}: {}", &game.id, e);
                    continue;
                },
            };
            if Controller::is_ai_turn(&game_state) {
                let game_board = GameBoard::new(&game_state);
                let rack = Rack::new(&game_state);
                match self.play_turn(&game.id.to_string(), game_board, rack) {
                    Ok(_result_string) => log::debug!("Successfully played turn in game {}", &game.id),
                    Err(result_string) => log::debug!("Failed to play turn in game {}: {}", &game.id, result_string),
                }
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

    fn play_turn(&mut self, game_id: &String, game_board: GameBoard, rack: Rack) -> Result<String, String> {
        log::debug!("Thinking...");
        let mut candidates = self.candidate_plays(&game_board, &rack);
        log::debug!("Determined candidates.");
        candidates.sort_by_key(|pair| -pair.1);
        let attempt_limit = cmp::min(candidates.len(), PLAY_ATTEMPTS_LIMIT as usize);
        for (candidate_play, score) in candidates[..attempt_limit].iter() {
            let mut serializable_play: Vec<FlatPlayedTileSerializer> = Vec::new();
            for played_tile in candidate_play.iter() {
                let row = played_tile.get_coordinates_ref().get_row();
                let column = played_tile.get_coordinates_ref().get_column();
                let letter_for_serializer = match played_tile.get_tile_ref().get_letter() {
                    Some(letter) => Some(String::from(letter)),
                    None => None,
                };
                let is_blank = played_tile.get_tile_ref().is_blank();
                let value = played_tile.get_tile_ref().get_value();
                let tile = TileSerializer{ letter: letter_for_serializer, is_blank, value };
                let is_exchange = false;
                let letter = played_tile.get_tile_ref().get_letter();
                serializable_play.push(
                    FlatPlayedTileSerializer{ is_blank, value, row, column, is_exchange, letter }
                );
            }
            match self.client.play_turn(game_id, &serializable_play) {
                Ok(_response) => {
                    if self.config.check_score {
                        match self.verify_score(game_id, &serializable_play, *score) {
                            Ok(msg) => {
                                log::info!("{}", &msg);
                                return Ok(msg);
                            },
                            Err(err) => {
                                log::error!("{}", err);
                                return Ok(String::from("Error verifying score."));
                            },
                        }
                    } else {
                        let success_message = format!("Successfully played turn in game {}.", game_id);
                        log::info!("{}", &success_message);
                        return Ok(success_message);
                    }
                },
                Err(err) => {
                    let error_message = format!(
                        "Error submitting turn {:?} to game {}. Error: {}",
                        &serializable_play, game_id, err
                    );
                    log::error!("{}", &error_message);
                },
            };
        }
        Err(format!("Failed to successfully play a turn in game {}.", game_id))
    }

    /// Verify that the score calculated by AISlobsterble matches that calculated by Slobsterble.
    fn verify_score(
        &mut self, game_id: &String, played_tiles: &Vec<FlatPlayedTileSerializer>, expected_score: i32
    ) -> Result<String, String> {
        match self.client.get_game(game_id) {
            Ok(after_play_game_state) => {
                let prev_move = after_play_game_state.prev_move;
                match prev_move {
                    Some(prev_move) => {
                        if prev_move.score != expected_score {
                            Err(format!(
                                "Expected score {} but got score {} in game {} with tiles {:?}",
                                expected_score, prev_move.score, game_id, &played_tiles
                            ))
                        } else {
                            Ok(format!(
                                "Successfully played turn in game {} for {} points.",
                                game_id, expected_score
                            ))
                        }
                    },
                    None => {
                        Err(format!("After successful turn play in game {} prev move is none.", game_id))
                    },
                }
            },
            Err(e) => {
                Err(format!(
                    "Failed to get game state in game {} for verifying score calculation. {}",
                    game_id, e
                ))
            },
        }
    }

    fn candidate_plays(&self, game_board: &GameBoard, rack: &Rack) -> Vec<(Vec<PlayedTile>, i32)> {
        if rack.tiles.iter().any(|tile| tile.is_letterless()) {
            let mut candidates: Vec<(Vec<PlayedTile>, i32)> = Vec::new();
            let letterless_count = rack.tiles.iter().filter(|tile| tile.is_letterless()).count();
            if letterless_count == 1 {
                for ch in b'A'..=b'Z' {
                    let ch = ch as char;
                    let filled_rack = rack.fill_blanks(&vec![ch]);
                    log::debug!("{:?}", &filled_rack.tiles);
                    candidates.extend(self.candidate_plays(game_board, &filled_rack));
                }
                return candidates;
            } else {
                let mut letter_fills: Vec<char> = Vec::new();
                for index in 0..letterless_count - 2 {
                    letter_fills.push(BLANK_FILLERS[index % BLANK_FILLERS.len()]);
                }
                letter_fills.push('A');
                letter_fills.push('A');
                for ch_1 in b'A'..=b'Z' {
                    let ch_1 = ch_1 as char;

                    letter_fills[letterless_count - 2] = ch_1;
                    for ch_2 in b'A'..=b'Z' {
                        let ch_2 = ch_2 as char;
                        letter_fills[letterless_count - 1] = ch_2;
                        let filled_rack = rack.fill_blanks(&letter_fills);
                        candidates.extend(self.candidate_plays(game_board, &filled_rack));
                    }
                }
            }
            return candidates;
        }
        let mut candidates: Vec<(Vec<PlayedTile>, i32)> = Vec::new();
        for start_row in 0..game_board.get_rows() {
            for start_column in 0..game_board.get_columns() {
                let start_coordinates = Coordinates::new(start_row, start_column);
                if game_board.is_occupied(&start_coordinates).unwrap_or(true) {
                    continue;
                }
                for axis in Axis::iterator() {
                    for num_tiles in 1..rack.tiles.len() + 1 {
                        // Check that it is ok to play this many tiles at this position.
                        let feasibility_tiles: Vec<&Tile> = (0..num_tiles).map(|index| &rack.tiles[index]).collect();
                        let played_tiles = game_board.build_played_tiles(&start_coordinates, feasibility_tiles, axis);
                        if played_tiles.is_err() {
                            continue;
                        }
                        let played_tiles = played_tiles.unwrap();
                        if !game_board.is_connected(&played_tiles) && !game_board.is_through_center(&played_tiles) {
                            continue;
                        }
                        let mut index_selection: Option<Vec<usize>> = Some((0..num_tiles).collect());
                        while index_selection.is_some() {
                            let mut ordering: Option<Vec<usize>> = Some((0..num_tiles).collect());
                            while ordering.is_some() {
                                let tiles_permutation: Vec<&Tile> = ordering.as_ref().unwrap()
                                    .iter().map(|index| &rack.tiles[index_selection.as_ref().unwrap()[*index]])
                                    .collect();

                                let played_tiles = game_board.build_played_tiles(&start_coordinates, tiles_permutation, axis);
                                let played_tiles = match played_tiles {
                                    Ok(played_tiles) => played_tiles,
                                    Err(e) => {
                                        log::error!("Failed to build played tiles: {}", e);
                                        ordering = next_permutation(ordering.unwrap());
                                        continue;
                                    },
                                };
                                let words_created = game_board.words_created(&played_tiles);
                                if words_created.iter().all(|word| self.dictionary.contains(word)) {
                                    let score = game_board.score(&played_tiles);
                                    candidates.push((played_tiles, score));
                                }
                                ordering = next_permutation(ordering.unwrap());
                            }
                            index_selection = next_combination(index_selection.unwrap(), rack.tiles.len());
                        }
                    }
                }
            }
        }
        candidates
    }

    pub fn run(&mut self) {
        let sleep_duration = Duration::from_secs(self.config.poll_interval_seconds as u64);
        loop {
            self.poll();
            thread::sleep(sleep_duration);
        }
    }
}

fn load_dictionary() -> HashSet<String> {
    let mut dictionary = HashSet::new();
    let words_string = fs::read_to_string("dictionary.txt").expect("Error loading dictionary file.");
    for word in words_string.lines() {
        dictionary.insert(word.to_uppercase());
    }
    dictionary
}


