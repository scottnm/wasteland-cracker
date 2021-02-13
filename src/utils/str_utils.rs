pub fn matching_char_count_ignore_case(a: &str, b: &str) -> usize {
    assert_eq!(a.len(), b.len());

    fn chars_eq_ignore_case((a, b): &(char, char)) -> bool {
        a.to_ascii_lowercase() == b.to_ascii_lowercase()
    }

    a.chars()
        .zip(b.chars())
        .filter(chars_eq_ignore_case)
        .count()
}

pub fn hamming_dist_ignore_case(a: &str, b: &str) -> usize {
    assert_eq!(a.len(), b.len());
    a.len() - matching_char_count_ignore_case(a, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matching_char_count_ignore_case() {
        assert_eq!(matching_char_count_ignore_case("apple", "APpLe"), 5);
        assert_eq!(matching_char_count_ignore_case("apple", "loodo"), 0);
        assert_eq!(matching_char_count_ignore_case("upper", "APpLe"), 2);
    }

    #[test]
    fn test_hamming_distance() {
        assert_eq!(hamming_dist_ignore_case("apple", "APpLe"), 0);
        assert_eq!(hamming_dist_ignore_case("apple", "loodo"), 5);
        assert_eq!(hamming_dist_ignore_case("upper", "APpLe"), 3);
    }
}
