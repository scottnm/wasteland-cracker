// Work breakdown
// - constrain the number of words generated to only as much as would fit in two panes
// - setup a better word selection algorithm which results in more common letters
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

use crate::dict;
use crate::randwrapper::{select_rand, RangeRng, ThreadRangeRng};
use std::str::FromStr;

const TITLE: &str = "FONV: Terminal Cracker";

#[derive(Debug, Clone, Copy)]
pub enum Difficulty {
    VeryEasy,
    Easy,
    Average,
    Hard,
    VeryHard,
}

impl FromStr for Difficulty {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("VeryEasy") || s.eq_ignore_ascii_case("VE") {
            return Ok(Difficulty::VeryEasy);
        }

        if s.eq_ignore_ascii_case("Easy") || s.eq_ignore_ascii_case("E") {
            return Ok(Difficulty::Easy);
        }

        if s.eq_ignore_ascii_case("Average") || s.eq_ignore_ascii_case("A") {
            return Ok(Difficulty::Average);
        }

        if s.eq_ignore_ascii_case("Hard") || s.eq_ignore_ascii_case("H") {
            return Ok(Difficulty::Hard);
        }

        if s.eq_ignore_ascii_case("VeryHard") || s.eq_ignore_ascii_case("VH") {
            return Ok(Difficulty::VeryHard);
        }

        Err("Invalid difficulty string")
    }
}

pub fn generate_words(difficulty: Difficulty, rng: &mut dyn RangeRng<usize>) -> Vec<String> {
    let word_len = match difficulty {
        Difficulty::VeryEasy => 4,
        Difficulty::Easy => 6,
        Difficulty::Average => 8,
        Difficulty::Hard => 10,
        Difficulty::VeryHard => 12,
    };

    const WORDS_TO_GENERATE_COUNT: usize = 16;

    let dict_chunk = dict::EnglishDictChunk::load(word_len);
    (0..WORDS_TO_GENERATE_COUNT)
        .map(|_| dict_chunk.get_random_word(rng))
        .collect()
}

pub fn run_game(difficulty: Difficulty) {
    // Generate a random set of words based on the difficulty
    let mut rng = ThreadRangeRng::new();
    let rand_words = generate_words(difficulty, &mut rng);

    // For the sake of keeping the windowing around let's dump those words in a window
    {
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

        window.mvaddstr(0, 0, format!("{:?}", difficulty));
        for (i, rand_word) in rand_words.iter().enumerate() {
            window.mvaddstr(i as i32 + 1, 0, rand_word);
        }

        window.refresh();
        std::thread::sleep(std::time::Duration::from_millis(3000));
        pancurses::endwin();
    }

    // now let's run a mock game_loop
    run_game_from_line_console(&rand_words, &mut rng);
}

fn run_game_from_line_console(words: &[String], rng: &mut dyn RangeRng<usize>) {
    // Select an answer
    let solution = select_rand(words, rng);

    println!("Solution: {}", solution);
    for word in words {
        let matching_char_count = crate::utils::matching_char_count_ignore_case(&solution, word);
        println!("  {} ({}/{})", word, matching_char_count, solution.len());
    }

    // On each game loop iteration...
    let mut remaining_guess_count = 4;
    while remaining_guess_count > 0 {
        // Let the user provide a guess
        println!("\nGuess? ");
        let next_guess: String = text_io::read!("{}");

        // Check for a win
        if &next_guess == solution {
            break;
        }

        // Validate the non-winning word in the guess list
        // TODO: won't be necessary when they can only select from a preset set of words
        if !words.iter().any(|w| w.eq_ignore_ascii_case(&next_guess)) {
            println!("Not a word in the list!");
            continue;
        }

        // Print the matching character count as a hint for the next guess
        let matching_char_count =
            crate::utils::matching_char_count_ignore_case(&solution, &next_guess);

        println!("{} / {} chars match!", matching_char_count, solution.len());

        // let the user know how many attempts they have left
        remaining_guess_count -= 1;
        println!("{} attempts left", remaining_guess_count);
    }

    if remaining_guess_count > 0 {
        println!("Correct!");
    } else {
        println!("Failed!");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::randwrapper;

    #[test]
    fn test_word_generation() {
        let mut rng = randwrapper::mocks::SequenceRangeRng::new(&[0, 2, 4, 7]);
        let tests = [
            (Difficulty::VeryEasy, ["aahs", "aani", "abac", "abba"]),
            (Difficulty::Easy, ["aahing", "aarrgh", "abacay", "abacot"]),
            (
                Difficulty::Average,
                ["aardvark", "aaronite", "abacisci", "abacuses"],
            ),
            (
                Difficulty::Hard,
                ["aardwolves", "abalienate", "abandoning", "abaptistum"],
            ),
            (
                Difficulty::VeryHard,
                [
                    "abalienating",
                    "abandonments",
                    "abbreviately",
                    "abbreviatory",
                ],
            ),
        ];

        for (difficulty, expected_words) in &tests {
            let generated_words = generate_words(*difficulty, &mut rng);
            let expected_word_cnt = 16;
            for i in 0..expected_word_cnt {
                let generated_word = &generated_words[i];
                let expected_word = expected_words[i % expected_words.len()];
                assert_eq!(generated_word, expected_word);
            }
        }
    }
}
