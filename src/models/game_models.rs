use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::slice::Iter;
use crate::models::serializers::GameSerializer;

const BINGO_BONUS: i32 = 50;
const BINGO_TILES_LENGTH: i32 = 7;

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct Tile {
    letter: Option<char>,
    value: i32,
    is_blank: bool,
}
impl Tile {
    pub fn get_letter(&self) -> Option<char> { self.letter }
    pub fn is_blank(&self) -> bool { self.is_blank }
    pub fn get_value(&self) -> i32 { self.value }
    pub fn is_letterless(&self) -> bool { self.letter.is_none() }
}

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct PlayedTile {
    coordinates: Coordinates,
    tile: Tile,
}
impl PlayedTile {
    pub fn get_coordinates_ref(&self) -> &Coordinates {
        &self.coordinates
    }
    pub fn get_tile_ref(&self) -> &Tile {
        &self.tile
    }
}

#[derive(Copy, Clone)]
pub struct Modifier {
    letter_multiplier: i32,
    word_multiplier: i32,
}

pub enum Axis {
    Horizontal,
    Vertical,
}
impl Axis {
    fn complement(&self) -> Axis {
        match self {
            Axis::Vertical => Axis::Horizontal,
            Axis::Horizontal => Axis::Vertical,
        }
    }
    pub fn iterator() -> Iter<'static, Axis> {
        static AXES: [Axis; 2] = [Axis::Horizontal, Axis::Vertical];
        AXES.iter()
    }
}
impl fmt::Display for Axis {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let axis = match self {
            Axis::Horizontal => "Horizontal",
            Axis::Vertical => "Vertical,"
        };
        write!(f, "{}", axis)
    }
}

