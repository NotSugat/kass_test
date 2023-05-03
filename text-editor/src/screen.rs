use super::row::*;
use crossterm::{
    self, cursor,
    style::{Attribute, Print, SetAttribute},
    terminal, QueueableCommand,
};
use std::{
    cmp::Ordering,
    env::current_exe,
    io::{stdout, Result, Write},
};
use text_editor::Position;

#[derive(Debug, Clone)]
pub struct Screen {
    width: usize,
    height: usize,
    ln_shift: u16,

    ln_display: bool,
}

const LNO_SHIFT: u16 = 6;

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

    pub fn draw_screen(
        &mut self,
        rows: &[Row],
        rowoff: usize,
        coloff: usize,
        cursor_at: usize,
    ) -> Result<()> {
        for i in 0..(self.height - 2) {
            let row = i + rowoff;
            if row >= rows.len() {
            } else {
                let mut len = rows[row].len();

                if len < coloff {
                    continue;
                }
                len -= coloff;
                let start = coloff;
                let end = start
                    + if len >= (self.width - (LNO_SHIFT as usize)) {
                        self.width - (LNO_SHIFT) as usize
                    } else {
                        len
                    };

                // Displays the relative line number
                let line_order = cursor_at.cmp(&row);

                let relative_ln = match line_order {
                    Ordering::Equal => row + 1,
                    Ordering::Greater => cursor_at - row,
                    Ordering::Less => row - cursor_at,
                };
                stdout()
                    .queue(SetAttribute(Attribute::Reset))?
                    .queue(cursor::MoveTo(0, i as u16))?
                    .queue(if line_order == Ordering::Equal {
                        Print(format!("{:<4}", relative_ln))
                    } else {
                        Print(format!("{:4}", relative_ln))
                    })?
                    .queue(cursor::MoveTo(LNO_SHIFT, i as u16))?
                    .queue(Print(rows[row].chars[start..end].to_string()))?;
            }
        }

        Ok(())
    }
    // draw status bar
    pub fn draw_statusbar(&self, len: usize) -> Result<()> {
        println!("hello world");
        stdout()
            .queue(cursor::MoveTo(LNO_SHIFT, self.height as u16 - 1))?
            .queue(Print(format!("{len} bytes written to the disk")))?;
        Ok(())
    }

    pub fn clear(&self) -> Result<()> {
        stdout().queue(terminal::Clear(terminal::ClearType::All))?;
        Ok(())
    }

    pub fn move_to(&mut self, pos: &Position, rowoff: u16, coloff: u16) -> Result<()> {
        stdout().queue(cursor::MoveTo(pos.x - coloff + LNO_SHIFT, pos.y - rowoff))?;
        Ok(())
    }
    // terminal boundary

    pub fn boundary(&self) -> Position {
        // minus 2 because of the scroll bar at the right side
        Position {
            x: self.width as u16 - LNO_SHIFT,
            y: self.height as u16 - 2,
        }
    }

    // Line number display
    pub fn line_number_display(&self) -> Result<()> {
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
