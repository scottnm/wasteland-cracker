// Work breakdown
// - setup a better word selection algorithm which results in more common letters
// - add support for selecting between the words in the TUI and highlighting the current selection
//      - mouse support?
//      - keyboard support?
// - add support for using that selection instead of text input to power the gameloop
// - add an output pane which tells you the results of your current selection
// - refactor out tui utils into its own module

// extensions/flavor
// - use appropriate font to give it a "fallout feel"
// - SFX

use crate::dict;
use crate::randwrapper::{select_rand, RangeRng, ThreadRangeRng};
use crate::utils::Rect;

const TITLE: &str = "FONV: Terminal Cracker";

#[derive(Debug, Clone, Copy)]
pub enum Difficulty {
    VeryEasy,
    Easy,
    Average,
    Hard,
    VeryHard,
}

impl std::str::FromStr for Difficulty {
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

enum Movement {
    Left,
    Right,
    Up,
    Down,
}

//#[derive(PartialEq, Eq)]
enum InputCmd {
    Move(Movement),
    Select,
    Quit,
}

struct HexDumpPane {
    dump_width: i32,
    dump_height: i32,
    addr_width: i32,
    addr_to_dump_padding: i32,
}

impl HexDumpPane {
    const fn width(&self) -> i32 {
        self.dump_width
    }

    const fn height(&self) -> i32 {
        self.dump_height
    }

    const fn max_bytes_in_pane(&self) -> usize {
        (self.dump_width * self.dump_height) as usize
    }

    const fn full_width(&self) -> i32 {
        self.dump_width + self.addr_to_dump_padding + self.addr_width
    }

    const fn addr_width(&self) -> usize {
        self.addr_width as usize
    }

