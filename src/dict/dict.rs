use crate::utils::rand::{select_rand, RangeRng};
use crate::utils::str_utils::hamming_dist_ignore_case;

// Each dict chunk represents all words of the same length from our src dict. This partitioning is a
// quick optimization since the cracker game will only concern itself with words of the same length.
pub struct EnglishDictChunk {
    word_len: usize,
    word_set: Vec<String>,
}

pub struct HammingDistanceIterator<'a> {
    cmp_word: String,
    dict_chunk: &'a EnglishDictChunk,
    next_candidate_distance: usize,
    next_item_candidate_index: usize,
}

impl EnglishDictChunk {
    #[cfg(test)]
    pub fn new_mock(word_len: usize, word_set: &[&str]) -> Self {
        assert!(word_set.iter().all(|w| w.len() == word_len));
        EnglishDictChunk {
            word_len,
            word_set: word_set.iter().map(|s| String::from(*s)).collect(),
        }
    }

    pub fn load(word_len: usize) -> Self {
        let dict_file_name = format!("assets/dict/{}_char_words_alpha.txt", word_len);
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

    pub fn get_hamming_distance_sorted_words(&self, word: &str) -> HammingDistanceIterator {
        HammingDistanceIterator {
            cmp_word: String::from(word),
            dict_chunk: self,
            next_candidate_distance: 1,
            next_item_candidate_index: 0,
        }
    }
}

impl<'a> Iterator for HammingDistanceIterator<'a> {
    type Item = (&'a str, usize);

    fn next(&mut self) -> Option<Self::Item> {
        while self.next_candidate_distance <= self.dict_chunk.word_len {
            let candidate_index = self.next_item_candidate_index;
            let current_candidate_distance = self.next_candidate_distance;

            // if we've made it to the end of the list, start over at the beginning and look for the next hamming distance
            self.next_item_candidate_index += 1;
            if self.next_item_candidate_index >= self.dict_chunk.word_set.len() {
                self.next_item_candidate_index = 0;
                self.next_candidate_distance += 1;
            }

            let candidate = &self.dict_chunk.word_set[candidate_index];
            let candidate_hamming_distance = hamming_dist_ignore_case(&candidate, &self.cmp_word);
            if candidate_hamming_distance == current_candidate_distance {
                return Some((candidate, candidate_hamming_distance));
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hamming_distance_iterator() {
        let word = "pens";

        let word_set = [
            //   3,      1,      2,      4,      0,      1,      1,      3
            "adds", "pans", "pils", "dull", "pens", "pins", "pent", "miss",
        ];

        let expected_words_sorted_by_hamming_distance = [
            ("pans", 1),
            ("pins", 1),
            ("pent", 1),
            ("pils", 2),
            ("adds", 3),
            ("miss", 3),
            ("dull", 4),
        ];
        // 1 less because we shouldn't match our own word
        assert_eq!(
            word_set.len() - 1,
            expected_words_sorted_by_hamming_distance.len()
        );

        let dict_chunk = EnglishDictChunk::new_mock(4, &word_set);
        let words_sorted_by_haming_distance: Vec<(&str, usize)> = dict_chunk
            .get_hamming_distance_sorted_words(&word)
            .collect();
        assert_eq!(
            words_sorted_by_haming_distance,
            expected_words_sorted_by_hamming_distance
        );
    }
}
