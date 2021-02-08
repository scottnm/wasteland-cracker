pub fn matching_char_count_ignore_case(a: &str, b: &str) -> usize {
    assert_eq!(a.len(), b.len());

    fn chars_eq_ignore_case((a, b): &(char, char)) -> bool {
        a.to_ascii_lowercase() == b.to_ascii_lowercase()
    }

    a.chars()
        .zip(b.chars())
        .filter(chars_eq_ignore_case)
        .count()
}

pub fn hamming_dist_ignore_case(a: &str, b: &str) -> usize {
    assert_eq!(a.len(), b.len());
    a.len() - matching_char_count_ignore_case(a, b)
}

pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
}

const GREEN_IDX: u8 = 1;
pub fn pancurses_green() -> pancurses::chtype {
    pancurses::COLOR_PAIR(GREEN_IDX as pancurses::chtype)
}

pub fn setup_pancurses_window(title: &str) -> pancurses::Window {
    let window = pancurses::initscr();
    pancurses::start_color();
    pancurses::init_pair(
        GREEN_IDX as i16,
        pancurses::COLOR_GREEN,
        pancurses::COLOR_BLACK,
    );
    pancurses::noecho(); // prevent key inputs rendering to the screen
    pancurses::cbreak();
    pancurses::curs_set(0);
    pancurses::set_title(title);
    window.nodelay(true); // don't block waiting for key inputs (we'll poll)
    window.keypad(true); // let special keys be captured by the program (i.e. esc/backspace/del/arrow keys)
    window
}

pub mod keys {
    pub const ASCII_ESC: char = 27 as char;
    pub const _ASCII_BACKSPACE: char = 8 as char;
    pub const _ASCII_DEL: char = 127 as char;
    pub const ASCII_ENTER: char = 10 as char;
}
