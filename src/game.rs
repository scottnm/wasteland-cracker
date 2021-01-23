use crate::dict;
use crate::randwrapper::RangeRng;

const TITLE: &str = "FONV: Terminal Cracker";

#[derive(Debug, Clone, Copy)]
pub enum Difficulty {
    VeryEasy,
    Easy,
    Average,
    Hard,
    VeryHard,
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
