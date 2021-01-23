extern crate pancurses;
extern crate rand;
extern crate snm_simple_file;

mod dict;
mod game;
mod randwrapper;
mod solver;
mod utils;

#[derive(Debug)]
enum Mode {
    Game(game::Difficulty),
    Solver(String, Vec<String>),
}

#[derive(Debug)]
struct CmdlineArgs {
    mode: Mode,
}

fn parse_cmdline_args() -> Result<CmdlineArgs, &'static str> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        return Err("Missing mode argument");
    }

    let mode_arg = &args[0];
    let mode = match mode_arg.as_str() {
        "--solver" => {
            if args.len() < 2 {
                return Err("Missing input file arg for solver mode");
            }

            let known_guess_args = args.iter().skip(2).map(|a| a.clone()).collect();
            Mode::Solver(args[1].clone(), known_guess_args)
        }
        "--game" => {
            if args.len() < 2 {
                return Err("Missing difficulty arg for game mode");
            }

            let parsed_difficulty = args[1].parse::<game::Difficulty>()?;
            Mode::Game(parsed_difficulty)
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

fn main() {
    let args = match parse_cmdline_args() {
        Ok(parsed_args) => parsed_args,
        Err(err_msg) => print_usage_and_exit(&err_msg),
    };

    match args.mode {
        Mode::Game(difficulty) => game::run_game(difficulty),
        Mode::Solver(input_password_file, known_guess_args) => {
            solver::solver(&input_password_file, &known_guess_args)
        }
    }
}
