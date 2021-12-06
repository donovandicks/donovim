use crate::Document;
use crate::Row;
use crate::Terminal;
use std::env;
use std::time::{Duration, Instant};
use termion::color;
use termion::event::Key;

const STATUS_FG_COLOR: color::Rgb = color::Rgb(239, 239, 239);
const STATUS_BG_COLOR: color::Rgb = color::Rgb(120, 120, 120);
const VERSION: &str = env!("CARGO_PKG_VERSION");

/**
 * List of Editor Modes
 */
#[derive(PartialEq, Debug)]
enum Mode {
    Normal,
    Insert,
}

/**
 * Holds cursor x and y position for the current document
 */
#[derive(Default, Clone)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

/**
 * Holds message for the current editor status
 */
struct StatusMessage {
    text: String,
    time: Instant,
}

impl StatusMessage {
    /**
     * Implement fromString for StatusMessage
     */
    fn from(message: String) -> Self {
        Self {
            time: Instant::now(),
            text: message,
        }
    }
}

/**
 * Holds values for the current editor instance
 */
pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    offset: Position,
    document: Document,
    status_message: StatusMessage,
    mode: Mode,
    search_results: Vec<Position>,
    highlighted_word: Option<String>,
}

impl Editor {
    /**
     * Initialize an Editor with Default Settings
     */
    pub fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut initial_status = String::from("HELP: :w = Save | :q = Quit | / = Search");

        let document = if args.len() > 1 {
            let file_name = &args[1];
            if let Ok(doc) = Document::open(file_name) {
                doc
            } else {
                initial_status = format!("ERR: Could not open file: {}", file_name);
                Document::default()
            }
        } else {
            Document::default()
        };

