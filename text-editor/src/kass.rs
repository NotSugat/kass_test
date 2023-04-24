use crate::screen;

use super::mode::*;
use super::row::*;
use super::screen::*;
use super::statusbar::*;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    execute,
    style::{Attribute, Print, SetAttribute},
    terminal, QueueableCommand,
};
use std::{
    fs::{read_to_string, OpenOptions},
    io::{stdout, Result, Write},
    path::Path,
};
use text_editor::*;

#[derive(Debug, Clone)]
pub struct Kass {
    current_mode: Mode,
    mode_changed: bool,

    key_event: KeyEvent,
    character: char,

    quit_kass: bool,

    // text: String,
    command: String,

    filepath: String,

    statusbar: Statusbar,
    screen: Screen,
    mode: String,

    rows: Vec<String>,
    rowoff: u16,
    coloff: u16,

    // cursor position
    cursor: Position,

    number_display: bool,

    terminal_width: usize,
    terminal_height: usize,
}

impl Kass {
    pub fn with_file(height: usize, width: usize, filepath: &String) -> Result<Self> {
        let lines = std::fs::read_to_string(filepath)
            .expect("Unable to open file")
            .split('\n')
            .map(|x| x.into())
            .collect::<Vec<String>>();
        Kass::new(&lines, height, width, filepath)
    }

