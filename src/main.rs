extern crate pancurses;
extern crate snm_rand_utils;
extern crate snm_simple_file;
#[macro_use]
extern crate static_assertions;

mod dict;
mod game;
mod solver;
mod utils;

use utils::tui::ascii_keycodes;
use utils::tui::pancurses as pancurses_utils;
use utils::tui::Rect;

#[derive(Debug)]
enum Mode {
    LaunchGui,
    LaunchGame(game::Difficulty),
    LaunchSolver(String, Vec<String>),
}

#[derive(Debug)]
struct CmdlineArgs {
    mode: Mode,
}

fn parse_cmdline_args() -> Result<CmdlineArgs, &'static str> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        return Ok(CmdlineArgs {
            mode: Mode::LaunchGui,
        });
    }

    let mode_arg = &args[0];
    let mode = match mode_arg.as_str() {
        "--solver" => {
            if args.len() < 2 {
                return Err("Missing input file arg for solver mode");
            }

            let known_guess_args = args.iter().skip(2).map(|a| a.clone()).collect();
            Mode::LaunchSolver(args[1].clone(), known_guess_args)
        }
        "--game" => {
            if args.len() < 2 {
                return Err("Missing difficulty arg for game mode");
            }

            let parsed_difficulty = args[1].parse::<game::Difficulty>()?;
            Mode::LaunchGame(parsed_difficulty)
        }
        _ => return Err("Invalid mode argument"),
    };

    Ok(CmdlineArgs { mode })
}

fn print_usage_and_exit(err_msg: &str) -> ! {
    println!("USAGE:");
    println!("    fonv_cracker.exe --solver input_file [guess matching_char_count]+");
    println!("    fonv_cracker.exe --game difficulty");
    println!("Input err: {}", err_msg);
    std::process::exit(1);
}

#[derive(Debug, Clone, Copy)]
enum Screen {
    StartMenu,
    Game(game::Difficulty),
    Solver,
}

fn run_start_menu(window: &pancurses::Window) -> Option<Screen> {
    const TITLE_LINES: [&str; 7] = [
        r#" _      __         __      __             __"#,
        r#"| | /| / /__ ____ / /____ / /__ ____  ___/ /"#,
        r#"| |/ |/ / _ `(_-</ __/ -_) / _ `/ _ \/ _  /"#,
        r#"|__/|__/\_,_/___/\__/\__/_/\_,_/_//_/\_,_/"#,
        r#"      / ___/______ _____/ /_____ ____"#,
        r#"     / /__/ __/ _ `/ __/  '_/ -_) __/"#,
        r#"     \___/_/  \_,_/\__/_/\_\\__/_/"#,
    ];

    let (window_height, window_width) = window.get_max_yx();

    let title_rect = {
        let title_width = TITLE_LINES.iter().map(|line| line.len()).max().unwrap() as i32;
        const TITLE_HEIGHT: i32 = TITLE_LINES.len() as i32;

        Rect {
            // center the title horizontally
            left: (window_width - title_width) / 2,
            // place the title just above the horizontal divide
            top: (window_height / 2) - (TITLE_HEIGHT + 1),
            width: title_width,
            height: TITLE_HEIGHT,
        }
    };

    let mut menu_cursor: usize = 0;
    const MENU_OPTIONS: [&str; 5] = [
        "Start Game (easy)",
        "Start Game (average)",
        "Start Game (hard)",
        "Launch Solver Utility",
        "Quit",
    ];

    const MENU_OPTION_RESULTS: [Option<Screen>; MENU_OPTIONS.len()] = [
        Some(Screen::Game(game::Difficulty::Easy)),
        Some(Screen::Game(game::Difficulty::Average)),
        Some(Screen::Game(game::Difficulty::Hard)),
        Some(Screen::Solver),
        None,
    ];

    let cursor_prefix = "> ";
    let menu_rect = {
        let max_menu_option_width = MENU_OPTIONS
            .iter()
            .map(|option_text| option_text.len())
            .max()
            .unwrap() as i32;
        let menu_width = max_menu_option_width + cursor_prefix.len() as i32;
        const MENU_HEIGHT: i32 = MENU_OPTIONS.len() as i32;

        Rect {
            // center the menu options horizontally
            left: (window_width - menu_width) / 2,
            // place the menu options just below the horizontal divide
            top: (window_height / 2) + 1,
            width: menu_width,
            height: MENU_HEIGHT,
        }
    };

    loop {
        // clear the screen
        window.erase();

        // Render the title card
        for (i, title_line) in TITLE_LINES.iter().enumerate() {
            let row_offset = (i as i32) + title_rect.top;
            let color_pair = pancurses_utils::green();
            window.attron(color_pair);
            window.mvaddstr(row_offset, title_rect.left, title_line);
            window.attroff(color_pair);
        }

        // Render the menu options
        for (i, menu_line) in MENU_OPTIONS.iter().enumerate() {
            let row_offset = (i as i32) + menu_rect.top;
            if i == menu_cursor {
                window.mvaddstr(row_offset, menu_rect.left, cursor_prefix);
            }
            window.mvaddstr(
                row_offset,
                menu_rect.left + cursor_prefix.len() as i32,
                menu_line,
            );
        }

        // Input handling
        // TODO: I think this input system might need some refactoring to share with the start menu
        if let Some(pancurses::Input::Character(ch)) = window.getch() {
            match ch {
                // check for movement inputs
                'w' => {
                    menu_cursor = if menu_cursor == 0 {
                        MENU_OPTIONS.len() - 1
                    } else {
                        menu_cursor - 1
                    }
                }
                's' => {
                    menu_cursor = if menu_cursor == MENU_OPTIONS.len() - 1 {
                        0
                    } else {
                        menu_cursor + 1
                    }
                }
                ascii_keycodes::ENTER => return MENU_OPTION_RESULTS[menu_cursor],
                _ => (),
            }
        };

        // blit the next frame
        window.refresh();
    }
}

const TITLE: &str = "Wasteland Cracker";

fn main() {
    let args = match parse_cmdline_args() {
        Ok(parsed_args) => parsed_args,
        Err(err_msg) => print_usage_and_exit(&err_msg),
    };

    let window = pancurses_utils::setup_window(TITLE);
    match args.mode {
        Mode::LaunchGame(difficulty) => game::run_game(difficulty, &window),
        Mode::LaunchSolver(input_password_file, known_guess_args) => {
            solver::solver(&input_password_file, &known_guess_args, &window)
        }
        Mode::LaunchGui => run_full_gui(&window),
    }
    pancurses::endwin();
}

fn run_full_gui(window: &pancurses::Window) {
    // Run the game until we quit
    let mut screen = Screen::StartMenu;
    loop {
        // Run the current screen until it signals a transition
        let next_screen = match screen {
            Screen::StartMenu => run_start_menu(&window),
            Screen::Game(difficulty) => {
                game::run_game(difficulty, &window);
                Some(Screen::StartMenu)
            }
            Screen::Solver => {
                solver::solver("assets/solver/input.txt", &Vec::new(), &window);
                Some(Screen::StartMenu)
            }
        };

        // If the transition includes a new screen start rendering that.
        screen = match next_screen {
            Some(s) => s,
            None => break,
        }
    }
}
