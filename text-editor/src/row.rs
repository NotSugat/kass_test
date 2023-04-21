use crate::editor_syntax::*;
use crossterm::style::Color;

const TAB_SIZE: usize = 8;

pub struct Row {
    pub chars: String,
    pub render: String,
}

impl Row {
    pub fn new(chars: String) -> Self {
        let mut result = Self {
            chars,
            render: String::new(),
            open_comment: false,
        };

        result.render_row();
        result
    }

    pub fn render_len(&self) -> usize {
        self.render.len()
    }

    pub fn len(&self) -> usize {
        self.chars.len()
    }

    pub fn insert_char(&mut self, at: usize, c: char) {
        if at >= self.chars.len() {
            self.chars.push(c);
        } else {
            self.chars.insert(at, c);
        }
        self.render_row();
    }

    /* returns true if row was modified, false otherwise */
    pub fn del_char(&mut self, at: usize, to_previous_tabstop: bool) -> bool {
        // 123456
        if at >= self.chars.len() {
            false
        } else {
            self.chars.remove(at);

            if to_previous_tabstop && at == self.chars.len() {
                let prev_stop = self.chars.len() - (self.chars.len() % TAB_SIZE);
                while self.chars.ends_with(' ') && self.chars.len() > prev_stop {
                    self.chars.pop();
                }
            }

            self.render_row();
            true
        }
    }

    pub fn split(&mut self, at: usize) -> String {
        let result = self.chars.split_off(at);
        self.render_row();

        result
    }

    pub fn indent_level(&self) -> usize {
        self.chars.len() - self.chars.trim_start_matches(' ').len()
    }

    pub fn append_string(&mut self, s: &str) {
        self.chars.push_str(s);
        self.render_row();
    }

    fn render_row(&mut self) {
        let mut render = String::new();
        let mut idx = 0;
        for c in self.chars.chars() {
            match c {
                '\t' => {
                    render.push(' ');
                    idx += 1;
                    while idx % TAB_SIZE != 0 {
                        render.push(' ');
                        idx += 1;
                    }
                }
                _ => {
                    render.push(c);
                    idx += 1;
                }
            }
        }

        self.render = render;
    }
}
