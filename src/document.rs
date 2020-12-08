use crate::Position;
use crate::Row;
use std::fs;
use std::io::{ Error, Write };

#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
    pub file_name: Option<String>,
    dirty: bool,
}

impl Document {
    /**
     * Open a file and store the contents in rows vector
     */
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        let contents: String = fs::read_to_string(filename)?;
        let mut rows: Vec<Row> = Vec::new();
        for line in contents.lines() {
            rows.push(Row::from(line));
        }
        Ok(Self {
            rows,
            file_name: Some(filename.to_string()),
            dirty: false,
        })
    }

    /**
     * Write document to disk
     */
    pub fn save(&mut self) -> Result<(), Error> {
        if let Some(file_name) = &self.file_name {
            let mut file: fs::File = fs::File::create(file_name)?;
            for row in &self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
            }
            self.dirty = false;
        }
        Ok(())
    }

    /**
     * Insert a single character into a Document at a given position
     */
    pub fn insert(&mut self, at: &Position, c: char) {
        if at.y > self.len() {
            return;
        }
        self.dirty = true;
        if c == '\n' {
            self.insert_newline(at);
            return;
        }
        if at.y == self.len() {
            let mut row: Row = Row::default();
            row.insert(0, c);
            self.rows.push(row);
        } else {
            let row: &mut Row = self.rows.get_mut(at.y).unwrap();
            row.insert(at.x, c);
        }
    }

    /**
     * Adds a line, moving the remainder of a current line down if applicable
     */
    fn insert_newline(&mut self, at: &Position) {
        if at.y == self.len() {
            self.rows.push(Row::default());
            return;
        } 
        let new_row = self.rows.get_mut(at.y).unwrap().split(at.x);
        self.rows.insert(at.y + 1, new_row);
    }

    /**
     * Remove the character under the cursor
     */
    pub fn delete(&mut self, at: &Position) {
        let len: usize = self.len();
        if at.y >= len {
            return;
        }
        self.dirty = true;
        if at.x == self.rows.get_mut(at.y).unwrap().len() && at.y < len - 1 {
            let next_row: Row = self.rows.remove(at.y + 1);
            let row: &mut Row = self.rows.get_mut(at.y).unwrap();
            row.append(&next_row);
        } else {
            let row: &mut Row = self.rows.get_mut(at.y).unwrap();
            row.delete(at.x);
        }
    }

    /**
     * Search document for query
     */
    pub fn find(&self, query: &str, after: &Position) -> Option<Position> {
        let mut x: usize = after.x;
        for (y, row) in self.rows.iter().enumerate().skip(after.y) {
            if let Some(x) = row.find(query, x) {
                return Some(Position { x, y });
            }
            x = 0;
        }
        None
    }

    pub fn find_all(&self, query: &str) -> Vec<Position> {
        let mut results: Vec<Position> = Vec::new();
        for (y, row) in self.rows.iter().enumerate() {
            if let Some(x) = row.find(query, 0) {
                results.push(Position { x, y });
            }
        }
        results
    }

    /**
     * Get the Row at the given index
     */
    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}