enum Direction {
    Positive,
    Negative,
}
impl Direction {
    fn multiplier(&self) -> i32 {
        match self {
            Direction::Positive => 1,
            Direction::Negative => -1,
        }
    }
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Coordinates {
    row: i32,
    column: i32,
}
impl Coordinates {
    pub fn new(row: i32, column: i32) -> Coordinates {
        Coordinates{ row, column }
    }
    pub fn get_row(&self) -> i32 { self.row }
    pub fn get_column(&self) -> i32 { self.column }
}
impl fmt::Display for Coordinates {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.row, self.column)
    }
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

    pub fn is_occupied(&self, coordinates: &Coordinates) -> Result<bool, String> {
        let row_bounds_err = format!("Row {} out of bounds for board with {} rows.", coordinates.row, self.rows);
        let column_bounds_err = format!("Column {} out of bounds for board with {} columns.", coordinates.row, self.rows);
        Ok(self.board_tiles.get(coordinates.row as usize).ok_or(row_bounds_err)?
            .get(coordinates.column as usize).ok_or(column_bounds_err)?.is_some())
    }

    /// Return true iff there is a board tile adjacent to at least one played tile.
    pub fn is_connected(&self, played_tiles: &Vec<PlayedTile>) -> bool {
        let adjacency_deltas = [(0, 1), (0, -1), (1, 0), (-1, 0)];
        for played_tile in played_tiles {
            for delta in adjacency_deltas {
                let row_delta = delta.0;
                let column_delta = delta.1;
                let adj_row = played_tile.coordinates.row + row_delta;
                let adj_column = played_tile.coordinates.column + column_delta;
                let board_tiles_row = match self.board_tiles.get(adj_row as usize) {
                    None => continue,
                    Some(value) => value,
                };
                match board_tiles_row.get(adj_column as usize) {
                    None => continue,
                    Some(value) => {
                        if value.is_some() {
                            return true;
                        }
                    }
                };
            }
        }
        false
    }

    /// Return true iff any gaps between played tiles are filled by board tiles.
    pub fn is_continuous(&self, played_tiles: &Vec<PlayedTile>) -> bool {
        if played_tiles.len() <= 1 {
            return true;
        }
        let first = played_tiles.first().unwrap();
        let second = played_tiles.get(1).unwrap();
        let last = played_tiles.last().unwrap();
        let delta = if first.coordinates.row == second.coordinates.row { (0, 1) } else { (1, 0) };
        let mut position = first.coordinates;
        let mut played_tiles_iter = played_tiles.iter();
        let mut current_tile = played_tiles_iter.next();
        while position != last.coordinates {
            match current_tile {
                None => return false,
                Some(current_tile_val) => {
                    if position == current_tile_val.coordinates {
                        current_tile = played_tiles_iter.next();
                    } else {
                        let board_row = match self.board_tiles.get(position.row as usize) {
                            None => return false,
                            Some(val) => val,
                        };
                        match board_row.get(position.column as usize) {
                            None => return false,
                            Some(tile) => if tile.is_none() {
                                return false;
                            }
                        };
                    }
                    position = Coordinates{
                        row: position.row + delta.0,
                        column: position.column + delta.1};
                }
            };
        }
        true
    }

    /// Return true iff the played tiles go through the center of the board.
    pub fn is_through_center(&self, played_tiles: &Vec<PlayedTile>) -> bool {
        let center = Coordinates{ row: self.rows / 2, column: self.columns / 2 };
        for tile in played_tiles.iter() {
            if tile.coordinates == center {
                return true;
            }
        }
        false
    }

    /// Return true iff all positions of played tiles are available for play.
    pub fn is_available(&self, played_tiles: &Vec<PlayedTile>) -> bool {
        for played_tile in played_tiles.iter() {
            let board_row = match self.board_tiles.get(played_tile.coordinates.row as usize) {
                None => return false,
                Some(val) => val,
            };
            match board_row.get(played_tile.coordinates.column as usize) {
                None => return false,
                Some(val) => {
                    if val.is_some() {
                        return false;
                    }
                }
            }
        }
        true
    }

    pub fn build_played_tiles(
            &self, start_coordinates: &Coordinates, tiles: Vec<&Tile>, axis: &Axis
    ) -> Result<Vec<PlayedTile>, String> {
        let mut played_tiles = Vec::new();
        let mut position = start_coordinates.clone();
        let delta = match axis {
            Axis::Horizontal => (0, 1),
            Axis::Vertical => (1, 0),
        };
        let row_limit_err = format!(
            "Not enough rows on the board to play {} tiles on the {} axis from {}",
            tiles.len(), &axis, &start_coordinates
        );
        let column_limit_err = format!(
            "Not enough columns on the board to play {} tiles on the {} axis from {}",
            tiles.len(), &axis, &start_coordinates
        );
        for (tile_index, tile) in tiles.iter().enumerate() {
            let mut board_row = self.board_tiles.get(position.row as usize).ok_or(&row_limit_err)?;
            let mut board_tile = board_row.get(position.column as usize).ok_or(&column_limit_err)?;
            if board_tile.is_some() && tile_index == 0 {
                return Err(format!("Start position {} is occupied", &start_coordinates));
            }
            while board_tile.is_some() {
                position = Coordinates{ row: position.row + delta.0, column: position.column + delta.1 };
                board_row = self.board_tiles.get(position.row as usize).ok_or(&row_limit_err)?;
                board_tile = board_row.get(position.column as usize).ok_or(&column_limit_err)?;
            }
            played_tiles.push(PlayedTile{ coordinates: position.clone(), tile: *tile.clone()});
            position = Coordinates{ row: position.row + delta.0, column: position.column + delta.1 };
        }
        Ok(played_tiles)
    }

    fn played_tile_map(played_tiles: &Vec<PlayedTile>) -> HashMap<Coordinates, &PlayedTile> {
        let mut played_tile_map: HashMap<Coordinates, &PlayedTile> = HashMap::new();
        for played_tile in played_tiles.iter() {
            played_tile_map.insert(played_tile.coordinates.clone(), &played_tile);
        }
        played_tile_map
    }

    pub fn build_word(&self, start: Coordinates, end: Coordinates, played_tile_map: &HashMap<Coordinates, &PlayedTile>) -> String {
        let axis = if start.row == end.row { Axis::Horizontal } else { Axis::Vertical };
        let delta = match axis {
            Axis::Horizontal => (0, 1),
            Axis::Vertical => (1, 0),
        };
        let mut position = start.clone();
        let mut handled_inclusive = false;
        let mut word = String::new();
        while position != end || !handled_inclusive {
            let letter = match self.board_tiles.get(position.row as usize).unwrap().get(position.column as usize).unwrap() {
                Some(tile) => tile.letter.expect("A blank letter was found on the board."),
                None => played_tile_map.get(&position).expect("No played tile in empty board space in iteration bounds for building a word.").tile.letter.expect("A blank letter was played."),
            };
            word.push(letter);
            if position == end {
                handled_inclusive = true;
            } else {
                position = Coordinates{ row: position.row + delta.0, column: position.column + delta.1 };
            }
        }
        word
    }

    pub fn words_created(&self, played_tiles: &Vec<PlayedTile>) -> Vec<String> {
        let primary_axis = self.primary_axis(played_tiles);
        let secondary_axis = primary_axis.complement();
        let played_tile_map = GameBoard::played_tile_map(played_tiles);
        let mut words = Vec::new();
        let primary_start = self.min_connected_position(&played_tiles.first().unwrap().coordinates, &primary_axis);
        let primary_end = self.max_connected_position(&played_tiles.last().unwrap().coordinates, &primary_axis);
        words.push(self.build_word(primary_start, primary_end, &played_tile_map));
        for played_tile in played_tiles.iter() {
            let start = self.min_connected_position(&played_tile.coordinates, &secondary_axis);
            let end = self.max_connected_position(&played_tile.coordinates, &secondary_axis);
            if start != end {
                words.push(self.build_word(start, end, &played_tile_map));
            }
        }
        words
    }

    fn primary_axis(&self, played_tiles: &Vec<PlayedTile>) -> Axis {
        match played_tiles.len().cmp(&1) {
            Ordering::Less => panic!("Cannot find the primary axis of no tiles."),
            Ordering::Equal => {
                let played_tile = played_tiles.first().unwrap();
                let horizontal_min = self.min_connected_position(&played_tile.coordinates, &Axis::Horizontal);
                let horizontal_max = self.max_connected_position(&played_tile.coordinates, &Axis::Horizontal);
                if horizontal_min == horizontal_max {
                    Axis::Vertical
                } else {
                    Axis::Vertical
                }
            },
            Ordering::Greater => {
                let tile_1 = played_tiles.get(0).unwrap();
                let tile_2 = played_tiles.get(1).unwrap();
                if tile_1.coordinates.row == tile_2.coordinates.row {
                    Axis::Horizontal
                } else {
                    Axis::Vertical
                }
            },
        }
    }

    pub fn score(&self, played_tiles: &Vec<PlayedTile>) -> i32 {
        let mut total = 0;
        let primary_axis = self.primary_axis(played_tiles);
        let secondary_axis = primary_axis.complement();
        if played_tiles.len() == 0 {
            return 0;
        } else if played_tiles.len() > 1 {
            // Score the secondary axis for each tile.
            for played_tile in played_tiles.iter() {
                total += self.score_secondary_axis(played_tile, &secondary_axis);
            }
        }
        total += self.score_primary_axis(played_tiles, primary_axis);
        if played_tiles.len() as i32 == BINGO_TILES_LENGTH {
            total += BINGO_BONUS;
        }
        return total;
    }

    fn score_secondary_axis(&self, played_tile: &PlayedTile, axis: &Axis) -> i32 {
        let mut total = 0;
        let modifier = self.modifiers
            .get(played_tile.coordinates.row as usize).unwrap()
            .get(played_tile.coordinates.column as usize).unwrap();
        let word_multiplier = modifier.word_multiplier;
        let start_position = self.min_connected_position(&played_tile.coordinates, axis);
        let end_position = self.max_connected_position(&played_tile.coordinates, axis);
        if start_position == end_position {
            return 0;
        }
        let delta = match axis {
            Axis::Horizontal => (0, 1),  Axis::Vertical => (1, 0),
        };
        let mut position = start_position.clone();
        while position != end_position {
            match self.board_tiles.get(position.row as usize).unwrap().get(position.column as usize).unwrap() {
                None => {
                    if played_tile.coordinates != position {
                        panic!("Encountered empty position in secondary axis iteration.");
                    }
                    total += played_tile.tile.value * modifier.letter_multiplier;
                },
                Some(board_tile) => {
                    total += board_tile.value;
                },
            };
            position = Coordinates{ row: position.row + delta.0, column: position.column + delta.1 };
        }
        total *= word_multiplier;
        return total;
    }

    fn score_primary_axis(&self, played_tiles: &Vec<PlayedTile>, axis: Axis) -> i32 {
        let mut total = 0;
        let mut word_multiplier = 1;
        if played_tiles.len() == 0 {
            return 0;
        }
        let mut played_tile_map: HashMap<Coordinates, &PlayedTile> = HashMap::new();
        for played_tile in played_tiles.iter() {
            played_tile_map.insert(played_tile.coordinates.clone(), &played_tile);
        }
        let coordinate_min = self.min_connected_position(&played_tiles.first().unwrap().coordinates, &axis);
        let coordinate_max = self.min_connected_position(&played_tiles.last().unwrap().coordinates, &axis);
        let delta = match axis {
            Axis::Horizontal => (0, 1), Axis::Vertical => (1, 0)
        };
        let mut position = coordinate_min.clone();
        let mut handled_inclusive = false;
        while position != coordinate_max || !handled_inclusive {
            let board_tile = self.board_tiles
                .get(position.row as usize).unwrap()
                .get(position.column as usize).unwrap();
            match board_tile {
                Some(board_tile) => {
                    total += board_tile.value;
                },
                None => {
                    let played_tile = played_tile_map.get(&position).unwrap();
                    let modifier = self.modifiers
                        .get(position.row as usize).unwrap()
                        .get(position.column as usize).unwrap();
                    total += played_tile.tile.value * modifier.letter_multiplier;
                    word_multiplier *= modifier.word_multiplier;
                },
            }
            if position == coordinate_max && !handled_inclusive {
                handled_inclusive = true;
                continue;
            }
            position = Coordinates{ row: position.row + delta.0, column: position.column + delta.1 };
        }
        total *= word_multiplier;
        return total;
    }

    fn min_connected_position(&self, start_position: &Coordinates, axis: &Axis) -> Coordinates {
        self.extremal_connected_position(start_position, axis, Direction::Negative)
    }
    fn max_connected_position(&self, start_position: &Coordinates, axis: &Axis) -> Coordinates {
        self.extremal_connected_position(start_position, axis, Direction::Positive)
    }
    fn extremal_connected_position(&self, start_position: &Coordinates, axis: &Axis, direction: Direction) -> Coordinates {
        let delta = match axis {
            Axis::Horizontal => (0, direction.multiplier()),
            Axis::Vertical => (direction.multiplier(), 0),
        };
        let mut min_position = start_position.clone();
        let mut adj_position = Coordinates{ row: min_position.row + delta.0, column: min_position.column + delta.1 };
        loop {
            let board_row = match self.board_tiles.get(adj_position.row as usize) {
                None => return min_position,
                Some(board_row_val) => board_row_val,
            };
            match board_row.get(adj_position.column as usize) {
                None => return min_position,
                Some(board_entry) => {
                    if board_entry.is_none() {
                        return min_position;
                    } else {
                        min_position = adj_position;
                        adj_position = Coordinates{ row: min_position.row + delta.0, column: min_position.column + delta.1 };
                    }
                }
            }
        }
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

    pub fn fill_blanks(&self, letter_fills: &Vec<char>) -> Rack {
        let blank_count = self.tiles.iter().filter(|tile| tile.is_letterless()).count();
        if letter_fills.len() != blank_count {
            panic!("Mismatch in number of blanks and letter fillers.");
        }
        let mut tiles: Vec<Tile> = Vec::new();
        let mut fill_index = 0;
        for tile in &self.tiles {
            if tile.is_letterless() {
                tiles.push(Tile{ letter: Some(letter_fills[fill_index]), is_blank: tile.is_blank, value: tile.value });
                fill_index += 1;
            } else {
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
        let a_1_1 = PlayedTile{ coordinates: Coordinates{ row: 1, column: 1 }, tile: a_tile.clone()};
        let a_2_1 = PlayedTile{ coordinates: Coordinates{ row: 2, column: 1 }, tile: a_tile.clone()};
        let a_1_2 = PlayedTile{ coordinates: Coordinates{ row: 1, column: 2 }, tile: a_tile.clone()};
        let b_1_1 = PlayedTile{ coordinates: Coordinates{ row: 1, column: 1 }, tile: b_tile.clone()};
        let b_1_2 = PlayedTile{ coordinates: Coordinates{ row: 1, column: 2 }, tile: b_tile.clone()};
        let b_1_2_copy = PlayedTile{ coordinates: Coordinates{ row: 1, column: 2 }, tile: b_tile.clone()};
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