    const fn padding(&self) -> i32 {
        self.addr_to_dump_padding
    }
}

// TODO: this chunk selection logic is pretty ugly. Can it be refactored for readability?
struct SelectedChunk {
    pane_num: usize,
    row_num: usize,
    col_start: usize,
    len: usize,
}

fn generate_words(difficulty: Difficulty, rng: &mut dyn RangeRng<usize>) -> Vec<String> {
    let word_len = match difficulty {
        Difficulty::VeryEasy => 4,
        Difficulty::Easy => 6,
        Difficulty::Average => 8,
        Difficulty::Hard => 10,
        Difficulty::VeryHard => 12,
    };

    const WORDS_TO_GENERATE_COUNT: usize = 12;

    let dict_chunk = dict::EnglishDictChunk::load(word_len);
    (0..WORDS_TO_GENERATE_COUNT)
        .map(|_| dict_chunk.get_random_word(rng))
        .collect()
}

fn move_selection(
    selection: SelectedChunk,
    movement: Movement,
    hex_dump_pane_dimensions: &HexDumpPane,
    num_panes: usize,
) -> SelectedChunk {
    let (col_move, row_move): (i32, i32) = match movement {
        Movement::Down => (0, 1),
        Movement::Up => (0, -1),
        Movement::Left => (-1, 0),
        // We might be at the beginning of a full word, in which case move past the end of the word
        Movement::Right => (selection.len as i32, 0),
    };

    // Naively update the row and col with our movement
    let mut next_col = col_move + (selection.col_start as i32);
    let mut next_row = row_move + (selection.row_num as i32);
    let mut next_pane = selection.pane_num as i32;

    // Check if we've moved from one pane to another by moving laterally across the columns in a row.
    if next_col >= hex_dump_pane_dimensions.width() {
        next_col = 0;
        next_pane += 1;
    } else if next_col < 0 {
        next_col = hex_dump_pane_dimensions.width() - 1;
        next_pane -= 1;
    }

    // Check if we've moved to an invalid row outside of our pane.
    // In which case just wrap around to the next valid row in the same pane.
    if next_row >= hex_dump_pane_dimensions.height() {
        next_row = 0;
    } else if next_row < 0 {
        next_row = hex_dump_pane_dimensions.height() - 1;
    }

    // Check if we've moved to an invalid pane outside of our hex dump.
    // In which case just wrap around to the next valid pane.
    if next_pane >= num_panes as i32 {
        next_pane = 0;
    } else if next_pane < 0 {
        next_pane = 1;
    }

    SelectedChunk {
        pane_num: next_pane as usize,
        row_num: next_row as usize,
        col_start: next_col as usize,
        len: 1,
    }
}

fn refit_selection(
    selection: SelectedChunk,
    words: &[String],
    word_offsets: &[usize],
    hex_dump_pane_dimensions: &HexDumpPane,
) -> SelectedChunk {
    let cursor_index = selection.pane_num * hex_dump_pane_dimensions.max_bytes_in_pane()
        + selection.row_num * hex_dump_pane_dimensions.width() as usize
        + selection.col_start;

    // turn our list of words and word_offsets into a list of ranges where those words live
    // in the contiguous hex dump memory span
    let word_ranges = words
        .iter()
        .zip(word_offsets.iter())
        .map(|(word, word_offset)| (*word_offset, word_offset + word.len()));

    // TODO: clean up this implementation. It's a bit ugly
    let mut result_selection = selection;

    for word_range in word_ranges {
        if cursor_index >= word_range.0 && cursor_index < word_range.1 {
            // if our cursor is on or in the middle of a full word, update the cursor selection
            // to highlight the whole word
            let cursor_offset = cursor_index - word_range.0;
            if cursor_offset <= result_selection.col_start {
                result_selection.col_start -= cursor_offset;
            } else {
                // account for the word starting on the previous row
                // account for the previous row, being in the previous pane
                if result_selection.row_num > 0 {
                    result_selection.row_num -= 1;
                } else {
                    result_selection.row_num = (hex_dump_pane_dimensions.height() - 1) as usize;
                    result_selection.pane_num -= 1;
                }
                result_selection.col_start +=
                    hex_dump_pane_dimensions.width() as usize - cursor_offset;
            }
            result_selection.len = word_range.1 - word_range.0;
            break;
        }
    }

    result_selection
}

// TODO: this chunk render function is pretty nasty. can it be refactored better readability
// one idea is to split out the rendering of memory addresses from rendering out the actual hex dumps
//   this could help greatly clean up some offset calculations...
fn render_hexdump_pane(
    window: &pancurses::Window,
    hex_dump_dimensions: &HexDumpPane,
    render_rect: &Rect,
    hex_dump_first_byte: usize,
    bytes: &str,
    pane_offset: usize,
    (highlighted_byte_start, highlighted_byte_end): (usize, usize),
) {
    for row in 0..hex_dump_dimensions.height() {
        let row_first_byte = pane_offset + (row * hex_dump_dimensions.width()) as usize;
        let mem_addr = format!(
            "0x{:0width$X}",
            hex_dump_first_byte + row_first_byte,
            width = hex_dump_dimensions.addr_width() - 2,
        );

        let y = row + render_rect.top;

        // render the memaddr
        window.mvaddstr(y, render_rect.left, &mem_addr);

        let begin_dump_offset =
            render_rect.left + mem_addr.len() as i32 + hex_dump_dimensions.padding();
        let byte_at_cols = bytes[row_first_byte..]
            .chars()
            .zip(0..hex_dump_dimensions.width());
        for (byte, col_index) in byte_at_cols {
            let byte_offset = row_first_byte + col_index as usize;
            if byte_offset >= highlighted_byte_start && byte_offset < highlighted_byte_end {
                window.attron(pancurses::A_BLINK);
            } else {
                window.attroff(pancurses::A_BLINK);
            }
            window.mvaddch(y, begin_dump_offset + col_index, byte);
        }
        window.attroff(pancurses::A_BLINK);
    }
}

fn obfuscate_words(
    words: &[String],
    target_size: usize,
    rng: &mut dyn RangeRng<usize>,
) -> (String, Vec<usize>) {
    let initial_length_from_words: usize = words.iter().fold(0, |acc, word| acc + word.len());
    let remaining_char_count_to_generate = target_size - initial_length_from_words;

    // place the words at offsets within the final obfuscated string such that filling in between/around
    // those words will generate the final obfuscated string
    let offsets = {
        let mut offsets = Vec::new();
        for word in words.iter() {
            for offset in offsets.iter_mut() {
                *offset += word.len();
            }
            offsets.insert(0, 0);
        }

        for _ in 0..remaining_char_count_to_generate {
            // Increment each offset starting from a random offset.
            // This simulates adding a character before a random word in the final obfuscated string
            // e.g. if offsets_to_bump_start = 1, then a character is added between words 0 and 1
            // and words 1->onward are then offset by an additional character.
            let offsets_to_bump_start = rng.gen_range(0, offsets.len() + 1);

            for i in offsets_to_bump_start..offsets.len() {
                offsets[i] += 1;
            }
        }
        offsets
    };

    let mut string_builder = String::with_capacity(target_size);

    // fill the string with the initial garbage chars
    for _ in 0..remaining_char_count_to_generate {
        let garbage_char = rng.gen_range('#' as usize, '.' as usize) as u8 as char;
        string_builder.push(garbage_char);
    }

    // insert each of the offset words
    for (word, offset) in words.iter().zip(offsets.iter()) {
        string_builder.insert_str(*offset, &word);
    }

    (string_builder, offsets)
}

pub fn run_game(difficulty: Difficulty) {
    const HEX_DUMP_PANE: HexDumpPane = HexDumpPane {
        dump_width: 12,  // 12 characters per row of the hexdump
        dump_height: 16, // 16 rows of hex dump per dump pane
        // TODO: update this to not be characters but bytes or bits or something
        addr_width: "0x1234".len() as i32, // 2 byte memaddr
        addr_to_dump_padding: 4,           // horizontal padding between panes in the memdump window
    };

    const HEXDUMP_PANE_VERT_OFFSET: i32 = 5;

    let left_hex_dump_rect = Rect {
        left: 0,
        top: HEXDUMP_PANE_VERT_OFFSET,
        width: HEX_DUMP_PANE.full_width(),
        height: HEX_DUMP_PANE.height(),
    };

    let right_hex_dump_rect = Rect {
        left: left_hex_dump_rect.width + HEX_DUMP_PANE.padding(),
        top: left_hex_dump_rect.top,
        width: HEX_DUMP_PANE.full_width(),
        height: HEX_DUMP_PANE.height(),
    };

    // Generate a random set of words based on the provided difficulty setting
    let mut rng = ThreadRangeRng::new();
    let words = generate_words(difficulty, &mut rng);

    // Generate a mock hexdump from the randomly generated words
    const MAX_BYTES_IN_DUMP: usize = HEX_DUMP_PANE.max_bytes_in_pane() * 2; // 2 dump panes
    let (hex_dump, word_offsets) = obfuscate_words(&words, MAX_BYTES_IN_DUMP, &mut rng);

    // For visual flair, randomize the mem address of the hex dump
    const MIN_MEMADDR: usize = 0xCC00;
    const MAX_MEMADDR: usize = 0xFFFF - MAX_BYTES_IN_DUMP;
    const_assert!(MIN_MEMADDR < MAX_MEMADDR);
    let hexdump_start_addr = rng.gen_range(MIN_MEMADDR, MAX_MEMADDR);

    // initially select the first character in the row pane
    let mut selected_chunk = SelectedChunk {
        pane_num: 0,
        row_num: 0,
        col_start: 0,
        len: 1,
    };

    // Immediately refit the selection in case the first character is part of a larger word
    selected_chunk = refit_selection(selected_chunk, &words, &word_offsets, &HEX_DUMP_PANE);

    // setup the window
    let window = pancurses::initscr();
    pancurses::noecho(); // prevent key inputs rendering to the screen
    pancurses::cbreak();
    pancurses::curs_set(0);
    pancurses::set_title(TITLE);
    window.nodelay(true); // don't block waiting for key inputs (we'll poll)
    window.keypad(true); // let special keys be captured by the program (i.e. esc/backspace/del/arrow keys)

    // TODO: refactor this loop for readability and testing
    loop {
        let polled_input_cmd = match window.getch() {
            Some(pancurses::Input::Character('w')) => Some(InputCmd::Move(Movement::Up)),
            Some(pancurses::Input::Character('s')) => Some(InputCmd::Move(Movement::Down)),
            Some(pancurses::Input::Character('a')) => Some(InputCmd::Move(Movement::Left)),
            Some(pancurses::Input::Character('d')) => Some(InputCmd::Move(Movement::Right)),
            Some(pancurses::Input::Character('q')) => Some(InputCmd::Quit),
            // TODO: handle entering in guesses... Some(pancurses::Input::Character('ENTER')) => (),
            _ => None,
        };

        if let Some(input_cmd) = polled_input_cmd {
            match input_cmd {
                // Handle moving the cursor around the hex dump pane
                InputCmd::Move(movement) => {
                    // Move the cursor based on our input
                    selected_chunk = move_selection(selected_chunk, movement, &HEX_DUMP_PANE, 2);
                    // If the cursor is now selecting a word, refit the selection highlight for the whole word
                    selected_chunk =
                        refit_selection(selected_chunk, &words, &word_offsets, &HEX_DUMP_PANE);
                }

                // Handle selecting a word
                InputCmd::Select => unimplemented!(),

                // Handle quitting the game early
                InputCmd::Quit => break,
            }
        }

        window.clear();

        // Render the hex dump header
        window.mvaddstr(0, 0, "ROBCO INDUSTRIES (TM) TERMALINK PROTOCOL");
        window.mvaddstr(1, 0, "ENTER PASSWORD NOW");
        const BLOCK_CHAR: char = '#';
        window.mvaddstr(
            3,
            0,
            format!(
                "# ATTEMPT(S) LEFT: {} {} {} {}",
                BLOCK_CHAR, BLOCK_CHAR, BLOCK_CHAR, BLOCK_CHAR
            ),
        );

        let highlighted_byte_range = {
            let start = selected_chunk.pane_num * HEX_DUMP_PANE.max_bytes_in_pane()
                + selected_chunk.row_num * HEX_DUMP_PANE.width() as usize
                + selected_chunk.col_start;
            let end = start + selected_chunk.len;
            (start, end)
        };

        // Render the left hex dump pane
        render_hexdump_pane(
            &window,
            &HEX_DUMP_PANE,
            &left_hex_dump_rect,
            hexdump_start_addr,
            &hex_dump,
            0,
            highlighted_byte_range,
        );

        // Render the right hex dump pane
        render_hexdump_pane(
            &window,
            &HEX_DUMP_PANE,
            &right_hex_dump_rect,
            hexdump_start_addr,
            &hex_dump,
            hex_dump.len() / 2,
            highlighted_byte_range,
        );

        window.refresh();

        // No need to waste cycles doing nothing but rendering over and over. Yield the processor.
        std::thread::sleep(std::time::Duration::from_millis(33));
    }
    pancurses::endwin();

    // now let's run a mock game_loop
    // run_game_from_line_console(&words, &mut rng);
}

fn _run_game_from_line_console(words: &[String], rng: &mut dyn RangeRng<usize>) {
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
            let expected_word_cnt = 12;
            for i in 0..expected_word_cnt {
                let generated_word = &generated_words[i];
                let expected_word = expected_words[i % expected_words.len()];
                assert_eq!(generated_word, expected_word);
            }
        }
    }

    #[test]
    fn test_obfuscate_words() {
        let mut rng = ThreadRangeRng::new();
        let words: Vec<String> = ["apple", "orange", "banana"]
            .iter()
            .map(|s| String::from(*s))
            .collect();

        const HEX_BYTE_COUNT: usize = 100;
        let (obfuscated_words, offsets) = obfuscate_words(&words, HEX_BYTE_COUNT, &mut rng);

        assert_eq!(HEX_BYTE_COUNT, obfuscated_words.len());
        for (word, word_offset) in words.iter().zip(offsets.iter()) {
            let word_in_blob = &obfuscated_words[*word_offset..][..word.len()];
            assert_eq!(word, word_in_blob);
        }
    }

    #[test]
    fn todo_build_gate_stop() {
        todo!("There are a bunch of todos in this file (mostly around refactoring). Try and address before next PR.");
    }

    #[test]
    fn bug_check() {
        // TODO: write test and fix
        // XXXXXX   dokieX <-- move the cursor from X to 'e' in "dokie"
        // XXXXXX   XXXXXX
        // XXXXXX   XXXXXX
        // XXXXXX   XXXXXX
        // XXokie   XXXXXX
        todo!("if a word wraps between panes and you move into a selection from the end of a word it crashes");
    }

    // what is interesting behavior to test?
    // - moving from one single char cell to the next
    // - moving from one single char cell horizontally across panes (right)
    // - moving from one single char cell horizontally across panes (left)
    // - wrapping around vertically in a pane (should this move to the next pane instead?)
    // - moving from one highlighted word to the next entry
    // - moving from one highlighted word when that word wraps around rows
    // - jumping down into the middle of a highlighted word
    // - jumping up into the middle of a highlighted word
    // - moving back into the middle of a highlighted word
}
