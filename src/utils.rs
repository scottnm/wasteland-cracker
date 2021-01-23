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

impl Rect {
    pub const fn _right(&self) -> i32 {
        let right = self.left + self.width - 1;
        right
    }

    pub const fn _bottom(&self) -> i32 {
        let bottom = self.top + self.height - 1;
        bottom
    }

    pub const fn _center_x(&self) -> i32 {
        self.left + self.width / 2
    }

    pub const fn _center_y(&self) -> i32 {
        self.top + self.height / 2
    }
}
