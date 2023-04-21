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
use text_editor::Position;

use super::mode::*;
use super::statusbar::*;

#[derive(Debug, Clone)]
pub struct Kass {
    current_mode: Mode,
    mode_changed: bool,

    key_event: KeyEvent,
    character: char,

    quit_kass: bool,

    text: String,
    command: String,

    filepath: String,

    statusbar: Statusbar,

    position_x: usize,
    position_y: usize,

    // cursor position
    cursor: Position,

    number_display: bool,

    terminal_width: usize,
    terminal_height: usize,
}

impl Kass {
    // constructor
    pub fn new(height: usize, width: usize, filepath: &String) -> Result<Kass> {
        let mut text = String::new();

        if Path::new(&filepath).exists() {
            text = read_to_string(filepath)?;
        }

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
            text,
            statusbar,
            command: String::from(""),
            quit_kass: false,
            filepath: String::from(filepath),
            position_x: 0,
            position_y: 0,

            cursor: Position::new(3, 0)?,

            number_display: false,
            terminal_height: height,
            terminal_width: width,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        self.statusbar.paint()?;
        if self.number_display {
            self.refresh_screen(5, 0, &self.text)?;
        } else {
            self.refresh_screen(0, 0, &self.text)?;
        }

        loop {
            if let Event::Key(event) = event::read()? {
                // set key_event
                self.key_event = event;

                // set character
                match event.code {
                    KeyCode::Char(c) => self.character = c,
                    _ => {}
                }

                self.handle_modes()?;

                if !self.mode_changed {
                    match self.current_mode {
                        Mode::Insert => self.handle_insert_mode()?,
                        Mode::Command => self.handle_command_mode()?,
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
                }
                KeyEvent {
                    code: KeyCode::Char('a'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => {
                    self.current_mode = Mode::Insert;
                    self.mode_changed = true;
                }

                // visual mode
                KeyEvent {
                    code: KeyCode::Char('v'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => self.current_mode = Mode::Visual,

                // command mode
                KeyEvent {
                    code: KeyCode::Char(':'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => self.current_mode = Mode::Command,
                _ => {
                    self.mode_changed = false;
                }
            },

            Mode::Command => match self.key_event {
                KeyEvent {
                    code: KeyCode::Esc, ..
                } => {
                    self.command = String::from("");
                    self.refresh_screen(0, 0, &self.text)?;
                    self.current_mode = Mode::Normal;
                }
                _ => self.mode_changed = false,
            },

            _ => match self.key_event {
                KeyEvent {
                    code: KeyCode::Esc, ..
                } => self.current_mode = Mode::Normal,
                _ => self.mode_changed = false,
            },
        }

        Ok(())
    }

    fn handle_insert_mode(&mut self) -> Result<()> {
        match self.key_event {
            KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                self.text.pop();

                self.refresh_screen(self.position_x, 0, &self.text)?;
            }
            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => {
                self.text.push('\n');

                self.cursor.y += 1;
                if self.cursor.x > 3 {
                    self.cursor.x -= 1;
                }
                self.position_y += 1;
                execute!(
                    stdout(),
                    cursor::MoveTo(self.position_x as u16, self.position_y as u16)
                )?;
            }
            _ => {
                // print
                if !self.character.is_control() {
                    self.text.push(self.character);

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
                self.refresh_screen(position_x, position_y, &self.command)?;
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
                        self.position_x = 4;
                    }

                    _ => {}
                }

                self.command = String::from("");
                self.refresh_screen(0, 0, &self.text)?;
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

    fn refresh_screen(&self, width: usize, height: usize, text: &String) -> Result<()> {
        self.statusbar.paint()?;
        execute!(
            stdout(),
            cursor::MoveTo(width as u16, height as u16),
            terminal::Clear(terminal::ClearType::All),
        )?;

        for ch in text.as_bytes().iter() {
            let character = *ch as char;

            if character == '\n' {
                write!(stdout(), "{}", "\r\n")?;
                stdout().flush()?;
            } else {
                write!(stdout(), "{}", character)?
            }
        }
        if self.number_display {
            self.line_number_display()?;
        }
        self.handle_cursor()?;
        stdout().flush()?;

        Ok(())
    }

    fn handle_cursor(&self) -> Result<()> {
        execute!(stdout(), cursor::MoveTo(self.cursor.x, self.cursor.y))?;
        Ok(())
    }

    fn line_number_display(&self) -> Result<()> {
        for i in 0..self.terminal_height - 1 {
            stdout()
                .queue(SetAttribute(Attribute::Reset))?
                .queue(cursor::MoveTo(0, i as u16))?
                .queue(Print(format!("{:3} ", i + 1)))?;
        }

        execute!(stdout(), cursor::MoveTo(self.position_x as u16, 0))?;
        stdout().flush()?;
        Ok(())
    }

    // write to file
    fn write_to_file(&self) -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.filepath)?;

        file.write_all(self.text.as_bytes())?;

        Ok(())
    }
}
