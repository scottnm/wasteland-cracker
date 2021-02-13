// Extended work breakdown
// - async dict load to hide loading times
// - support a better interface for providing the input to the solver
// - add timed mode
// - add extra game rules for handling selecting brackets?
// - use appropriate font to give it a "fallout feel"
// - use appropriate animations to give it a "fallout feel"
// - SFX
// - refactor out tui utils into its own module
// - improve TUI navigation logic to be more intuitive
// - refactor different components into modules
// - address all cleanup/refactoring todos

use crate::dict::dict::EnglishDictChunk;
use crate::utils::rand::{RangeRng, ThreadRangeRng};
use crate::utils::utils::{keys, matching_char_count_ignore_case, Rect};

const MAX_ATTEMPTS: usize = 4;

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

// TODO: should this be split out into two structs?
// one for dump dimensions and another for formatting? (i.e. the padding param)
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
#[derive(Debug, PartialEq, Eq)]
struct SelectedChunk {
    pane_num: usize,
    row_num: usize,
    col_start: usize,
    len: usize,
}

// H_amming D_istance D_istribution Entry
#[derive(Clone, Copy)]
struct HDDEntry {
    num_words: usize, // the number of words to look for with this hamming distance
    hamming_distance: usize, // the hamming distance to look for
}

fn get_hamming_distance_distribution(difficulty: Difficulty) -> [HDDEntry; 4] {
    let distances = match difficulty {
        Difficulty::VeryEasy => [1, 2, 3, 4],
        Difficulty::Easy => [1, 3, 4, 5],
        Difficulty::Average => [1, 3, 5, 7],
        Difficulty::Hard => [1, 4, 6, 9],
        Difficulty::VeryHard => [1, 3, 7, 10],
    };

    [
        HDDEntry {
            num_words: 1,
            hamming_distance: distances[0],
        },
        HDDEntry {
            num_words: 2,
            hamming_distance: distances[1],
        },
        HDDEntry {
            num_words: 3,
            hamming_distance: distances[2],
        },
        HDDEntry {
            num_words: 5,
            hamming_distance: distances[3],
        },
    ]
}

fn get_word_len_for_difficulty(difficulty: Difficulty) -> usize {
    match difficulty {
        Difficulty::VeryEasy => 4,
        Difficulty::Easy => 6,
        Difficulty::Average => 8,
        Difficulty::Hard => 10,
        Difficulty::VeryHard => 12,
    }
}

fn generate_words(
    dict_chunk: &EnglishDictChunk,
    hd_distribution: &[HDDEntry; 4],
    rng: &mut dyn RangeRng<usize>,
) -> (Vec<String>, String) {
    let total_words_in_distribution = hd_distribution.iter().fold(0, |acc, e| acc + e.num_words);

    let mut words = Vec::with_capacity(total_words_in_distribution + 1);
    let goal_word = dict_chunk.get_random_word(rng);
    words.push(goal_word.clone());

    let mut current_hd_distribution_index = 0;
    let mut hd_distribution_tracker: [HDDEntry; 4] = hd_distribution.clone();
    let mut hamming_distance_sorted_iter = dict_chunk.get_hamming_distance_sorted_words(&goal_word);

    while current_hd_distribution_index < hd_distribution_tracker.len() {
        let current_hd_distribution_entry =
            &mut hd_distribution_tracker[current_hd_distribution_index];
        assert_ne!(current_hd_distribution_entry.num_words, 0);

        let next_sorted_word_pair = hamming_distance_sorted_iter.next();
        let (word, hamming_distance) = match next_sorted_word_pair {
            None => break, // we are out of words!
            Some(sorted_word_pair) => sorted_word_pair,
        };

        if hamming_distance >= current_hd_distribution_entry.hamming_distance {
            current_hd_distribution_entry.num_words -= 1;
            words.push(String::from(word));

            if current_hd_distribution_entry.num_words == 0 {
                current_hd_distribution_index += 1;
            }
        }
    }

    // the code can manage finding fewer words, but this represents a bug
    assert_eq!(words.len(), total_words_in_distribution + 1);
    (words, goal_word)
}

