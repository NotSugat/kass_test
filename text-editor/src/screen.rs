use crossterm::{
    self, cursor, execute,
    style::{Attribute, Print, SetAttribute},
    QueueableCommand,
};
use std::io::{stdout, Result, Write};
use text_editor::Position;

#[derive(Debug, Clone)]
pub struct Screen {
    width: usize,
    height: usize,
    ln_shift: u16,

    ln_display: bool,
}

const LNO_SHIFT: u16 = 7;

impl Screen {
    pub fn new() -> Result<Self> {
        let ln_display = true;
        let (terminal_width, terminal_height) = crossterm::terminal::size()?;
        Ok(Screen {
            width: (terminal_width) as usize,
            height: (terminal_height) as usize,
            ln_display: true,
            ln_shift: if ln_display { LNO_SHIFT } else { 0 },
        })
    }

    pub fn draw_screen(&mut self, rows: &[String], rowoff: usize) -> Result<()> {
        for i in 0..(self.height - 2) {
            let row = i + rowoff;
            if row >= rows.len() {
            } else {
                let len = rows[row].len().min(self.width);
                stdout()
                    .queue(cursor::MoveTo(0, i as u16))?
                    .queue(Print(rows[row][0..len].to_string()))?;
            }
        }
        Ok(())
    }

    pub fn move_to(&mut self, pos: &Position, rowoff: u16) -> Result<()> {
        stdout().queue(cursor::MoveTo(pos.x, pos.y - rowoff))?;
        Ok(())
    }
    // terminal boundary

    pub fn boundary(&self) -> Position {
        // minus 2 because of the scroll bar at the right side
        Position {
            x: self.width as u16,
            y: self.height as u16 - 2,
        }
    }

    // Line number display
    fn line_number_display(&self) -> Result<()> {
        for i in 0..self.height - 1 {
            stdout()
                .queue(SetAttribute(Attribute::Reset))?
                .queue(Print(format!("{:3} ", i + 1)))?
                .queue(cursor::MoveTo(0, i as u16))?;
        }

        // execute!(stdout(), cursor::MoveTo(self.position_x as u16, 0))?;
        stdout().flush()?;
        Ok(())
    }
}
