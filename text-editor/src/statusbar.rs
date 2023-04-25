use std::io::{stdout, Result};

use crossterm::{
    cursor, queue,
    style::{Color, Print, SetForegroundColor, Stylize},
};

#[derive(Debug, Clone)]
pub struct Statusbar {
    mode: String,
    terminal_height: usize,
    terminal_width: usize,
}

impl Statusbar {
    pub fn new(mode: String, terminal_height: usize, terminal_width: usize) -> Result<Statusbar> {
        Ok(Statusbar {
            mode,
            terminal_width,
            terminal_height,
        })
    }

    pub fn paint(&self, mode: String, path: String) -> Result<()> {
        let styled = mode.magenta();
        let styled_path = path.blue();
        let content = String::from("analyser");

        SetForegroundColor(Color::Cyan);
        // SetBackgroundColor(Color::White);

        for i in 0..self.terminal_width {
            queue!(
                stdout(),
                cursor::MoveTo(0, (self.terminal_height - 2) as u16),
                SetForegroundColor(Color::White),
                Print(' ')
            )?;
        }

        queue!(
            stdout(),
            cursor::MoveTo(2, (self.terminal_height - 2) as u16),
            Print(styled),
        )?;
        queue!(
            stdout(),
            cursor::MoveTo(10, (self.terminal_height - 2) as u16),
            Print(styled_path),
        )?;
        queue!(
            stdout(),
            cursor::MoveTo(
                (self.terminal_width - content.len()) as u16,
                (self.terminal_height - 2) as u16
            ),
            Print(content.green()),
        )?;

        Ok(())
    }
}
