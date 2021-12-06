use crate::{FileType, Position, Row};
use std::fs;
use std::io::{Error, Write};

#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
    pub file_name: Option<String>,
    dirty: bool,
    file_type: FileType,
}

impl Document {
    /// Open a file and store the contents in the `rows` vector
    ///
    /// # Args
    ///
    /// - `filename`: The plain name of the file to open
    ///
    /// # Returns
    ///
    /// - The `Document` if successful
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        let contents = fs::read_to_string(filename)?;
        let file_type = FileType::from(filename);
        let rows = contents.lines().map(Row::from).collect();

        Ok(Self {
            rows,
            file_name: Some(filename.to_string()),
            dirty: false,
            file_type,
        })
    }

    /// Retrieve the file type of the current `Document`
    ///
    /// # Returns
    ///
    /// - The name of the filetype
    pub fn file_type(&self) -> String {
        self.file_type.name()
    }

    /// Write the current `Document` to disk
    ///
    /// # Returns
    ///
    /// - Unit or any Error encountered during the save operation
    pub fn save(&mut self) -> Result<(), Error> {
        if let Some(file_name) = &self.file_name {
            let mut file = fs::File::create(file_name)?;
            self.file_type = FileType::from(file_name);

            for row in &self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
            }
            self.dirty = false;
        }

        Ok(())
    }

    /// Insert a single character into a Document at a given position
    ///
    /// # Args
    ///
    /// - `at`: The (x, y) pair where the character should be placed
    /// - `c`: The character to insert
    pub fn insert(&mut self, at: &Position, c: char) {
        if at.y > self.len() {
            return;
        }

        self.dirty = true;
        if c == '\n' {
            self.insert_newline(at);
        } else if at.y == self.rows.len() {
            let mut row = Row::default();
            row.insert(0, c);
            self.rows.push(row);
        } else {
            let row = self.rows.get_mut(at.y).unwrap();
            row.insert(at.x, c);
        }

        self.unhighlight_rows(at.y);
    }

    /// Adds a line, moving the remainder of a current line down if applicable
    ///
    /// # Args
    ///
    /// - `at`: The (x, y) pair where the new newline should be placed
    fn insert_newline(&mut self, at: &Position) {
        if at.y == self.len() {
            self.rows.push(Row::default());
            return;
        }

        let current_row = &mut self.rows[at.y];
        let new_row = current_row.split(at.x);
        self.rows.insert(at.y + 1, new_row);
    }

    /// Remove the character under the cursor
    ///
    /// # Args
    ///
    /// - `at`: The current cursor position
    pub fn delete(&mut self, at: &Position) {
        let len: usize = self.len();

        if at.y >= len {
            return;
        }

        self.dirty = true;

        if at.x == self.rows.get_mut(at.y).unwrap().len() && at.y < len - 1 {
            let next_row = self.rows.remove(at.y + 1);
            let row = self.rows.get_mut(at.y).unwrap();
            row.append(&next_row);
        } else {
            let row = self.rows.get_mut(at.y).unwrap();
            row.delete(at.x);
        }

        self.unhighlight_rows(at.y);
    }

    /// Search document for query
    ///
    /// # Args
    ///
    /// - `query`: The text to search for
    /// - `after`: The starting location to search from
    ///
    /// # Returns
    ///
    /// - The position of the query if found
    pub fn find(&self, query: &str, after: &Position) -> Option<Position> {
        let mut x = after.x;
        for (y, row) in self.rows.iter().enumerate().skip(after.y) {
            if let Some(x) = row.find(query, x) {
                return Some(Position { x, y });
            }
            x = 0;
        }
        None
    }

    /// Find all matches for a query
    ///
    /// # Args
    ///
    /// - `query`: The text to search for
    ///
    /// # Returns
    ///
    /// - A vector of all positions that match the query
    pub fn find_all(&self, query: &str) -> Vec<Position> {
        // TODO: Refactor
        let mut results = Vec::new();

        for (y, row) in self.rows.iter().enumerate() {
            if let Some(x) = row.find(query, 0) {
                results.push(Position { x, y });
            }
        }

        results
    }

    /// Checks if until is within the bounds of the document
    ///
    /// # Args
    ///
    /// - `until`: The number of rows
    ///
    /// # Returns
    ///
    /// - `until` or the length of the rows on the document
    fn unwrap_until(&mut self, until: usize) -> usize {
        if until <= self.rows.len() {
            until
        } else {
            self.rows.len()
        }
    }

    /// Highlight the document
    ///
    /// # Args
    ///
    /// - `word`:
    /// - `until`: The row to highlight to, if `None` will highlight whole document
    pub fn highlight(&mut self, word: &Option<String>, until: Option<usize>) {
        let mut start_with_comment: bool = false;
        let until = if let Some(until) = until {
            self.unwrap_until(until)
        } else {
            self.rows.len()
        };

        for row in &mut self.rows[..until] {
            start_with_comment = row.highlight(
                self.file_type.highlighting_options(),
                word,
                start_with_comment,
            );
        }
    }

    fn unhighlight_rows(&mut self, start: usize) {
        let start = start.saturating_sub(1);
        for row in self.rows.iter_mut().skip(start) {
            row.is_highlighted = false;
        }
    }

    /// Get the `Row` at the given index
    ///
    /// # Args
    ///
    /// - `index`: The row number to retrieve
    ///
    /// # Returns
    ///
    /// - The row if one exists at `index`
    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    /// Checks if the document is empty (has no rows)
    ///
    /// # Returns
    ///
    /// - Whether the document is empty
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Retrieves the number of rows on the document
    ///
    /// # Returns
    ///
    /// - The number of rows on the document
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Checks if the document is in a `dirty` state, meaning it has been modified
    /// since last save or load
    ///
    /// # Returns
    ///
    /// - Whether the document is dirty
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}
