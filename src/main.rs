#[derive(Debug, PartialEq, Eq)]
enum InputValidationErr {
    InputEmpty,
    InvalidPasswordLengthFound,
    NonEnglishWordFound,
}

fn validate_input_passwords(pwds: Vec<String>) -> Result<Vec<String>, InputValidationErr> {
    if pwds.len() == 0 {
        return Err(InputValidationErr::InputEmpty);
    }

    let required_len = pwds[0].len();
    let equal_len = pwds.iter().all(|a| a.len() == required_len);
    if !equal_len {
        return Err(InputValidationErr::InvalidPasswordLengthFound);
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
}