fn simple_shuffle<T>(mut v: Vec<T>, rng: &mut dyn RangeRng<usize>) -> Vec<T> {
    const NUM_SWAPS: usize = 100; // a good-enough heuristic for shuffling the words in place

    for _ in 0..NUM_SWAPS {
        let index = rng.gen_range(0, v.len());
        v.swap(0, index);
    }

    v
}

fn generate_words_from_difficulty(
    difficulty: Difficulty,
    rng: &mut dyn RangeRng<usize>,
) -> (Vec<String>, String) {
    let dict_chunk = EnglishDictChunk::load(get_word_len_for_difficulty(difficulty));
    let hd_distribution = get_hamming_distance_distribution(difficulty);
    generate_words(&dict_chunk, &hd_distribution, rng)
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

fn refit_selection<S: AsRef<str>>(
    selection: SelectedChunk,
    words: &[S],
    word_offsets: &[usize],
    hex_dump_pane_dimensions: &HexDumpPane,
) -> SelectedChunk {
    let cursor_index = selection.pane_num * hex_dump_pane_dimensions.max_bytes_in_pane()
        + selection.row_num * hex_dump_pane_dimensions.width() as usize
        + selection.col_start;

    // TODO: clean up this implementation. It's a bit ugly
    let mut result_selection = selection;

    // turn our list of words and word_offsets into a list of ranges where those words live
    // in the contiguous hex dump memory span
    let word_ranges = words
        .iter()
        .zip(word_offsets.iter())
        .map(|(word, word_offset)| (*word_offset, word_offset + word.as_ref().len()));

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

fn try_select_word<'a, S: AsRef<str>>(
    selection: &SelectedChunk,
    words: &'a [S],
    word_offsets: &[usize],
    hex_dump_pane_dimensions: &HexDumpPane,
) -> Option<&'a str> {
    let cursor_index = selection.pane_num * hex_dump_pane_dimensions.max_bytes_in_pane()
        + selection.row_num * hex_dump_pane_dimensions.width() as usize
        + selection.col_start;

    for (word, word_offset) in words.iter().zip(word_offsets.iter()) {
        if cursor_index >= *word_offset && cursor_index < word_offset + word.as_ref().len() {
            // For safety we'll return the word if the cursor is anywhere in the word selection,
            // but we only really expect it to be at the start of the word.
            assert_eq!(cursor_index, *word_offset);
            assert_eq!(selection.len, word.as_ref().len());
            return Some(word.as_ref());
        }
    }

    None
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

fn render_game_window(
    window: &pancurses::Window,
    cursor_selection: &SelectedChunk,
    hex_dump_start_addr: usize,
    hex_dump: &str,
    hex_dump_dimensions: &HexDumpPane,
    hex_dump_rects: &[Rect],
    denied_selections: &[(&str, usize)],
    accepted_selection: &Option<&str>,
) {
    // Render the hex dump header
    window.mvaddstr(0, 0, "ROBCO INDUSTRIES (TM) TERMALINK PROTOCOL");
    window.mvaddstr(1, 0, "ENTER PASSWORD NOW");

    let attempts_left_title = "# ATTEMPT(S) LEFT:";
    window.mvaddstr(3, 0, attempts_left_title);

    let total_attempts = {
        let mut attempts = denied_selections.len();
        if accepted_selection.is_some() {
            attempts += 1;
        }
        attempts
    };

    for i in 0..(MAX_ATTEMPTS - total_attempts) {
        const BLOCK_CHAR_CHUNK: &str = " #";
        let offset = attempts_left_title.len() + i * BLOCK_CHAR_CHUNK.len();
        window.mvaddstr(3, offset as i32, BLOCK_CHAR_CHUNK);
    }

    let highlighted_byte_range = {
        let start = cursor_selection.pane_num * hex_dump_dimensions.max_bytes_in_pane()
            + cursor_selection.row_num * hex_dump_dimensions.width() as usize
            + cursor_selection.col_start;
        let end = start + cursor_selection.len;
        (start, end)
    };

    // render each hex dump pane (assume ordered left to right)
    for hex_dump_pane_index in 0..hex_dump_rects.len() {
        let hex_dump_rect = &hex_dump_rects[hex_dump_pane_index];
        let pane_byte_offset = hex_dump_pane_index * hex_dump_dimensions.max_bytes_in_pane();
        render_hexdump_pane(
            &window,
            hex_dump_dimensions,
            &hex_dump_rect,
            hex_dump_start_addr + pane_byte_offset,
            &hex_dump,
            pane_byte_offset,
            highlighted_byte_range,
        );
    }

    // Render the selection history
    let mut row_cursor = window.get_max_y() - 5; // 5 provides a nice padding from the bottom
    let selection_history_start_col = window.get_max_x() - 20; // 20 provides enough room for any selected word

    let write_history_entries = |row: &mut i32, entries: &[&str]| {
        for entry in entries.iter().rev() {
            window.mvaddstr(*row, selection_history_start_col, format!(">{}", entry));
            *row -= 1;
        }
    };

    // first render the accepted solution if provided or the failure text if we've lost
    window.attron(pancurses::A_BLINK);
    if let Some(accepted_selection) = accepted_selection {
        let lines = [
            accepted_selection,
            "Exact match!",
            "Please wait",
            "while system",
            "is accessed.",
        ];
        write_history_entries(&mut row_cursor, &lines);
    } else if denied_selections.len() == MAX_ATTEMPTS {
        let lines = ["TOO MANY ATTEMPTS!", "Entering secure", "lock mode"];
        write_history_entries(&mut row_cursor, &lines);
    }
    window.attroff(pancurses::A_BLINK);

    // now render each denied entry
    for (denied_word, matching_char_count) in denied_selections.iter().rev() {
        let char_count_str = format!("{}/{} correct.", matching_char_count, denied_word.len());
        let lines = ["Entry denied", &char_count_str, denied_word];
        write_history_entries(&mut row_cursor, &lines);
    }
}

pub fn run_game(difficulty: Difficulty, window: &pancurses::Window) {
    const HEX_DUMP_PANE: HexDumpPane = HexDumpPane {
        dump_width: 12,  // 12 characters per row of the hexdump
        dump_height: 16, // 16 rows of hex dump per dump pane
        // TODO: update this to not be characters but bytes or bits or something
        addr_width: "0x1234".len() as i32, // 2 byte memaddr
        addr_to_dump_padding: 4,           // horizontal padding between panes in the memdump window
    };

    const HEXDUMP_PANE_VERT_OFFSET: i32 = 5;

    let hex_dump_rects = {
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

        [left_hex_dump_rect, right_hex_dump_rect]
    };

    // Generate a random set of words based on the provided difficulty setting
    let mut rng = ThreadRangeRng::new();
    let (unshuffled_words, solution) = generate_words_from_difficulty(difficulty, &mut rng);
    assert_eq!(unshuffled_words.len(), 12); // the game isn't broken if we don't have 12 words but it represents a bug
    let words = simple_shuffle(unshuffled_words, &mut rng);

    let mut denied_selections = Vec::new();
    let mut accepted_selection = None;
    fn is_game_over(
        denied_selections: &[(&str, usize)],
        accepted_selection: &Option<&str>,
    ) -> bool {
        denied_selections.len() == MAX_ATTEMPTS || accepted_selection.is_some()
    }
    const GAME_OVER_HOLD_TIME: std::time::Duration = std::time::Duration::from_secs(3);
    let mut game_over_timer = None;

    // Generate a mock hexdump from the randomly generated words
    const MAX_BYTES_IN_DUMP: usize = HEX_DUMP_PANE.max_bytes_in_pane() * 2; // 2 dump panes
    let (hex_dump, word_offsets) = obfuscate_words(&words, MAX_BYTES_IN_DUMP, &mut rng);

    // For visual flair, randomize the mem address of the hex dump
    const MIN_MEMADDR: usize = 0xCC00;
    const MAX_MEMADDR: usize = 0xFFFF - MAX_BYTES_IN_DUMP;
    const_assert!(MIN_MEMADDR < MAX_MEMADDR);
    let hex_dump_start_addr = rng.gen_range(MIN_MEMADDR, MAX_MEMADDR);

    // initially select the first character in the row pane
    let mut selected_chunk = SelectedChunk {
        pane_num: 0,
        row_num: 0,
        col_start: 0,
        len: 1,
    };

    // Immediately refit the selection in case the first character is part of a larger word
    selected_chunk = refit_selection(selected_chunk, &words, &word_offsets, &HEX_DUMP_PANE);

    // TODO: refactor this loop for readability and testing
    loop {
        // Poll for input
        let polled_input_cmd = match window.getch() {
            Some(pancurses::Input::Character('w')) => Some(InputCmd::Move(Movement::Up)),
            Some(pancurses::Input::Character('s')) => Some(InputCmd::Move(Movement::Down)),
            Some(pancurses::Input::Character('a')) => Some(InputCmd::Move(Movement::Left)),
            Some(pancurses::Input::Character('d')) => Some(InputCmd::Move(Movement::Right)),
            Some(pancurses::Input::Character(keys::ASCII_ESC)) => Some(InputCmd::Quit),
            Some(pancurses::Input::Character(keys::ASCII_ENTER))
            | Some(pancurses::Input::KeyEnter) => Some(InputCmd::Select),
            _ => None,
        };

        // Handle the input
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
                InputCmd::Select => {
                    if !is_game_over(&denied_selections, &accepted_selection) {
                        let selected_word_result =
                            try_select_word(&selected_chunk, &words, &word_offsets, &HEX_DUMP_PANE);
                        if let Some(selected_word) = selected_word_result {
                            if selected_word == solution {
                                accepted_selection = Some(selected_word);
                            } else {
                                let matching_char_count =
                                    matching_char_count_ignore_case(&solution, &selected_word);
                                denied_selections.push((selected_word, matching_char_count));
                            }
                        }

                        if is_game_over(&denied_selections, &accepted_selection) {
                            game_over_timer = Some(std::time::Instant::now());
                        }
                    }
                }

                // Handle quitting the game early
                InputCmd::Quit => break,
            }
        }

        // Render the next frame
        window.erase();
        render_game_window(
            &window,
            &selected_chunk,
            hex_dump_start_addr,
            &hex_dump,
            &HEX_DUMP_PANE,
            &hex_dump_rects,
            &denied_selections,
            &accepted_selection,
        );
        window.refresh();

        // No need to waste cycles doing nothing but rendering over and over.
        // Yield the processor until the next frame.
        std::thread::sleep(std::time::Duration::from_millis(33));

        // If the game is over and we've been staring at the screen for long enough exit
        if game_over_timer.is_some() && game_over_timer.unwrap().elapsed() >= GAME_OVER_HOLD_TIME {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::rand::mocks as rand_mocks;

    #[test]
    fn test_word_generation() {
        // use a single-value rng for value 0. This will make sure the goal_word is the first word in the original word list
        let mut rng = rand_mocks::SingleValueRangeRng::new(0);

        let test_hd_distribution = [
            HDDEntry {
                num_words: 1,
                hamming_distance: 1,
            },
            HDDEntry {
                num_words: 2,
                hamming_distance: 2,
            },
            HDDEntry {
                num_words: 3,
                hamming_distance: 3,
            },
            HDDEntry {
                num_words: 4,
                hamming_distance: 4,
            },
        ];

        let goal_word = "dude";
        let words = [
            goal_word, // 0
            "dede",    // 1
            "door",    // 3
            "dodo",    // 2
            "doom",    // 3
            "abba",    // 4
            "rude",    // 1
            "duds",    // 1
            "rube",    // 2
            "cube",    // 2
            "sick",    // 4
            "stop",    // 4
            "soil",    // 4
            "roll",    // 4
        ];

        let expected_generated_words = [
            goal_word, // goal
            "dede",    // hd 1
            "dodo", "rube", // hd 2
            "door", "doom", "abba", // hd 3
            "sick", "stop", "soil", "roll", // hd 4
        ];

        let test_dict = EnglishDictChunk::new_mock(4, &words);
        let (generated_words, solution) =
            generate_words(&test_dict, &test_hd_distribution, &mut rng);

        assert_eq!(solution, goal_word);
        assert_eq!(generated_words, expected_generated_words);
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

    fn move_and_refit(
        mut selection: SelectedChunk,
        movement: Movement,
        words: &[&str],
        word_offsets: &[usize],
        hex_dump_pane_dimensions: &HexDumpPane,
        num_panes: usize,
    ) -> SelectedChunk {
        selection = move_selection(selection, movement, &hex_dump_pane_dimensions, num_panes);
        refit_selection(selection, &words, &word_offsets, &hex_dump_pane_dimensions)
    }

    #[test]
    fn test_single_char_move_next() {
        // .... ....
        // .abc .xyz
        // .... ....
        //  ^^
        let start_selection = SelectedChunk {
            pane_num: 0,
            row_num: 2,
            col_start: 1,
            len: 1,
        };
        let expected_end_selection = SelectedChunk {
            pane_num: 0,
            row_num: 2,
            col_start: 2,
            len: 1,
        };
        let movement = Movement::Right;
        let words = ["abc", "xyz"];
        let word_offsets = [5, 17];
        let hex_dump_pane_dimensions = HexDumpPane {
            dump_width: 4,
            dump_height: 3,
            addr_width: 0,           // unused
            addr_to_dump_padding: 0, // unused
        };

        let end_selection = move_and_refit(
            start_selection,
            movement,
            &words,
            &word_offsets,
            &hex_dump_pane_dimensions,
            2,
        );

        assert_eq!(end_selection, expected_end_selection);
    }

    #[test]
    fn test_single_char_move_across_panes_right() {
        // .... ....
        // .abc .xyz
        // .... ....
        //    ^ ^
        let start_selection = SelectedChunk {
            pane_num: 0,
            row_num: 2,
            col_start: 3,
            len: 1,
        };
        let expected_end_selection = SelectedChunk {
            pane_num: 1,
            row_num: 2,
            col_start: 0,
            len: 1,
        };
        let movement = Movement::Right;
        let words = ["abc", "xyz"];
        let word_offsets = [5, 17];
        let hex_dump_pane_dimensions = HexDumpPane {
            dump_width: 4,
            dump_height: 3,
            addr_width: 0,           // unused
            addr_to_dump_padding: 0, // unused
        };

        let end_selection = move_and_refit(
            start_selection,
            movement,
            &words,
            &word_offsets,
            &hex_dump_pane_dimensions,
            2,
        );

        assert_eq!(end_selection, expected_end_selection);
    }

    #[test]
    fn test_single_char_move_across_panes_left() {
        // .... ....
        // .abc .xyz
        // .... ....
        // ^       ^
        let start_selection = SelectedChunk {
            pane_num: 0,
            row_num: 2,
            col_start: 0,
            len: 1,
        };
        let expected_end_selection = SelectedChunk {
            pane_num: 1,
            row_num: 2,
            col_start: 3,
            len: 1,
        };
        let movement = Movement::Left;
        let words = ["abc", "xyz"];
        let word_offsets = [5, 17];
        let hex_dump_pane_dimensions = HexDumpPane {
            dump_width: 4,
            dump_height: 3,
            addr_width: 0,           // unused
            addr_to_dump_padding: 0, // unused
        };

        let end_selection = move_and_refit(
            start_selection,
            movement,
            &words,
            &word_offsets,
            &hex_dump_pane_dimensions,
            2,
        );

        assert_eq!(end_selection, expected_end_selection);
    }

    #[test]
    fn test_word_move_wrap_vertical() {
        //        v-start
        // .... ....
        // .abc .xyz
        // .... ....
        //        ^-end
        let start_selection = SelectedChunk {
            pane_num: 1,
            row_num: 0,
            col_start: 2,
            len: 1,
        };
        let expected_end_selection = SelectedChunk {
            pane_num: 1,
            row_num: 2,
            col_start: 2,
            len: 1,
        };
        let movement = Movement::Up;
        let words = ["abc", "xyz"];
        let word_offsets = [5, 17];
        let hex_dump_pane_dimensions = HexDumpPane {
            dump_width: 4,
            dump_height: 3,
            addr_width: 0,           // unused
            addr_to_dump_padding: 0, // unused
        };

        let end_selection = move_and_refit(
            start_selection,
            movement,
            &words,
            &word_offsets,
            &hex_dump_pane_dimensions,
            2,
        );

        assert_eq!(end_selection, expected_end_selection);
    }

    #[test]
    fn test_word_move_right() {
        // v  v
        // abc. ....
        // .... .xyz
        // .... ....
        let start_selection = SelectedChunk {
            pane_num: 0,
            row_num: 0,
            col_start: 0,
            len: 3,
        };
        let expected_end_selection = SelectedChunk {
            pane_num: 0,
            row_num: 0,
            col_start: 3,
            len: 1,
        };
        let movement = Movement::Right;
        let words = ["abc", "xyz"];
        let word_offsets = [0, 17];
        let hex_dump_pane_dimensions = HexDumpPane {
            dump_width: 4,
            dump_height: 3,
            addr_width: 0,           // unused
            addr_to_dump_padding: 0, // unused
        };

        let end_selection = move_and_refit(
            start_selection,
            movement,
            &words,
            &word_offsets,
            &hex_dump_pane_dimensions,
            2,
        );

        assert_eq!(end_selection, expected_end_selection);
    }

    #[test]
    fn test_word_move_left() {
        // abc. ....
        // .... .xyz
        // .... ^^..
        let start_selection = SelectedChunk {
            pane_num: 1,
            row_num: 1,
            col_start: 1,
            len: 3,
        };
        let expected_end_selection = SelectedChunk {
            pane_num: 1,
            row_num: 1,
            col_start: 0,
            len: 1,
        };
        let movement = Movement::Left;
        let words = ["abc", "xyz"];
        let word_offsets = [0, 17];
        let hex_dump_pane_dimensions = HexDumpPane {
            dump_width: 4,
            dump_height: 3,
            addr_width: 0,           // unused
            addr_to_dump_padding: 0, // unused
        };

        let end_selection = move_and_refit(
            start_selection,
            movement,
            &words,
            &word_offsets,
            &hex_dump_pane_dimensions,
            2,
        );

        assert_eq!(end_selection, expected_end_selection);
    }

    #[test]
    fn test_move_word_wrapped() {
        //   v  v
        // ..ab ....
        // c... .xyz
        // .... ....
        let start_selection = SelectedChunk {
            pane_num: 0,
            row_num: 0,
            col_start: 2,
            len: 3,
        };
        let expected_end_selection = SelectedChunk {
            pane_num: 1,
            row_num: 0,
            col_start: 0,
            len: 1,
        };
        let movement = Movement::Right;
        let words = ["abc", "xyz"];
        let word_offsets = [2, 17];
        let hex_dump_pane_dimensions = HexDumpPane {
            dump_width: 4,
            dump_height: 3,
            addr_width: 0,           // unused
            addr_to_dump_padding: 0, // unused
        };

        let end_selection = move_and_refit(
            start_selection,
            movement,
            &words,
            &word_offsets,
            &hex_dump_pane_dimensions,
            2,
        );

        assert_eq!(end_selection, expected_end_selection);
    }

    #[test]
    fn test_move_up_into_word_selection_vertical() {
        //        v-start
        // .... ....
        // .abc ....
        // .... .xyz
        //       ^-end
        let start_selection = SelectedChunk {
            pane_num: 1,
            row_num: 0,
            col_start: 2,
            len: 1,
        };
        let expected_end_selection = SelectedChunk {
            pane_num: 1,
            row_num: 2,
            col_start: 1,
            len: 3,
        };
        let movement = Movement::Up;
        let words = ["abc", "xyz"];
        let word_offsets = [5, 21];
        let hex_dump_pane_dimensions = HexDumpPane {
            dump_width: 4,
            dump_height: 3,
            addr_width: 0,           // unused
            addr_to_dump_padding: 0, // unused
        };

        let end_selection = move_and_refit(
            start_selection,
            movement,
            &words,
            &word_offsets,
            &hex_dump_pane_dimensions,
            2,
        );

        assert_eq!(end_selection, expected_end_selection);
    }

    #[test]
    fn test_move_down_into_word_selection_vertical() {
        // .... ....
        // .abc ..v-start
        // .... .xyz
        //       ^-end
        let start_selection = SelectedChunk {
            pane_num: 1,
            row_num: 1,
            col_start: 2,
            len: 1,
        };
        let expected_end_selection = SelectedChunk {
            pane_num: 1,
            row_num: 2,
            col_start: 1,
            len: 3,
        };
        let movement = Movement::Down;
        let words = ["abc", "xyz"];
        let word_offsets = [5, 21];
        let hex_dump_pane_dimensions = HexDumpPane {
            dump_width: 4,
            dump_height: 3,
            addr_width: 0,           // unused
            addr_to_dump_padding: 0, // unused
        };

        let end_selection = move_and_refit(
            start_selection,
            movement,
            &words,
            &word_offsets,
            &hex_dump_pane_dimensions,
            2,
        );

        assert_eq!(end_selection, expected_end_selection);
    }

    #[test]
    fn test_move_left_into_cross_pane_word_selection() {
        //       v-start
        // .... z...
        // .abc ....
        // ..xy ....
        //   ^-end
        let start_selection = SelectedChunk {
            pane_num: 1,
            row_num: 0,
            col_start: 1,
            len: 1,
        };
        let expected_end_selection = SelectedChunk {
            pane_num: 0,
            row_num: 2,
            col_start: 2,
            len: 3,
        };
        let movement = Movement::Left;
        let words = ["abc", "xyz"];
        let word_offsets = [5, 10];
        let hex_dump_pane_dimensions = HexDumpPane {
            dump_width: 4,
            dump_height: 3,
            addr_width: 0,           // unused
            addr_to_dump_padding: 0, // unused
        };

        let end_selection = move_and_refit(
            start_selection,
            movement,
            &words,
            &word_offsets,
            &hex_dump_pane_dimensions,
            2,
        );

        assert_eq!(end_selection, expected_end_selection);
    }

    #[test]
    fn ensure_word_len_for_difficulty_matches_hamming_distance_distribution_for_difficulty() {
        let difficulties = [
            Difficulty::VeryEasy,
            Difficulty::Easy,
            Difficulty::Average,
            Difficulty::Hard,
            Difficulty::VeryHard,
        ];

        for d in &difficulties {
            let word_len = get_word_len_for_difficulty(*d);
            let hamming_distance_distribution = get_hamming_distance_distribution(*d);

            for hdd_entry in &hamming_distance_distribution {
                assert!(hdd_entry.hamming_distance <= word_len);
            }
        }
    }
}
