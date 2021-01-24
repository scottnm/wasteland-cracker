pub fn matching_char_count_ignore_case(a: &str, b: &str) -> usize {
    fn chars_eq_ignore_case((a, b): &(char, char)) -> bool {
        a.to_ascii_lowercase() == b.to_ascii_lowercase()
    }

    a.chars()
        .zip(b.chars())
        .filter(chars_eq_ignore_case)
        .count()
}

pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
}
