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

struct SelectedChunk {
    pane_num: usize,
    row_num: usize,
    col_start: usize,
    len: usize,
    dirty: bool,
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

    const WORDS_TO_GENERATE_COUNT: usize = 12;

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
    bytes: &str,
    selected_chunk: Option<&SelectedChunk>,
) {
    for row in 0..hex_dump_dimensions.height() {
        let byte_offset = (row * hex_dump_dimensions.width()) as usize;
        let mem_addr = format!(
            "0x{:0width$X}",
            mem_start + byte_offset,
            width = hex_dump_dimensions.addr_width() - 2,
        );

        let row_bytes = &bytes[byte_offset..][..hex_dump_dimensions.width() as usize];

        let y = row + render_rect.top;

        // render the memaddr
        window.mvaddstr(y, render_rect.left, &mem_addr);

        // render the dump
        window.mvaddstr(
            y,
            render_rect.left + mem_addr.len() as i32 + hex_dump_dimensions.padding(),
            row_bytes,
        );
    }

    if let Some(selection) = selected_chunk {
        let y = selection.row_num as i32 + render_rect.top;
        let hex_dump_col_offset = render_rect.left
            + hex_dump_dimensions.addr_width() as i32
            + hex_dump_dimensions.padding();
        let row_1_col = hex_dump_col_offset + selection.col_start as i32;
        let selection_len = selection.len as i32;
        let selection_len_row_1 = std::cmp::min(
            hex_dump_dimensions.width() - selection.col_start as i32,
            selection_len,
        );

        window.mvchgat(y, row_1_col, selection_len_row_1, pancurses::A_BLINK, 0);
        if selection_len != selection_len_row_1 {
            let row_2_col = hex_dump_col_offset;
            let selection_len_row_2 = selection_len - selection_len_row_1;
            window.mvchgat(y + 1, row_2_col, selection_len_row_2, pancurses::A_BLINK, 0);
        }
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
        dump_width: 12,                    // 12 characters per row of the hexdump
        dump_height: 16,                   // 16 rows of hex dump per dump pane
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

    // Generate a random set of words based on the difficulty
    let mut rng = ThreadRangeRng::new();
    let words = generate_words(difficulty, &mut rng);
    let (hex_dump, word_offsets) =
        obfuscate_words(&words, HEX_DUMP_PANE.max_bytes_in_pane() * 2, &mut rng);
    let (hex_dump_left_pane, hex_dump_right_pane) =
        hex_dump.split_at(HEX_DUMP_PANE.max_bytes_in_pane());
    let hexdump_start_addr = rng.gen_range(0xCC00, 0xFFFF);

    // initially select the first character in the row pane
    let mut selected_chunk = SelectedChunk {
        pane_num: 0,
        row_num: 0,
        col_start: 0,
        len: 1,
        dirty: true,
    };

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
        #[derive(PartialEq, Eq)]
        enum InputCmd {
            Movement(i32, i32),
            Quit,
        }

        let input_cmd = match window.getch() {
            Some(pancurses::Input::Character('w')) => Some(InputCmd::Movement(0, -1)),
            Some(pancurses::Input::Character('a')) => Some(InputCmd::Movement(-1, 0)),
            Some(pancurses::Input::Character('s')) => Some(InputCmd::Movement(0, 1)),
            Some(pancurses::Input::Character('d')) => {
                Some(InputCmd::Movement(selected_chunk.len as i32, 0))
            }
            Some(pancurses::Input::Character('q')) => Some(InputCmd::Quit),
            // TODO: handle entering in guesses... Some(pancurses::Input::Character('ENTER')) => (),
            _ => None,
        };

        // Handle quitting early
        if input_cmd == Some(InputCmd::Quit) {
            break;
        }

        // Handle movement inputs
        // If we detect an input which says we should move the cursor, update the selected chunk
        if let Some(InputCmd::Movement(col_move, row_move)) = input_cmd {
            let mut next_col = col_move + (selected_chunk.col_start as i32);
            let mut next_row = row_move + (selected_chunk.row_num as i32);
            let mut next_pane = selected_chunk.pane_num as i32;

            if next_col >= HEX_DUMP_PANE.width() {
                next_col = 0;
                next_pane += 1;
            } else if next_col < 0 {
                next_col = HEX_DUMP_PANE.width() - 1;
                next_pane -= 1;
            }

            if next_row >= HEX_DUMP_PANE.height() {
                next_row = 0;
            } else if next_row < 0 {
                next_row = HEX_DUMP_PANE.height() - 1;
            }

            if next_pane >= 2 {
                next_pane = 0;
            } else if next_pane < 0 {
                next_pane = 1;
            }

            selected_chunk = SelectedChunk {
                pane_num: next_pane as usize,
                row_num: next_row as usize,
                col_start: next_col as usize,
                len: 1,
                dirty: true,
            };
        }

        // if we've moved (or just initialized) the chunk selection, we may need to refit it to select
        // a whole word.
        if selected_chunk.dirty {
            // the index of the cursor in the contiguous hex dump memory span
            let cursor_index = selected_chunk.pane_num * HEX_DUMP_PANE.max_bytes_in_pane()
                + selected_chunk.row_num * HEX_DUMP_PANE.width() as usize
                + selected_chunk.col_start;

            // turn our list of words and word_offsets into a list of ranges where those words live
            // in the contiguous hex dump memory span
            let word_ranges = words
                .iter()
                .zip(word_offsets.iter())
                .map(|(word, word_offset)| (*word_offset, word_offset + word.len()));
            for word_range in word_ranges {
                if cursor_index >= word_range.0 && cursor_index < word_range.1 {
                    // if our cursor is on or in the middle of a full word, update the cursor selection
                    // to highlight the whole word
                    let cursor_offset = cursor_index - word_range.0;
                    if cursor_offset > selected_chunk.col_start {
                        selected_chunk.row_num -= 1;
                        selected_chunk.col_start += HEX_DUMP_PANE.width() as usize - cursor_offset;
                    } else {
                        selected_chunk.col_start -= cursor_offset;
                    }
                    selected_chunk.len = word_range.1 - word_range.0;
                    break;
                }
            }

            selected_chunk.dirty = false;
        }

        let (left_chunk_selection, right_chunk_selection) = if selected_chunk.pane_num == 0 {
            (Some(&selected_chunk), None)
        } else {
            (None, Some(&selected_chunk))
        };

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

        // Render the left hex dump pane
        render_hexdump_pane(
            &window,
            &HEX_DUMP_PANE,
            &left_hex_dump_rect,
            hexdump_start_addr,
            &hex_dump_left_pane,
            left_chunk_selection,
        );

        // Render the right hex dump pane
        render_hexdump_pane(
            &window,
            &HEX_DUMP_PANE,
            &right_hex_dump_rect,
            hexdump_start_addr + hex_dump_left_pane.len(),
            &hex_dump_right_pane,
            right_chunk_selection,
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
}
