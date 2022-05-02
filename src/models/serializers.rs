use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_with::{formats::Flexible, TimestampSeconds};

#[derive(Debug)]
#[serde_with::serde_as]
#[derive(Deserialize)]
pub struct GameInfo {
    #[serde_as(as = "TimestampSeconds<String, Flexible>")]
    pub started: DateTime<Utc>,
    #[serde_as(as = "Option<TimestampSeconds<String, Flexible>>")]
    pub completed: Option<DateTime<Utc>>,
    pub whose_turn_name: String,
    pub game_players: GamePlayerInfo,
    pub id: String,
}

#[derive(Debug)]
#[derive(Deserialize)]
pub struct GamePlayerInfo {
    pub score: i32,
    pub player: PlayerInfo,
    pub turn_order: i32,
}
#[derive(Debug)]
#[derive(Deserialize)]
pub struct PlayerInfo {
    pub id: String,
    pub display_name: String,
}

#[derive(Deserialize)]
pub struct GameSerializer {
    pub board_state: Vec<PlayedTileSerializer>,
    pub game_players: Vec<GamePlayerSerializer>,
    pub board_layout: BoardLayoutSerializer,
    pub turn_number: i32,
    pub whose_turn_name: String,
    pub num_tiles_remaining: i32,
    pub rack: Vec<TileCountSerializer>,
    pub prev_move: Option<PrevMoveSerializer>,
    pub fetcher_player_id: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayedTileSerializer {
    pub tile: TileSerializer,
    pub row: i32,
    pub column: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TileSerializer {
    pub letter: Option<String>,
    pub is_blank: bool,
    pub value: i32,
}

#[derive(Deserialize)]
pub struct TileCountSerializer {
    pub tile: TileSerializer,
    pub count: i32,
}

#[derive(Deserialize)]
pub struct GamePlayerSerializer {
    pub score: i32,
    pub turn_order: i32,
    pub player: PlayerSerializer,
    pub num_tiles_remaining: i32,
}

#[derive(Deserialize)]
pub struct PlayerSerializer {
    pub id: i32,
    pub display_name: String,
}


#[derive(Deserialize)]
pub struct BoardLayoutSerializer {
    pub rows: i32,
    pub columns: i32,
    pub modifiers: Vec<PositionedModifierSerializer>,
}

#[derive(Deserialize)]
pub struct PositionedModifierSerializer {
    pub row: i32,
    pub column: i32,
    pub modifier: ModifierSerializer,
}

#[derive(Deserialize)]
pub struct ModifierSerializer {
    pub word_multiplier: i32,
    pub letter_multiplier: i32,
}

#[derive(Deserialize)]
pub struct PrevMoveSerializer {
    pub word: Option<String>,
    pub score: i32,
    pub player_id: i32,
    pub display_name: String,
    pub exchanged_count: i32,
}