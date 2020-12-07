use std::cmp;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default)]
pub struct Row {
    string: String,
    len: usize,
}

/**
 * Implement From::String for Row
 */
impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        Self {
            string: String::from(slice),
            len: slice.graphemes(true).count(),
        }
    }
}

impl Row {
    /**
     * Return a substring of Row.string
     */
    pub fn render(&self, start: usize, end: usize) -> String {
        let end: usize = cmp::min(end, self.string.len());
        let start: usize = cmp::min(start, end);
        let mut result: String = String::new();
        for grapheme in self.string[..]
            .graphemes(true)
            .skip(start)
            .take(end - start)
        {
            if grapheme == "\t" {
                result.push_str("    ");
            } else {
                result.push_str(grapheme);
            }
        }
        result
    }

    /**
     * Insert the specified char at the specified location in the current row
     */
    pub fn insert(&mut self, at: usize, c: char) {
        if at >= self.len() {
            self.string.push(c);
            self.len += 1;
            return;
        }
        let mut result: String = String::new();
        let mut length: usize = 0;
        for (index, grapheme) in self.string[..].graphemes(true).enumerate() {
            length += 1;
            if index == at {
                length += 1;
                result.push(c);
            }
            result.push_str(grapheme);
        }
        self.len = length;
        self.string = result;
    }

    /**
     * Remove grapheme under the cursor
     */
    pub fn delete(&mut self, at: usize) {
        if at >= self.len() {
            return;
        } 
        let mut result: String = String::new();
        let mut length: usize = 0;
        for (index, grapheme) in self.string[..].graphemes(true).enumerate() {
            if index != at {
                length += 1;
                result.push_str(grapheme);
            }
        }
        self.len = length;
        self.string = result;
    }

    /**
     * Convert row to bytes
     */
    pub fn as_bytes(&self) -> &[u8] {
        self.string.as_bytes()
    }

    /**
     * Split a row at the given column and return the remainder
     */
    pub fn split(&mut self, at: usize) -> Self {
        let mut row: String = String::new();
        let mut length: usize = 0;
        let mut splitted_row: String = String::new();
        let mut splitted_length: usize = 0;
        for (index, grapheme) in self.string[..].graphemes(true).enumerate() {
            if index < at {
                length += 1;
                row.push_str(grapheme);
            } else {
                splitted_length += 1;
                splitted_row.push_str(grapheme);
            }
        }

        self.string = row;
        self.len = length;
        Self {
            string: splitted_row,
            len: splitted_length,
        }
    }

    /**
     * Append a row to the current row
     */
    pub fn append(&mut self, new: &Self) {
        self.string = format!("{}{}", self.string, new.string);
        self.len += new.len;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}
