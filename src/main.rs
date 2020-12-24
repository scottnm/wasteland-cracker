#[macro_use]
extern crate lazy_static;

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
            words: input_helpers::read_lines("src/words_alpha.txt").collect(),
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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let file = &args[1];
    let guesses = &args[2..];

    let input_passwords = {
        let pwds: Vec<String> = input_helpers::read_lines(&file).collect();
        match validate_input_passwords(pwds) {
            Ok(validated_pwds) => validated_pwds,
            Err(e) => panic!("Input failed validation: {:?}", e),
        }
    };

    for guess in guesses {
        if input_passwords.iter().filter(|l| *l == guess).count() != 1usize {
            panic!("{} was not found in password list!", guess);
        }
    }

    let possible_passwords = input_passwords
        .iter()
        .filter(|l| guesses.iter().all(|g| *l != g));

    println!("Matching {:?}... ", &guesses);
    for possible_password in possible_passwords {
        print!("    {} = ", possible_password);
        for guess in guesses {
            let matching_count = possible_password
                .chars()
                .zip(guess.chars())
                .filter(|(a, b)| a == b)
                .count();
            print!("{} ", matching_count);
        }
        println!();
    }
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
}
