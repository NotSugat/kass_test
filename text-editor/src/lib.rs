use std::io::Result;

#[derive(Default, Debug, Clone)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

impl Position {
    pub fn new(position_x: u16, position_y: u16) -> Result<Position> {
        Ok(Position {
            x: position_x,
            y: position_y,
        })
    }
    pub fn above(&self, row: usize) -> bool {
        self.y < row as u16
    }

    pub fn left_of(&self, col: usize) -> bool {
        self.x < col as u16
    }

    pub fn row(&self) -> usize {
        self.y as usize
    }
}

pub enum MovementKey {
    Left,
    Right,
    Up,
    Down,
    // Tab,
}
