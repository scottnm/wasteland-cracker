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

pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
}
