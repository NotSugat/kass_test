#[derive(Debug, Clone)]
pub struct Row {
    pub chars: String,
    pub render: String,
}

impl Row {
    pub fn new(chars: String) -> Self {
        let mut result = Self {
            chars,
            render: String::new(),
        };

        result.render_row();
        result
    }

    pub fn len(&self) -> usize {
        self.chars.len()
    }

    pub fn append_char(&mut self, position_x: usize, clipboard: String) {
        self.chars.insert_str(position_x, clipboard.as_str());
    }

    pub fn insert_char(&mut self, at: usize, c: char) {
        if at >= self.chars.len() {
            self.chars.push(c);
        } else {
            self.chars.insert(at, c);
        }
        self.render_row();
    }

    fn render_row(&mut self) {
        let mut render = String::new();
        for c in self.chars.chars() {
            render.push(c);
        }

        self.render = render;
    }

    /* returns true if row was modified, false otherwise */
    pub fn del_char(&mut self, idx: usize) -> bool {
        if idx >= self.chars.len() {
            false
        } else {
            self.chars.remove(idx);
            true
        }
    }

    pub fn split(&mut self, at: usize) -> String {
        let result = self.chars.split_off(at);

        result
    }

    pub fn indent_level(&self) -> usize {
        self.chars.len() - self.chars.trim_start_matches(' ').len()
    }

    pub fn append_string(&mut self, s: String) {
        let strContent = s.as_str();
        self.chars.push_str(strContent);
    }
}
