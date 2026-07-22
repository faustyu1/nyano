pub struct Document {
    pub lines: Vec<Vec<char>>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub dirty: bool,
}

const TAB_WIDTH: usize = 4;

fn visual_width(c: char, at_vis_col: usize) -> usize {
    if c == '\t' {
        TAB_WIDTH - (at_vis_col % TAB_WIDTH)
    } else {
        1
    }
}

impl Document {
    pub fn from_text(text: &str) -> Self {
        // Fix #2: store tabs as-is instead of expanding them, so they are
        // preserved on save rather than silently replaced with spaces.
        let lines: Vec<Vec<char>> = if text.is_empty() {
            vec![Vec::new()]
        } else {
            text.lines().map(|line| line.chars().collect()).collect()
        };
        let lines = if lines.is_empty() { vec![Vec::new()] } else { lines };
        Document { lines, cursor_row: 0, cursor_col: 0, dirty: false }
    }

    pub fn to_text(&self) -> String {
        // Fix #3: append a trailing newline to produce a well-formed POSIX
        // text file and avoid stripping the newline that was already there.
        let mut s = self
            .lines
            .iter()
            .map(|line| line.iter().collect::<String>())
            .collect::<Vec<String>>()
            .join("\n");
        s.push('\n');
        s
    }

    /// Expand tabs to spaces for on-screen rendering.
    /// Used by the UI layer; the underlying char vec is not modified.
    pub fn line_to_visual(line: &[char]) -> String {
        let mut result = String::new();
        let mut vis = 0usize;
        for &c in line {
            if c == '\t' {
                let spaces = TAB_WIDTH - (vis % TAB_WIDTH);
                for _ in 0..spaces {
                    result.push(' ');
                }
                vis += spaces;
            } else {
                result.push(c);
                vis += 1;
            }
        }
        result
    }

    /// Visual screen column of the cursor (tabs count as multiple columns).
    pub fn visual_cursor_col(&self) -> usize {
        let row = self.cursor_row;
        let col = self.cursor_col;
        let mut vis = 0usize;
        for (i, &c) in self.lines[row].iter().enumerate() {
            if i == col {
                break;
            }
            vis += visual_width(c, vis);
        }
        vis
    }

    /// Convert a visual screen column back to a char index in the given row.
    fn char_col_from_visual(&self, row: usize, target_vis: usize) -> usize {
        let line = &self.lines[row];
        let mut vis = 0usize;
        for (i, &c) in line.iter().enumerate() {
            if vis >= target_vis {
                return i;
            }
            vis += visual_width(c, vis);
        }
        line.len()
    }

    pub fn insert_char(&mut self, c: char) {
        let row = self.cursor_row;
        let col = self.cursor_col;
        self.lines[row].insert(col, c);
        self.cursor_col += 1;
        self.dirty = true;
    }

    pub fn insert_tab(&mut self) {
        // Fix #2: insert a real tab character so tabs are preserved on save.
        self.insert_char('\t');
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
        // Fix #2: translate visual column to char index so mouse clicks land
        // on the correct character even when tabs are present.
        self.cursor_col = self.char_col_from_visual(row, col);
    }
}
