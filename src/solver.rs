use crate::dict;

use crate::utils::{keys, Rect};

#[derive(Debug, PartialEq, Eq)]
enum InputValidationErr {
    InputEmpty,
    InvalidPasswordLengthFound,
    PasswordNotFoundInEnglishDict,
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

    let dict = dict::EnglishDictChunk::load(required_len);

    let all_valid_words = pwds.iter().all(|p| dict.is_word(&p));
    if !all_valid_words {
        return Err(InputValidationErr::PasswordNotFoundInEnglishDict);
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
        let matching_count =
            crate::utils::matching_char_count_ignore_case(passwords[i].as_ref(), &guess.word);
        if matching_count != guess.char_count {
            passwords.swap_remove(i);
        }
    }
    passwords
}

pub fn solver(password_file: &str, guess_args: &[String], window: &pancurses::Window) {
    let input_passwords = {
        let pwds: Vec<String> = snm_simple_file::read_lines(&password_file).collect();
        match validate_input_passwords(pwds) {
            Ok(validated_pwds) => validated_pwds,
            Err(e) => panic!("Input failed validation: {:?}", e),
        }
    };

    let known_guesses = {
        let mut known_guesses = Vec::new();
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

    /* FIXME:
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
    */

    let mut menu_cursor: i32 = 0;
    let cursor_prefix = "> ";
    let word_column_width = remaining_passwords.iter().map(|p| p.len()).max().unwrap() as i32;
    let padding_width = 4;
    let char_count_column_width = 2; // 00

    let menu_rect = {
        let menu_width = (cursor_prefix.len() as i32)
            + word_column_width
            + padding_width
            + char_count_column_width;
        let menu_height = remaining_passwords.len() as i32;

        Rect {
            // center the menu options horizontally
            left: (window.get_max_x() - menu_width) / 2,
            // center the menu options vertically
            top: (window.get_max_y() - menu_height) / 2,
            width: menu_width,
            height: menu_height,
        }
    };

    loop {
        // Input handling
        // TODO: I think this input system might need some refactoring to share with the start menu
        if let Some(pancurses::Input::Character(ch)) = window.getch() {
            match ch {
                // check for movement inputs
                'w' => menu_cursor = std::cmp::max(0, menu_cursor - 1),
                's' => {
                    menu_cursor = std::cmp::min(remaining_passwords.len() as i32, menu_cursor + 1)
                }
                keys::ASCII_ESC => break,
                _ => (),
            }
        };

        window.erase();

        for (i, pwd) in remaining_passwords.iter().enumerate() {
            let row = i as i32 + menu_rect.top;
            let col_offset = menu_rect.left + cursor_prefix.len() as i32;
            window.mvaddstr(row, col_offset, pwd);
            window.mvaddstr(
                row,
                col_offset + menu_rect.width - char_count_column_width,
                "00",
            );
        }

        let back_button_row = menu_rect.top + (remaining_passwords.len() + 2) as i32;
        let back_button_text = "[ Back ]";
        window.mvaddstr(
            back_button_row,
            menu_rect.left + cursor_prefix.len() as i32,
            back_button_text,
        );

        if (menu_cursor as usize) < remaining_passwords.len() {
            window.mvaddstr(menu_rect.top + menu_cursor, menu_rect.left, cursor_prefix);
        } else {
            assert_eq!(menu_cursor as usize, remaining_passwords.len());
            window.mvchgat(
                back_button_row,
                menu_rect.left + cursor_prefix.len() as i32,
                back_button_text.len() as i32,
                pancurses::A_BLINK,
                0,
            );
        }

        window.refresh();

        // No need to waste cycles doing nothing but rendering over and over.
        // Yield the processor until the next frame.
        std::thread::sleep(std::time::Duration::from_millis(33));
    }

    /* FIXME:
    match &remaining_passwords[..] {
        [] => println!("No solution matched the provided guesses!"),
        [solution] => println!("The password is... {}", solution),
        _ => (),
    };
    */
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
            InputValidationErr::PasswordNotFoundInEnglishDict,
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
