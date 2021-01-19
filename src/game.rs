const TITLE: &str = "FONV: Terminal Cracker";

pub fn run_game() {
    // setup the window
    let window = pancurses::initscr();
    pancurses::noecho(); // prevent key inputs rendering to the screen
    pancurses::cbreak();
    pancurses::curs_set(0);
    pancurses::set_title(TITLE);
    window.nodelay(true); // don't block waiting for key inputs (we'll poll)
    window.keypad(true); // let special keys be captured by the program (i.e. esc/backspace/del/arrow keys)

    // TODO just open a stub window for now. We'll write the game soon.
    window.clear();
    window.refresh();
    std::thread::sleep(std::time::Duration::from_millis(1000));
}