    // constructor
    pub fn new(data: &[String], height: usize, width: usize, filepath: &String) -> Result<Self> {
        let statusbar = Statusbar::new(String::from("Normal"), height, width)?;

        Ok(Kass {
            current_mode: Mode::Normal,
            mode_changed: false,
            key_event: KeyEvent {
                code: KeyCode::Char('f'),
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            },
            character: 'f',
            statusbar,
            screen: Screen::new()?,
            command: String::from(""),
            quit_kass: false,
            filepath: String::from(filepath),

            mode: String::from("Normal"),

            rows: if data.is_empty() {
                Vec::new()
            } else {
                Vec::from(data)
            },
            rowoff: 0,
            coloff: 0,
            cursor: Position::default(),

            number_display: false,
            terminal_height: height,
            terminal_width: width,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        self.statusbar.paint(self.mode.clone())?;
        self.refresh_screen()?;

        loop {
            if let Event::Key(event) = event::read()? {
                // set key_event
                self.key_event = event;

                // set character
                match event.code {
                    KeyCode::Char(c) => self.character = c,
                    _ => {}
                }
                self.screen
                    .move_to(&self.cursor, self.rowoff, self.coloff)?;
                self.handle_modes()?;

                if !self.mode_changed {
                    match self.current_mode {
                        Mode::Insert => {
                            self.handle_insert_mode()?;
                        }
                        Mode::Normal => {
                            self.handle_normal_mode()?;
                        }
                        Mode::Command => {
                            self.handle_command_mode()?;
                        }
                        _ => {}
                    }
                }

                // quit kass
                if self.quit_kass {
                    break;
                }
            }
        }

        Ok(())
    }

    fn handle_modes(&mut self) -> Result<()> {
        match self.current_mode {
            Mode::Normal => match self.key_event {
                // insert mode
                KeyEvent {
                    code: KeyCode::Char('i'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => {
                    self.current_mode = Mode::Insert;
                    self.mode_changed = true;
                    self.mode = "Insert".to_string();
                    self.refresh_screen()?;
                }
                KeyEvent {
                    code: KeyCode::Char('a'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => {
                    self.current_mode = Mode::Insert;
                }

                // visual mode
                KeyEvent {
                    code: KeyCode::Char('v'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => {
                    self.current_mode = Mode::Visual;
                    self.mode = "Visual".to_string();
                    self.refresh_screen()?;
                }

                // command mode
                KeyEvent {
                    code: KeyCode::Char(':'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => {
                    self.current_mode = Mode::Command;
                    self.mode = "Command".to_string();
                    self.refresh_screen()?;
                }
                _ => {
                    self.mode_changed = false;
                }
            },

            Mode::Command => match self.key_event {
                KeyEvent {
                    code: KeyCode::Esc, ..
                } => {
                    self.command = String::from("");
                    // self.refresh_screen()?;
                    self.current_mode = Mode::Normal;
                    self.mode_changed = true;
                    self.mode = "Normal".to_string();
                    self.refresh_screen()?;
                }
                _ => self.mode_changed = false,
            },

            _ => match self.key_event {
                KeyEvent {
                    code: KeyCode::Esc, ..
                } => {
                    self.current_mode = Mode::Normal;
                    self.mode_changed = true;
                    self.mode = "Normal".to_string();
                    self.refresh_screen()?;
                }
                _ => self.mode_changed = false,
            },
        }

        Ok(())
    }

    fn handle_normal_mode(&mut self) -> Result<()> {
        match self.key_event {
            KeyEvent {
                code: KeyCode::Char(key),
                modifiers: KeyModifiers::NONE,
                ..
            } => match key {
                'h' => self.move_cursor(MovementKey::Left),
                'l' => self.move_cursor(MovementKey::Right),
                'j' => self.move_cursor(MovementKey::Down),
                'k' => self.move_cursor(MovementKey::Up),
                _ => {}
            },
            KeyEvent { code, .. } => match code {
                KeyCode::Left => self.move_cursor(MovementKey::Left),
                KeyCode::Right => self.move_cursor(MovementKey::Right),
                KeyCode::Down => self.move_cursor(MovementKey::Down),
                KeyCode::Up => self.move_cursor(MovementKey::Up),

                _ => {}
            },
        }
        Ok(())
    }

    //cursor handler

    fn move_cursor(&mut self, key: MovementKey) {
        match key {
            MovementKey::Left => {
                if self.cursor.x != 0 {
                    self.cursor.x -= 1;
                } else if self.cursor.y > 0 {
                    self.cursor.y -= 1;
                    self.cursor.x = self.rows[self.cursor.y as usize].len() as u16;
                }
            }

            MovementKey::Right => {
                if self.cursor.y < self.rows.len() as u16 {
                    let idx = self.cursor.y;

                    // checks whether cursor exceeds rows length or not
                    if self.cursor.x < self.rows[idx as usize].len() as u16 {
                        self.cursor.x += 1;
                    } else if self.cursor.y < self.rows.len() as u16 {
                        self.cursor.y += 1;
                        self.cursor.x = 0;
                    }
                }
            }

            MovementKey::Up => self.cursor.y = self.cursor.y.saturating_sub(1),
            MovementKey::Down if self.cursor.y < self.rows.len() as u16 => self.cursor.y += 1,
            _ => {}
        }

        // for clamping the cursor to the front of the line after the end of the previous line
        let rowlen = if self.cursor.y as usize >= self.rows.len() {
            0
        } else {
            self.rows[self.cursor.y as usize].len() as u16
        };

        // compare length of the row and cursor x position and gives min value between them
        self.cursor.x = self.cursor.x.min(rowlen);

        self.refresh_screen()
            .expect("not working refresh screen in move cursor function");
    }

    fn scroll(&mut self) -> Result<()> {
        let bounds = self.screen.boundary();

        // for vertical scrolling
        if self.cursor.y < self.rowoff {
            self.rowoff = self.cursor.y;
        }
        if self.cursor.y >= self.rowoff + bounds.y {
            self.rowoff = self.cursor.y - bounds.y + 1;
        }

        // for horizontal scrolling
        if self.cursor.x < self.coloff {
            self.coloff = self.cursor.x;
        }
        if self.cursor.x >= self.coloff + bounds.x {
            self.coloff = self.cursor.x - bounds.x + 1;
        }

        Ok(())
    }

    // handle insert mode
    fn handle_insert_mode(&mut self) -> Result<()> {
        match self.key_event {
            KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                // self.text.pop();
                if self.cursor.x != 0 {
                    self.cursor.x = self.cursor.x.saturating_sub(1);
                } else {
                    self.cursor.y = self.cursor.y.saturating_sub(1);
                }
                self.refresh_screen()?;
            }
            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => {
                // self.text.push('\n');

                self.cursor.y += 1;

                self.refresh_screen()?;
            }
            _ => {
                // print
                if !self.character.is_control() {
                    // self.text.push(self.character);

                    let output = write!(stdout(), "{}", self.character);

                    self.cursor.x += 1;

                    stdout().flush()?;
                    drop(output);
                }
            }
        }

        self.mode_changed = false;

        Ok(())
    }

    fn handle_command_mode(&mut self) -> Result<()> {
        let position_x = 0;
        let position_y = self.terminal_height - 1;

        execute!(
            stdout(),
            cursor::MoveTo(position_x as u16, position_y as u16)
        )?;

        match self.key_event {
            KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                self.command.pop();
                self.refresh_screen()?;
            }

            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => {
                match self.command.as_str() {
                    // quit
                    ":q" => self.quit_kass = true,

                    // write to file
                    ":w" => self.write_to_file()?,

                    // write and quit
                    ":wq" => {
                        self.write_to_file()?;
                        self.quit_kass = true;
                    }
                    ":set nu" => {
                        self.number_display = true;
                    }

                    _ => {}
                }

                self.command = String::from("");
                self.refresh_screen()?;
                self.current_mode = Mode::Normal;
            }

            _ => {
                if !self.character.is_control() {
                    self.command.push(self.character);

                    write!(stdout(), "{}", self.command)?;
                    stdout().flush()?;
                }
            }
        }

        self.mode_changed = false;

        Ok(())
    }

    fn refresh_screen(&mut self) -> Result<()> {
        self.scroll()?;
        execute!(
            stdout(),
            cursor::MoveTo(0, 0),
            terminal::Clear(terminal::ClearType::All),
        )?;

        self.statusbar.paint(self.mode.clone())?;
        self.screen
            .draw_screen(&self.rows, self.rowoff as usize, self.coloff as usize)?;

        // for ch in self.text.as_bytes().iter() {
        //     let character = *ch as char;

        //     if character == '\n' {
        //         write!(stdout(), "{}", "\r\n")?;
        //         stdout().flush()?;
        //     } else {
        //         write!(stdout(), "{}", character)?
        //     }
        // }

        self.screen
            .move_to(&self.cursor, self.rowoff, self.coloff)?;
        stdout().flush()?;

        Ok(())
    }

    // write to file
    fn write_to_file(&self) -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.filepath)?;

        // file.write_all(self.text.as_bytes())?;

        Ok(())
    }
}
