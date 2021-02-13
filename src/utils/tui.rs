pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
}

pub mod pancurses {
    const GREEN_IDX: u8 = 1;

    pub fn green() -> pancurses::chtype {
        pancurses::COLOR_PAIR(GREEN_IDX as pancurses::chtype)
    }

    pub fn setup_window(title: &str) -> pancurses::Window {
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
}

pub mod ascii_keycodes {
    pub const ESC: char = 27 as char;
    pub const BKSP: char = 8 as char;
    pub const DEL: char = 127 as char;
    pub const ENTER: char = 10 as char;
}
