use rand::Rng;

pub trait RangeRng<T: PartialOrd> {
    fn gen_range(&mut self, lower: T, upper: T) -> T;
}

pub fn select_rand<'a, T>(seq: &'a [T], rng: &mut dyn RangeRng<usize>) -> &'a T {
    let index = rng.gen_range(0, seq.len());
    &seq[index]
}

pub struct ThreadRangeRng {
    rng: rand::rngs::ThreadRng,
}

impl ThreadRangeRng {
    pub fn new() -> ThreadRangeRng {
        ThreadRangeRng {
            rng: rand::thread_rng(),
        }
    }
}

impl<T: PartialOrd + rand::distributions::uniform::SampleUniform> RangeRng<T> for ThreadRangeRng {
    fn gen_range(&mut self, lower: T, upper: T) -> T {
        self.rng.gen_range(lower, upper)
    }
}

#[cfg(test)]
pub mod mocks {
    use super::*;

    pub struct SingleValueRangeRng<T: PartialOrd + Copy> {
        value: T,
    }

    pub struct SequenceRangeRng<T: PartialOrd + Copy> {
        next: usize,
        seq: Vec<T>,
    }

    impl<T: PartialOrd + Copy> SingleValueRangeRng<T> {
        pub fn new(value: T) -> SingleValueRangeRng<T> {
            SingleValueRangeRng { value }
        }
    }

    impl<T: PartialOrd + Copy> RangeRng<T> for SingleValueRangeRng<T> {
        fn gen_range(&mut self, lower: T, upper: T) -> T {
            assert!(lower <= self.value);
            assert!(upper > self.value);
            self.value
        }
    }

    impl<T: PartialOrd + Copy> SequenceRangeRng<T> {
        pub fn new(value: &[T]) -> SequenceRangeRng<T> {
            SequenceRangeRng {
                next: 0,
                seq: Vec::from(value),
            }
        }
    }

    impl<T: PartialOrd + Copy> RangeRng<T> for SequenceRangeRng<T> {
        fn gen_range(&mut self, lower: T, upper: T) -> T {
            let value = self.seq[self.next];
            self.next = (self.next + 1) % self.seq.len();

            assert!(lower <= value);
            assert!(upper > value);
            value
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gen_wrapper<T: PartialOrd>(rng: &mut dyn RangeRng<T>, lower: T, upper: T) -> T {
        rng.gen_range(lower, upper)
    }

    #[test]
    fn test_thread_random() {
        // this test is mostly here to verify that things compile
        let mut rng = ThreadRangeRng::new();
        let first_value = rng.gen_range(0, 10);
        let next_value = gen_wrapper(&mut rng, 10, 20);
        assert_ne!(first_value, next_value);
    }

    #[test]
    fn test_single_value_random() {
        let mut rng = mocks::SingleValueRangeRng::new(10i32);
        let first_value = rng.gen_range(0, 100);
        for _ in 1..10 {
            let next_value = gen_wrapper(&mut rng, 0, 100);
            assert_eq!(first_value, next_value);
        }
    }
}
