use super::cli::Cli;
use clap::Parser;
use lazy_static::lazy_static;

pub struct Document {
    pub lines: Vec<Vec<char>>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub dirty: bool,
    history: Vec<HistoryEntry>,
    history_index: usize,
    clean_history_index: Option<usize>,
}

#[derive(Clone, Copy)]
struct Cursor {
    row: usize,
    col: usize,
}

#[derive(Clone)]
struct HistoryEntry {
    edit: Edit,
    before: Cursor,
    after: Cursor,
}

#[derive(Clone)]
enum Edit {
    Insert {
        row: usize,
        col: usize,
        text: Vec<char>,
    },
    Delete {
        row: usize,
        col: usize,
        text: Vec<char>,
    },
    SplitLine {
        row: usize,
        col: usize,
    },
    JoinLine {
        row: usize,
        col: usize,
    },
}

lazy_static! {
    static ref TAB_WIDTH: usize = if let Some(tw) = Cli::parse().tab_width {
        tw
    } else {
        4
    };
}

fn visual_width(c: char, at_vis_col: usize) -> usize {
    if c == '\t' {
        *TAB_WIDTH - (at_vis_col % *TAB_WIDTH)
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
        let lines = if lines.is_empty() {
            vec![Vec::new()]
        } else {
            lines
        };
        Document {
            lines,
            cursor_row: 0,
            cursor_col: 0,
            dirty: false,
            history: Vec::new(),
            history_index: 0,
            clean_history_index: Some(0),
        }
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
                let spaces = *TAB_WIDTH - (vis % *TAB_WIDTH);
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
        let before = self.cursor();
        let after = Cursor {
            row: before.row,
            col: before.col + 1,
        };
        self.apply_new_edit(
            Edit::Insert {
                row: before.row,
                col: before.col,
                text: vec![c],
            },
            before,
            after,
        );
    }

    pub fn insert_tab(&mut self) {
        // Fix #2: insert a real tab character so tabs are preserved on save.
        self.insert_char('\t');
    }

    pub fn insert_newline(&mut self) {
        let before = self.cursor();
        let after = Cursor {
            row: before.row + 1,
            col: 0,
        };
        self.apply_new_edit(
            Edit::SplitLine {
                row: before.row,
                col: before.col,
            },
            before,
            after,
        );
    }

    pub fn backspace(&mut self) {
        if self.cursor_col > 0 {
            let before = self.cursor();
            let after = Cursor {
                row: before.row,
                col: before.col - 1,
            };
            let deleted = self.lines[before.row][after.col];
            self.apply_new_edit(
                Edit::Delete {
                    row: before.row,
                    col: after.col,
                    text: vec![deleted],
                },
                before,
                after,
            );
        } else if self.cursor_row > 0 {
            let before = self.cursor();
            let prev_row = before.row - 1;
            let prev_len = self.lines[prev_row].len();
            let after = Cursor {
                row: prev_row,
                col: prev_len,
            };
            self.apply_new_edit(
                Edit::JoinLine {
                    row: prev_row,
                    col: prev_len,
                },
                before,
                after,
            );
        }
    }

    pub fn delete_forward(&mut self) {
        let before = self.cursor();
        if before.col < self.lines[before.row].len() {
            let deleted = self.lines[before.row][before.col];
            self.apply_new_edit(
                Edit::Delete {
                    row: before.row,
                    col: before.col,
                    text: vec![deleted],
                },
                before,
                before,
            );
        } else if before.row + 1 < self.lines.len() {
            self.apply_new_edit(
                Edit::JoinLine {
                    row: before.row,
                    col: before.col,
                },
                before,
                before,
            );
        }
    }

    pub fn undo(&mut self) -> bool {
        if self.history_index == 0 {
            return false;
        }

        self.history_index -= 1;
        let entry = self.history[self.history_index].clone();
        self.revert_edit(&entry.edit);
        self.set_cursor(entry.before);
        self.update_dirty();
        true
    }

    pub fn redo(&mut self) -> bool {
        if self.history_index == self.history.len() {
            return false;
        }

        let entry = self.history[self.history_index].clone();
        self.apply_edit(&entry.edit);
        self.history_index += 1;
        self.set_cursor(entry.after);
        self.update_dirty();
        true
    }

    pub fn mark_saved(&mut self) {
        self.clean_history_index = Some(self.history_index);
        self.dirty = false;
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

    fn cursor(&self) -> Cursor {
        Cursor {
            row: self.cursor_row,
            col: self.cursor_col,
        }
    }

    fn set_cursor(&mut self, cursor: Cursor) {
        self.cursor_row = cursor.row;
        self.cursor_col = cursor.col;
    }

    fn apply_new_edit(&mut self, edit: Edit, before: Cursor, after: Cursor) {
        self.apply_edit(&edit);
        self.set_cursor(after);
        self.record_edit(edit, before, after);
    }

    fn record_edit(&mut self, edit: Edit, before: Cursor, after: Cursor) {
        if self.history_index < self.history.len() {
            self.history.truncate(self.history_index);
            if self
                .clean_history_index
                .is_some_and(|index| index > self.history_index)
            {
                self.clean_history_index = None;
            }
        }

        self.history.push(HistoryEntry {
            edit,
            before,
            after,
        });
        self.history_index += 1;
        self.update_dirty();
    }

    fn update_dirty(&mut self) {
        self.dirty = self.clean_history_index != Some(self.history_index);
    }

    fn apply_edit(&mut self, edit: &Edit) {
        match edit {
            Edit::Insert { row, col, text } => {
                for (offset, c) in text.iter().enumerate() {
                    self.lines[*row].insert(*col + offset, *c);
                }
            }
            Edit::Delete { row, col, text } => {
                self.lines[*row].drain(*col..*col + text.len());
            }
            Edit::SplitLine { row, col } => {
                let rest = self.lines[*row].split_off(*col);
                self.lines.insert(*row + 1, rest);
            }
            Edit::JoinLine { row, .. } => {
                let next_line = self.lines.remove(*row + 1);
                self.lines[*row].extend(next_line);
            }
        }
    }

    fn revert_edit(&mut self, edit: &Edit) {
        match edit {
            Edit::Insert { row, col, text } => {
                self.lines[*row].drain(*col..*col + text.len());
            }
            Edit::Delete { row, col, text } => {
                for (offset, c) in text.iter().enumerate() {
                    self.lines[*row].insert(*col + offset, *c);
                }
            }
            Edit::SplitLine { row, .. } => {
                let next_line = self.lines.remove(*row + 1);
                self.lines[*row].extend(next_line);
            }
            Edit::JoinLine { row, col } => {
                let rest = self.lines[*row].split_off(*col);
                self.lines.insert(*row + 1, rest);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Document;

    #[test]
    fn undo_and_redo_restore_text_and_cursor() {
        let mut doc = Document::from_text("one");
        doc.cursor_col = 3;
        doc.insert_char('!');
        doc.insert_newline();
        doc.insert_char('x');

        assert_eq!(doc.to_text(), "one!\nx\n");
        assert_eq!((doc.cursor_row, doc.cursor_col), (1, 1));

        assert!(doc.undo());
        assert_eq!(doc.to_text(), "one!\n\n");
        assert_eq!((doc.cursor_row, doc.cursor_col), (1, 0));

        assert!(doc.undo());
        assert_eq!(doc.to_text(), "one!\n");
        assert_eq!((doc.cursor_row, doc.cursor_col), (0, 4));

        assert!(doc.undo());
        assert_eq!(doc.to_text(), "one\n");
        assert_eq!((doc.cursor_row, doc.cursor_col), (0, 3));
        assert!(!doc.undo());

        assert!(doc.redo());
        assert!(doc.redo());
        assert!(doc.redo());
        assert_eq!(doc.to_text(), "one!\nx\n");
        assert_eq!((doc.cursor_row, doc.cursor_col), (1, 1));
        assert!(!doc.redo());
    }

    #[test]
    fn undo_tracks_the_saved_state() {
        let mut doc = Document::from_text("");
        doc.insert_char('a');
        doc.mark_saved();
        assert!(!doc.dirty);

        doc.insert_char('b');
        assert!(doc.dirty);

        assert!(doc.undo());
        assert_eq!(doc.to_text(), "a\n");
        assert!(!doc.dirty);

        assert!(doc.redo());
        assert_eq!(doc.to_text(), "ab\n");
        assert!(doc.dirty);
    }

    #[test]
    fn a_new_edit_discards_redo_history() {
        let mut doc = Document::from_text("");
        doc.insert_char('a');
        doc.insert_char('b');
        assert!(doc.undo());

        doc.insert_tab();
        assert_eq!(doc.to_text(), "a\t\n");
        assert!(!doc.redo());

        assert!(doc.undo());
        assert_eq!(doc.to_text(), "a\n");
    }

    #[test]
    fn undo_and_redo_restore_deleted_text_and_joined_lines() {
        let mut doc = Document::from_text("abc");
        doc.cursor_col = 1;
        doc.delete_forward();
        assert_eq!(doc.to_text(), "ac\n");

        assert!(doc.undo());
        assert_eq!(doc.to_text(), "abc\n");
        assert_eq!((doc.cursor_row, doc.cursor_col), (0, 1));

        assert!(doc.redo());
        assert_eq!(doc.to_text(), "ac\n");

        let mut doc = Document::from_text("ab\ncd");
        doc.cursor_row = 1;
        doc.backspace();
        assert_eq!(doc.to_text(), "abcd\n");
        assert_eq!((doc.cursor_row, doc.cursor_col), (0, 2));

        assert!(doc.undo());
        assert_eq!(doc.to_text(), "ab\ncd\n");
        assert_eq!((doc.cursor_row, doc.cursor_col), (1, 0));

        assert!(doc.redo());
        assert_eq!(doc.to_text(), "abcd\n");
    }
}
