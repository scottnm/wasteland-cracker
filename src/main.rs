#[macro_use]
extern crate lazy_static;
extern crate snm_simple_file;

#[derive(Debug, PartialEq, Eq)]
enum InputValidationErr {
    InputEmpty,
    InvalidPasswordLengthFound,
    NonEnglishWordFound,
}

struct EnglishDict {
    words: std::collections::HashSet<String>,
}

impl EnglishDict {
    fn load() -> Self {
        Self {
            words: snm_simple_file::read_lines("src/dict/words_alpha.txt").collect(),
        }
    }

    fn is_word(word: &str) -> bool {
        lazy_static! {
            static ref DICT: EnglishDict = EnglishDict::load();
        }

        DICT.words.contains(word)
    }
}

fn validate_input_passwords(pwds: Vec<String>) -> Result<Vec<String>, InputValidationErr> {
    if pwds.len() == 0 {
        return Err(InputValidationErr::InputEmpty);
    }

    let required_len = pwds[0].len();
    let equal_len = pwds.iter().all(|p| p.len() == required_len);
    if !equal_len {
        return Err(InputValidationErr::InvalidPasswordLengthFound);
    }

    let all_valid_words = pwds.iter().all(|p| EnglishDict::is_word(&p));
    if !all_valid_words {
        return Err(InputValidationErr::NonEnglishWordFound);
    }

    Ok(pwds)
}

#[derive(Debug)]
struct KnownGuess {
    word: String,
    char_count: usize,
}

impl KnownGuess {
    fn new<S>(word: S, char_count: usize) -> Self
    where
        S: AsRef<str>,
    {
        KnownGuess {
            word: String::from(word.as_ref()),
            char_count,
        }
    }
}

fn filter_matching_passwords<S>(guess: &KnownGuess, mut passwords: Vec<S>) -> Vec<S>
where
    S: AsRef<str>,
{
    for i in (0..passwords.len()).rev() {
        let matching_count = passwords[i]
            .as_ref()
            .chars()
            .zip(guess.word.chars())
            .filter(|(a, b)| a == b)
            .count();
        if matching_count != guess.char_count {
            passwords.swap_remove(i);
        }
    }
    passwords
}

fn main() {
    let file_arg: String = std::env::args().nth(1).unwrap();
    let input_passwords = {
        let pwds: Vec<String> = snm_simple_file::read_lines(&file_arg).collect();
        match validate_input_passwords(pwds) {
            Ok(validated_pwds) => validated_pwds,
            Err(e) => panic!("Input failed validation: {:?}", e),
        }
    };

    let mut known_guesses = {
        let mut known_guesses = Vec::new();
        let guess_args: Vec<String> = std::env::args().skip(2).collect();
        for guess_slice in guess_args.chunks(2) {
            let guess_word = &guess_slice[0];
            let guess_char_count = &guess_slice[1];
            known_guesses.push(KnownGuess::new(
                guess_word,
                guess_char_count.parse().unwrap(),
            ));
        }
        known_guesses
    };

    for guess in &known_guesses {
        if !input_passwords.contains(&guess.word) {
            panic!("{} was not found in password list!", guess.word);
        }
    }

    let mut remaining_passwords = input_passwords.clone();
    for known_guess in &known_guesses {
        remaining_passwords = filter_matching_passwords(&known_guess, remaining_passwords);
    }

    while remaining_passwords.len() > 1 {
        println!("Remaining passwords:");
        for p in &remaining_passwords {
            println!("    {}", p);
        }

        let guess_word: String = text_io::read!("{}");
        let guess_char_count: usize = text_io::read!("{}");
        let next_guess = KnownGuess::new(&guess_word, guess_char_count);
        if !remaining_passwords.contains(&next_guess.word) {
            println!("{} was not found in password list!", next_guess.word);
            continue;
        }

        remaining_passwords = filter_matching_passwords(&next_guess, remaining_passwords);
        known_guesses.push(next_guess);
    }

    match &remaining_passwords[..] {
        [] => println!("No solution matched the provided guesses!"),
        [solution] => println!("The password is... {}", solution),
        _ => (),
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_input_validation_empty_input() {
        assert_eq!(
            validate_input_passwords(vec![]).unwrap_err(),
            InputValidationErr::InputEmpty,
        );
    }

    #[test]
    fn check_input_validation_unequal_len_input() {
        assert_eq!(
            validate_input_passwords(vec![
                String::from("apple"),
                String::from("bale"),
                String::from("grape")
            ])
            .unwrap_err(),
            InputValidationErr::InvalidPasswordLengthFound,
        );
    }

    #[test]
    fn check_input_validation_valid_words() {
        let input_with_valid_words = vec![
            String::from("apple"),
            String::from("seeds"),
            String::from("grape"),
        ];

        let input_with_invalid_words = vec![
            String::from("apple"),
            String::from("seedz"),
            String::from("grape"),
        ];

        assert_eq!(
            validate_input_passwords(input_with_valid_words.clone()).unwrap(),
            input_with_valid_words,
        );

        assert_eq!(
            validate_input_passwords(input_with_invalid_words).unwrap_err(),
            InputValidationErr::NonEnglishWordFound,
        );
    }

    #[test]
    fn check_filter_matching_passwords() {
        let guess = KnownGuess::new("apple", 2);
        let pwd_start = vec!["apple", "bppef", "elppa"];
        let pwd_remaining = vec!["bppef"];

        assert_eq!(filter_matching_passwords(&guess, pwd_start), pwd_remaining);
    }
}
