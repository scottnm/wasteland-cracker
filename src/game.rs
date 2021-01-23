// Work breakdown
// - constrain the number of words generated to only as much as would fit in two panes
// - setup a better word selection algorithm which results in more common letters
// - render two panes
// - place the words throughout the pane w/ filler text that goes in between words
// - add support for selecting between the words in the TUI and highlighting the current selection
//      - mouse support?
//      - keyboard support?
// - add support for using that selection instead of text input to power the gameloop
// - add an output pane which tells you the results of your current selection
// - refactor out tui utils into its own module
// - randomize the hex start address for fun flavor

// extensions/flavor
// - use appropriate font to give it a "fallout feel"
// - SFX

use crate::dict;
use crate::randwrapper::{select_rand, RangeRng, ThreadRangeRng};
use crate::utils::Rect;
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

struct HexDumpPane {
    dump_width: i32,
    dump_height: i32,
    addr_width: i32,
    addr_to_dump_padding: i32,
}

impl HexDumpPane {
    fn height(&self) -> i32 {
        self.dump_height
    }

    fn full_width(&self) -> i32 {
        self.dump_width + self.addr_to_dump_padding + self.addr_width
    }
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

fn generate_words(difficulty: Difficulty, rng: &mut dyn RangeRng<usize>) -> Vec<String> {
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

fn render_hexdump_pane(
    window: &pancurses::Window,
    hex_dump_dimensions: &HexDumpPane,
    render_rect: &Rect,
    mem_start: usize,
    bytes: &[char],
) {
    unimplemented!();
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

        const HEXDUMP_ROW_WIDTH: i32 = 12; // 12 characters per row of the hexdump
        const HEXDUMP_MAX_ROWS: i32 = 16; // 16 rows of hex dump per dump pane
        const HEXDUMP_MAX_BYTES: i32 = HEXDUMP_ROW_WIDTH * HEXDUMP_MAX_ROWS;
        const MEMADDR_HEX_WIDTH: i32 = "0x1234".len() as i32; // 2 byte memaddr
        const PANE_HORIZONTAL_PADDING: i32 = 4; // horizontal padding between panes in the memdump window

        const HEXDUMP_PANE_WIDTH: i32 = MEMADDR_HEX_WIDTH
            + PANE_HORIZONTAL_PADDING
            + HEXDUMP_ROW_WIDTH
            + (PANE_HORIZONTAL_PADDING / 2);

        const HEXDUMP_PANE_VERT_OFFSET: i32 = 5;

        let (window_height, window_width) = window.get_max_yx();
        let (window_center_x, window_center_y) = (window_width / 2, window_height / 2);

        // are all of these constants only used by this struct?
        let hex_dump_pane_dimensions = HexDumpPane {
            dump_width: HEXDUMP_ROW_WIDTH,
            dump_height: HEXDUMP_MAX_ROWS,
            addr_width: MEMADDR_HEX_WIDTH,
            addr_to_dump_padding: PANE_HORIZONTAL_PADDING,
        };

        let left_hex_dump_rect = Rect {
            left: window_center_x
                - hex_dump_pane_dimensions.full_width()
                - (PANE_HORIZONTAL_PADDING / 2),
            top: HEXDUMP_PANE_VERT_OFFSET,
            width: hex_dump_pane_dimensions.full_width(),
            height: hex_dump_pane_dimensions.height(),
        };

        let right_hex_dump_rect = Rect {
            left: left_hex_dump_rect.left
                + left_hex_dump_rect.width
                + (PANE_HORIZONTAL_PADDING / 2),
            top: left_hex_dump_rect.top,
            width: hex_dump_pane_dimensions.full_width(),
            height: hex_dump_pane_dimensions.height(),
        };

        let hexdump_temp_fill = vec!['x'; HEXDUMP_MAX_BYTES as usize];
        let hexdump_first_addr = 0x1234; // TODO: randomize for fun flavor
        render_hexdump_pane(
            &window,
            &hex_dump_pane_dimensions,
            &left_hex_dump_rect,
            hexdump_first_addr,
            &hexdump_temp_fill,
        );

        render_hexdump_pane(
            &window,
            &hex_dump_pane_dimensions,
            &right_hex_dump_rect,
            hexdump_first_addr + hexdump_temp_fill.len(),
            &hexdump_temp_fill,
        );

        /* TODO: render the words in the memdump
        window.mvaddstr(0, 0, format!("{:?}", difficulty));
        for (i, rand_word) in rand_words.iter().enumerate() {
            window.mvaddstr(i as i32 + 1, 0, rand_word);
        }
        */

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
