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

// Work breakdown
// - generate a set of words given a difficulty/length
// - constrain the number of words generated to only as much as would fit in two panes
// - select a word to be the solution word
// - run a window-less gameloop which lets us input words and get back the results of "N matching chars to solution"
// - setup the win-lose condition that you only have 4 guesses
// - render two panes
// - place the words throughout the pane w/ filler text that goes in between words
// - add support for selecting between the words in the TUI and highlighting the current selection
//      - mouse support?
//      - keyboard support?
// - add support for using that selection instead of text input to power the gameloop
// - add an output pane which tells you the results of your current selection

// extensions/flavor
// - use appropriate font to give it a "fallout feel"
// - SFX
