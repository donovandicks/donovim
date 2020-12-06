use crate::Position;
use std::io::{self, stdout, Write};
use termion::color;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

pub struct Size {
    pub width: u16,
    pub height: u16,
}

pub struct Terminal {
    size: Size,
    _stdout: RawTerminal<std::io::Stdout>,
}

impl Terminal {
    /**
     * Initialize a Terminal with Default Settings
     *
     * Defaults:
     *  Heigh and Width are retrieved automatically on invocation
     */
    pub fn default() -> Result<Self, std::io::Error> {
        let size: (u16, u16) = termion::terminal_size()?;
        Ok(Self {
            size: Size {
                width: size.0,
                height: size.1.saturating_sub(2),
            },
            _stdout: stdout().into_raw_mode()?,
        })
    }

    /**
     * Returns a read only reference to internal size to prevent editing
     */
    pub fn size(&self) -> &Size {
        &self.size
    }

    /**
     * Clears the screen
     */
    pub fn clear_screen() {
        // \x1b is the escape character, always followed by [
        // J is the Erase in Display command
        // 2 is an argument for J to clear the entire screen
        // NOTE: 1 clears up to the cursor, 0 clears from the cursor on
        // print!("\x1b[2J");
        // H is the Cursor Position command
        // Column;RowH -> 1-based index
        // print!("\x1b[1;1H");

        // Same as above escape sequence, also moves cursor to top
        print!("{}", termion::clear::All);
    }

    /**
     * Moves the cursor to the given x, y position
     */
    #[allow(clippy::cast_possible_truncation)]
    pub fn cursor_position(position: &Position) {
        let Position { mut x, mut y } = position;
        x = x.saturating_add(1);
        y = y.saturating_add(1);
        let x = x as u16;
        let y = y as u16;
        print!("{}", termion::cursor::Goto(x, y));
    }

    pub fn cursor_hide() {
        print!("{}", termion::cursor::Hide);
    }

    pub fn cursor_show() {
        print!("{}", termion::cursor::Show);
    }

    pub fn clear_current_line() {
        print!("{}", termion::clear::CurrentLine);
    }

    pub fn set_bg_color(color: color::Rgb) {
        print!("{}", color::Bg(color));
    }

    pub fn set_fg_color(color: color::Rgb) {
        print!("{}", color::Fg(color));
    }

    pub fn reset_bg_color() {
        print!("{}", color::Bg(color::Reset));
    }

    pub fn reset_fg_color() {
        print!("{}", color::Fg(color::Reset));
    }

    /**
     * Prints out remaining stdout buffer
     */
    pub fn flush() -> Result<(), io::Error> {
        stdout().flush()
    }

    /**
     * Loop over stdin and return input keys
     */
    pub fn read_key() -> Result<Key, io::Error> {
        loop {
            if let Some(key) = io::stdin().lock().keys().next() {
                return key;
            }
        }
    }
}
