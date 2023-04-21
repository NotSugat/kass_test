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
}

pub enum MovementKey {
    Left,
    Right,
    Up,
    Down,
}