        Self {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to initialize terminal"),
            document,
            cursor_position: Position::default(),
            offset: Position::default(),
            status_message: StatusMessage::from(initial_status),
            mode: Mode::Normal,
            search_results: vec![],
            highlighted_word: None,
        }
    }

    /**
     * Run the Editor
     * Loops forever until error or quit signal received.
     * Processes key presses.
     *
     * # Panics
     * - On error when calling refresh_screen
     * - On error when calling process_keypress
     *
     * # Exits
     * - On CTRL-Q keypress
     */
    pub fn run(&mut self) {
        loop {
            if let Err(err) = self.refresh_screen() {
                error(err);
            }
            if let Err(err) = self.process_keypress() {
                error(err);
            }
            if self.should_quit {
                // self.cursor_position = Position { x: 1, y: 1 };
                self.draw_rows();
                Terminal::clear_screen();
                break;
            }
        }
    }

    /**
     * Handle given command from :
     */
    fn process_command(&mut self) {
        let command: Option<String> = self.prompt(":", |_, _, _| {}).unwrap_or(None);
        let command: &str = command.as_ref().map(String::as_ref).unwrap();
        match command {
            "w" => self.save(),
            "q" => {
                if self.document.is_dirty() {
                    self.status_message = StatusMessage::from(
                        "Document has unsaved changes! Add ! to override.".to_string(),
                    );
                    return;
                }
                self.should_quit = true;
            }
            "q!" => self.should_quit = true,
            "wq" => {
                self.save();
                self.should_quit = true;
            }
            _ => {
                self.status_message =
                    StatusMessage::from(format!("Unrecognized Command: {:?}", command))
            }
        }
    }

    /**
     * Handles Keypresses in Normal mode
     */
    fn process_normal_keypress(&mut self, c: char) {
        match c {
            'a' => {
                self.move_cursor(Key::Right);
                self.mode = Mode::Insert;
            }
            'i' => self.mode = Mode::Insert,
            'j' => self.move_cursor(Key::Down),
            'k' => self.move_cursor(Key::Up),
            'h' => self.move_cursor(Key::Left),
            'l' => self.move_cursor(Key::Right),
            'w' => self.move_cursor(Key::Char('w')),
            'o' => {
                self.move_cursor(Key::Down);
                self.document.insert(&self.cursor_position, '\n');
                self.mode = Mode::Insert;
            }
            'n' => {
                if let Some(new_pos) = self
                    .search_results
                    .iter()
                    .find(|&pos| pos.y > self.cursor_position.y)
                {
                    self.cursor_position = new_pos.clone();
                };
            }
            'N' => {
                if let Some(new_pos) = self
                    .search_results
                    .iter()
                    .rfind(|&pos| pos.y < self.cursor_position.y)
                {
                    self.cursor_position = new_pos.clone();
                };
            }
            ':' => self.process_command(),
            '/' => self.search(),
            _ => (),
        }
    }

    /**
     * Handles Keypresses in Insert mode
     */
    fn process_insert_keypress(&mut self, c: char) {
        self.document.insert(&self.cursor_position, c);
        if c == '\n' {
            self.move_cursor(Key::Down);
        } else {
            self.move_cursor(Key::Right);
        }
    }

    /**
     * Reads a key, propogates error if one is returned
     * Sets should_quit if CTRL-Q
     */
    fn process_keypress(&mut self) -> Result<(), std::io::Error> {
        let pressed_key: Key = Terminal::read_key()?;
        match pressed_key {
            Key::Esc => self.mode = Mode::Normal,
            Key::Char(c) => {
                if self.mode == Mode::Insert {
                    self.process_insert_keypress(c);
                } else {
                    self.process_normal_keypress(c);
                }
            }
            Key::Delete => self.document.delete(&self.cursor_position),
            Key::Backspace => {
                if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                    self.move_cursor(Key::Backspace);
                    self.document.delete(&self.cursor_position);
                }
            }
            Key::Up
            | Key::Down
            | Key::Left
            | Key::Right
            | Key::PageUp
            | Key::PageDown
            | Key::End
            | Key::Home => self.move_cursor(pressed_key),
            _ => (),
        }
        self.scroll();
        Ok(())
    }

    /**
     * Save the document. Abort on empty prompt or erorr
     */
    fn save(&mut self) {
        if self.document.file_name.is_none() {
            let new_name: Option<String> = self.prompt("Save as: ", |_, _, _| {}).unwrap_or(None);
            if new_name.is_none() {
                self.status_message = StatusMessage::from("Save aborted.".to_string());
                return;
            }
            self.document.file_name = new_name;
        }

        if self.document.save().is_ok() {
            self.status_message = StatusMessage::from("File saved successfully.".to_string());
        } else {
            self.status_message = StatusMessage::from("Error writing to disk.".to_string());
        }
    }

    /**
     * Prompt the user for an input
     */
    fn prompt<C>(&mut self, prompt: &str, callback: C) -> Result<Option<String>, std::io::Error>
    where
        C: Fn(&mut Self, Key, &String),
    {
        let mut result: String = String::new();
        loop {
            self.status_message = StatusMessage::from(format!("{}{}", prompt, result));
            self.refresh_screen()?;
            let key: Key = Terminal::read_key()?;
            match key {
                Key::Backspace => {
                    if !result.is_empty() {
                        result.truncate(result.len() - 1);
                    }
                }
                Key::Char('\n') => break,
                Key::Char(c) => {
                    if !c.is_control() {
                        result.push(c);
                    }
                }
                Key::Esc => {
                    result.truncate(0);
                    break;
                }
                _ => (),
            }
            callback(self, key, &result);
        }
        self.status_message = StatusMessage::from(String::new());
        if result.is_empty() {
            return Ok(None);
        }
        Ok(Some(result))
    }

    /**
     * Changes the offset to keep up with the cursor position
     */
    fn scroll(&mut self) {
        let Position { x, y } = self.cursor_position;
        let width: usize = self.terminal.size().width as usize;
        let height: usize = self.terminal.size().height as usize;
        let mut offset: &mut Position = &mut self.offset;

        if y < offset.y {
            offset.y = y;
        } else if y >= offset.y.saturating_add(height) {
            offset.y = y.saturating_sub(height).saturating_add(1);
        }

        if x < offset.x {
            offset.x = x;
        } else if x >= offset.x.saturating_add(width) {
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }

    /**
     * Query the document incrementally
     */
    fn search(&mut self) {
        let old_position: Position = self.cursor_position.clone();
        if let Some(query) = self
            .prompt("/", |editor, _, query| {
                if let Some(position) = editor.document.find(query, &editor.cursor_position) {
                    editor.cursor_position = position;
                    editor.scroll();
                }
                editor.highlighted_word = Some(query.to_string());
            })
            .unwrap_or(None)
        {
            if let Some(position) = self.document.find(&query[..], &old_position) {
                self.cursor_position = position;
                self.search_results = self.document.find_all(&query);
            } else {
                self.status_message = StatusMessage::from(format!("Pattern not found: {}", query));
            }
        } else {
            self.cursor_position = old_position;
            self.scroll();
        }
        self.highlighted_word = None;
    }

    /**
     * Moves the cursor based on the given key
     */
    fn move_cursor(&mut self, key: Key) {
        let terminal_height: usize = self.terminal.size().height as usize;
        let Position { mut y, mut x } = self.cursor_position;
        let height: usize = self.document.len();
        let mut width: usize = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };
        match key {
            Key::Char('w') => {
                let row: &Row = self.document.row(y).unwrap();
                let new_idx: usize = row.peek_white(x);
                if new_idx > 0 {
                    x = new_idx;
                } else if let Some(row) = self.document.row(y + 1) {
                    let new_idx: usize = row.peek_alpha(0);
                    x = new_idx;
                    y += 1;
                }
            }
            Key::Up => y = y.saturating_sub(1),
            Key::Down => {
                if y < height {
                    y = y.saturating_add(1)
                }
            }
            Key::Left => x = x.saturating_sub(1),
            Key::Backspace => {
                if x > 0 {
                    x -= 1;
                } else if y > 0 {
                    y -= 1;
                    if let Some(row) = self.document.row(y) {
                        x = row.len();
                    } else {
                        x = 0;
                    }
                }
            }
            Key::Right => {
                if x < width {
                    x = x.saturating_add(1)
                }
            }
            Key::PageUp => {
                y = if y > terminal_height {
                    y - terminal_height
                } else {
                    0
                }
            }
            Key::PageDown => {
                y = if y.saturating_add(terminal_height) < height {
                    y + terminal_height as usize
                } else {
                    height
                }
            }
            Key::Home => x = 0,
            Key::End => x = width,
            _ => (),
        }
        width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };
        if x > width {
            x = width;
        }
        self.cursor_position = Position { x, y }
    }

    /**
     * Clears the screen by writing an escape sequence to the terminal
     */
    fn refresh_screen(&mut self) -> Result<(), std::io::Error> {
        Terminal::cursor_hide();
        Terminal::cursor_position(&Position::default());
        if self.should_quit {
            Terminal::clear_screen();
            println!("Goodbye.\r");
        } else {
            self.document.highlight(
                &self.highlighted_word,
                Some(
                    self.offset
                        .y
                        .saturating_add(self.terminal.size().height as usize),
                ),
            );
            self.draw_rows();
            self.draw_status_bar();
            self.draw_message_bar();
            Terminal::cursor_position(&Position {
                x: self.cursor_position.x.saturating_sub(self.offset.x),
                y: self.cursor_position.y.saturating_sub(self.offset.y),
            });
        }
        Terminal::cursor_show();
        Terminal::flush()
    }

    /**
     * Draw bar for status data
     */
    fn draw_status_bar(&self) {
        let mut status: String;
        let width: usize = self.terminal.size().width as usize;
        let modified_indicator = if self.document.is_dirty() {
            " (modified) "
        } else {
            ""
        };
        let mut file_name: String = "[No Name]".to_string();
        if let Some(name) = &self.document.file_name {
            file_name = name.clone();
            file_name.truncate(20);
        }

        status = format!(
            "{} - {} lines{}- {:?}",
            file_name,
            self.document.len(),
            modified_indicator,
            self.mode,
        );

        let line_indicator: String = format!(
            "{} | {}/{}",
            self.document.file_type(),
            self.cursor_position.y.saturating_add(1),
            self.document.len()
        );
        let len: usize = status.len() + line_indicator.len();
        if width > len {
            status.push_str(&" ".repeat(width - len));
        }
        status = format!("{}{}", status, line_indicator);

        status.truncate(width);
        Terminal::set_bg_color(STATUS_BG_COLOR);
        Terminal::set_fg_color(STATUS_FG_COLOR);
        println!("{}\r", status);
        Terminal::reset_bg_color();
        Terminal::reset_fg_color();
    }

    /**
     * Draw bar for messages
     */
    fn draw_message_bar(&self) {
        Terminal::clear_current_line();
        let message: &StatusMessage = &self.status_message;
        if Instant::now() - message.time < Duration::new(5, 0) {
            let mut text: String = message.text.clone();
            text.truncate(self.terminal.size().width as usize);
            print!("{}", text);
        }
    }
    /**
     * Displays the welcome message in the center of the screen
     */
    fn draw_welcome_message(&self) {
        let mut welcome_msg: String = format!("Hecto editor -- version {}\r", VERSION);
        let width: usize = std::cmp::min(self.terminal.size().width as usize, welcome_msg.len());
        let len: usize = welcome_msg.len();
        let padding: usize = width.saturating_sub(len) / 2;
        let spaces: String = " ".repeat(padding.saturating_sub(1));
        welcome_msg = format!("~{}{}", spaces, welcome_msg);
        welcome_msg.truncate(width);
        println!("{}\r", welcome_msg);
    }

    /**
     * Display the range of lines of the file according to the offset x
     */
    pub fn draw_row(&self, row: &Row) {
        let width: usize = self.terminal.size().width as usize;
        let start: usize = self.offset.x;
        let end: usize = self.offset.x + width;
        let row: String = row.render(start, end);
        println!("{}\r", row)
    }

    /**
     * Display the range of terminal rows according to offset y
     */
    fn draw_rows(&self) {
        Terminal::set_bg_color(color::Rgb(29, 32, 33));
        let height: u16 = self.terminal.size().height;
        for terminal_row in 0..height {
            Terminal::clear_current_line();
            if let Some(row) = self.document.row(terminal_row as usize + self.offset.y) {
                self.draw_row(row);
            } else if self.document.is_empty() && terminal_row == height / 3 {
                self.draw_welcome_message();
            } else {
                println!("~\r");
            }
        }
        Terminal::reset_bg_color();
    }
}

/**
 * Custom panic wrapper
 */
fn error(e: std::io::Error) -> ! {
    Terminal::clear_screen();
    panic!("{}", e);
}
