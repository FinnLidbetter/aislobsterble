
use std::collections::HashMap;
use crate::models::serializers::GameSerializer;

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct Tile {
    letter: Option<char>,
    value: i32,
    is_blank: bool,
}

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct PlayedTile {
    row: i32,
    column: i32,
    tile: Tile,
}

#[derive(Copy, Clone)]
pub struct Modifier {
    letter_multiplier: i32,
    word_multiplier: i32,
}

pub struct GameBoard {
    rows: i32,
    columns: i32,
    board_tiles: Vec<Vec<Option<Tile>>>,
    modifiers: Vec<Vec<Modifier>>,
}

impl GameBoard {
    pub fn new(game_state: &GameSerializer) -> GameBoard {
        let rows = game_state.board_layout.rows;
        let columns = game_state.board_layout.columns;
        let mut modifier_map = HashMap::new();
        for positioned_modifier in game_state.board_layout.modifiers.iter() {
            let row = positioned_modifier.row;
            let column = positioned_modifier.column;
            let letter_multiplier = positioned_modifier.modifier.letter_multiplier;
            let word_multiplier = positioned_modifier.modifier.word_multiplier;
            let modifier = Modifier{ letter_multiplier, word_multiplier };
            modifier_map.insert((row, column), modifier);
        }
        let mut modifiers: Vec<Vec<Modifier>> = Vec::new();
        let unit_modifier = Modifier{ letter_multiplier: 1, word_multiplier: 1 };
        for row in 0..rows {
            let mut modifier_row: Vec<Modifier> = Vec::new();
            for column in 0..columns {
                let modifier = modifier_map.remove(&(row, column));
                match modifier {
                    Some(modifier) => modifier_row.push(modifier),
                    None => modifier_row.push(unit_modifier.clone()),
                };
            }
            modifiers.push(modifier_row);
        }
        let mut played_tile_map = HashMap::new();
        for played_tile in game_state.board_state.iter() {
            let row = played_tile.row;
            let column = played_tile.column;
            let letter = match &played_tile.tile.letter {
                Some(letter) => Some(letter.chars().next().unwrap()),
                None => None,
            };
            let value = played_tile.tile.value;
            let is_blank = played_tile.tile.is_blank;
            let tile = Tile{ letter, is_blank, value };
            played_tile_map.insert((row, column), tile);
        }
        let mut board_tiles: Vec<Vec<Option<Tile>>> = Vec::new();
        for row in 0..rows {
            let mut board_tiles_row: Vec<Option<Tile>> = Vec::new();
            for column in 0..columns {
                let tile = played_tile_map.remove(&(row, column));
                board_tiles_row.push(tile);
            }
            board_tiles.push(board_tiles_row);
        }
        GameBoard{ rows, columns, board_tiles, modifiers }
    }

    pub fn get_rows(&self) -> i32 {
        self.rows
    }
    pub fn get_columns(&self) -> i32 {
        self.columns
    }
    pub fn get_board_tiles_ref(&self) -> &Vec<Vec<Option<Tile>>> {
        &self.board_tiles
    }
    pub fn get_modifiers_ref(&self) -> &Vec<Vec<Modifier>> {
        &self.modifiers
    }

}

pub struct Rack {
    pub tiles: Vec<Tile>,
}
impl Rack {
    pub fn new(game_state: &GameSerializer) -> Rack {
        let mut tiles = Vec::new();
        for tile_count in game_state.rack.iter() {
            let letter = match &tile_count.tile.letter {
                Some(letter) => Some(letter.chars().next().unwrap()),
                None => None
            };
            let is_blank = tile_count.tile.is_blank;
            let value = tile_count.tile.value;
            let tile = Tile{ letter, is_blank, value };
            for _ in 0..tile_count.count {
                tiles.push(tile.clone());
            }
        }
        Rack{ tiles }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_order() {
        let none_blank_0 = Tile{ letter: None, is_blank: true, value: 1 };
        let a_0 = Tile{ letter: Some('A'), is_blank: false, value: 0};
        let a_blank_0 = Tile{ letter: Some('A'), is_blank: true, value: 0};
        let a_1 = Tile{ letter: Some('A'), is_blank: false, value: 1};
        let a_blank_1 = Tile{ letter: Some('A'), is_blank: true, value: 1};
        let b_0 = Tile{ letter: Some('B'), is_blank: false, value: 0};
        let b_0_copy = Tile{ letter: Some('B'), is_blank: false, value: 0};
        // Letterless blanks are less than non-None letters.
        assert!(none_blank_0 < a_0);
        // Earlier alphabet letter sorts ahead of later alphabet letter.
        assert!(a_0 < b_0);
        // Non-blank letter sorts ahead of blank letter.
        assert!(a_0 < a_blank_0);
        // Lower value sorts ahead of higher value.
        assert!(a_0 < a_1);
        // Letter takes precedence over blankness and value.
        assert!(a_blank_1 < b_0);
        // Value takes precedence over blankness.
        assert!(a_blank_0 < a_1);
        // Tiles with identical entries are equal.
        assert!(b_0 == b_0_copy);
    }

    #[test]
    fn test_played_tile_order() {
        let a_tile = Tile{ letter: Some('A'), is_blank: false, value: 1};
        let b_tile = Tile{ letter: Some('B'), is_blank: false, value: 1};
        let a_1_1 = PlayedTile{ row: 1, column: 1, tile: a_tile.clone()};
        let a_2_1 = PlayedTile{ row: 2, column: 1, tile: a_tile.clone()};
        let a_1_2 = PlayedTile{ row: 1, column: 2, tile: a_tile.clone()};
        let b_1_1 = PlayedTile{ row: 1, column: 1, tile: b_tile.clone()};
        let b_1_2 = PlayedTile{ row: 1, column: 2, tile: b_tile.clone()};
        let b_1_2_copy = PlayedTile{ row: 1, column: 2, tile: b_tile.clone()};
        // Lower row sorts ahead of higher row.
        assert!(a_1_1 < a_2_1);
        // Lower column sorts ahead of higher column.
        assert!(a_1_1 < a_1_2);
        // Lower tile sorts ahead of higher Tile.
        assert!(a_1_1 < b_1_1);
        // Row sorts ahead of column and Tile.
        assert!(b_1_2 < a_2_1);
        // Column sorts ahead of Tile.
        assert!(b_1_1 < a_1_2);
        // PlayedTiles with identical entries are equal.
        assert!(b_1_2 == b_1_2_copy);
    }
}