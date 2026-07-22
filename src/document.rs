pub struct Document {
    pub lines: Vec<Vec<char>>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub dirty: bool,
}

fn expand_tabs(line: &str) -> Vec<char> {
    let mut result = Vec::with_capacity(line.len());
    for c in line.chars() {
        if c == '\t' {
            for _ in 0..4 {
                result.push(' ');
            }
        } else {
            result.push(c);
        }
    }
    result
}

impl Document {
    pub fn from_text(text: &str) -> Self {
        let lines: Vec<Vec<char>> = if text.is_empty() {
            vec![Vec::new()]
        } else {
            text.lines().map(|line| expand_tabs(line)).collect()
        };
        let lines = if lines.is_empty() { vec![Vec::new()] } else { lines };
        Document { lines, cursor_row: 0, cursor_col: 0, dirty: false }
    }

    pub fn to_text(&self) -> String {
        self.lines
            .iter()
            .map(|line| line.iter().collect::<String>())
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn insert_char(&mut self, c: char) {
        let row = self.cursor_row;
        let col = self.cursor_col;
        self.lines[row].insert(col, c);
        self.cursor_col += 1;
        self.dirty = true;
    }

    pub fn insert_tab(&mut self) {
        for _ in 0..4 {
            self.insert_char(' ');
        }
    }

    pub fn insert_newline(&mut self) {
        let row = self.cursor_row;
        let col = self.cursor_col;
        let rest = self.lines[row].split_off(col);
        self.lines.insert(row + 1, rest);
        self.cursor_row += 1;
        self.cursor_col = 0;
        self.dirty = true;
    }

    pub fn backspace(&mut self) {
        if self.cursor_col > 0 {
            let row = self.cursor_row;
            self.lines[row].remove(self.cursor_col - 1);
            self.cursor_col -= 1;
            self.dirty = true;
        } else if self.cursor_row > 0 {
            let current_line = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            let prev_len = self.lines[self.cursor_row].len();
            self.lines[self.cursor_row].extend(current_line);
            self.cursor_col = prev_len;
            self.dirty = true;
        }
    }
    
    pub fn delete_forward(&mut self) {
        let row = self.cursor_row;
        if self.cursor_col < self.lines[row].len() {
            self.lines[row].remove(self.cursor_col);
            self.dirty = true;
        } else if row + 1 < self.lines.len() {
            let next_line = self.lines.remove(row + 1);
            self.lines[row].extend(next_line);
            self.dirty = true;
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].len();
        }
    }

    pub fn move_right(&mut self) {
        let line_len = self.lines[self.cursor_row].len();
        if self.cursor_col < line_len {
            self.cursor_col += 1;
        } else if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.clamp_col();
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.clamp_col();
        }
    }

    fn clamp_col(&mut self) {
        let line_len = self.lines[self.cursor_row].len();
        if self.cursor_col > line_len {
            self.cursor_col = line_len;
        }
    }

    pub fn set_cursor_from_screen(&mut self, row: usize, col: usize) {
        let row = row.min(self.lines.len() - 1);
        self.cursor_row = row;
        self.clamp_col_to(col);
    }

    fn clamp_col_to(&mut self, col: usize) {
        let line_len = self.lines[self.cursor_row].len();
        self.cursor_col = col.min(line_len);
    }
}
