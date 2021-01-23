use crate::randwrapper::{select_rand, RangeRng};

// Each dict chunk represents all words of the same length from our src dict. This partitioning is a
// quick optimization since the cracker game will only concern itself with words of the same length.
pub struct EnglishDictChunk {
    word_len: usize,
    word_set: Vec<String>,
}

impl EnglishDictChunk {
    pub fn load(word_len: usize) -> Self {
        let dict_file_name = format!("src/dict/{}_char_words_alpha.txt", word_len);
        let word_set = snm_simple_file::read_lines(&dict_file_name).collect();
        EnglishDictChunk { word_len, word_set }
    }

    pub fn is_word(&self, word: &str) -> bool {
        assert_eq!(self.word_len, word.len());
        self.word_set.iter().any(|word_in_set| word_in_set == word)
    }

    pub fn get_random_word(&self, rng: &mut dyn RangeRng<usize>) -> String {
        select_rand(&self.word_set, rng).clone()
    }
}
