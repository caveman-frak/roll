#[cfg(test)]
pub(crate) mod rng {
    use {
        crate::dice::Dice,
        rand::{rngs::mock::StepRng, Error, RngCore},
        std::iter::Cycle,
    };

    fn increment(dice: Dice) -> u64 {
        1 + (u32::MAX / dice.faces() as u32) as u64
    }

    pub(crate) fn step_rng(dice: Dice, start: u64, step: u64) -> impl RngCore {
        let increment = increment(dice);
        StepRng::new(start * increment, increment * step)
    }

    pub(crate) fn rng(dice: Dice, start: u64) -> impl RngCore {
        step_rng(dice, start, 1)
    }

    pub(crate) fn seq_rng<'a, S: Iterator<Item = u64> + Sized + Clone + 'a>(
        dice: Dice,
        sequence: S,
    ) -> impl RngCore + 'a {
        SeqRng::new(increment(dice), sequence)
    }

    struct SeqRng<S: Iterator<Item = u64> + Sized + Clone> {
        increment: u64,
        sequence: Cycle<S>,
    }

    impl<S: Iterator<Item = u64> + Sized + Clone> SeqRng<S> {
        pub fn new(increment: u64, sequence: S) -> Self {
            SeqRng {
                increment,
                sequence: sequence.cycle(),
            }
        }
    }

    impl<S: Iterator<Item = u64> + Sized + Clone> RngCore for SeqRng<S> {
        #[inline]
        fn next_u32(&mut self) -> u32 {
            self.next_u64() as u32
        }

        #[inline]
        fn next_u64(&mut self) -> u64 {
            self.sequence.next().unwrap_or_default() * self.increment
        }

        #[inline]
        fn fill_bytes(&mut self, dest: &mut [u8]) {
            let mut left = dest;
            while left.len() >= 4 {
                let (l, r) = { left }.split_at_mut(4);
                left = r;
                let chunk: [u8; 4] = self.next_u32().to_le_bytes();
                l.copy_from_slice(&chunk);
            }
            let n = left.len();
            if n > 0 {
                let chunk: [u8; 4] = self.next_u32().to_le_bytes();
                left.copy_from_slice(&chunk[..n]);
            }
        }

        #[inline]
        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
            self.fill_bytes(dest);
            Ok(())
        }
    }

    mod test {
        use {super::*, crate::dice::Dice::*};

        #[test]
        fn print_dice_rng() {
            for dice in [D2, D3, D4, D6, D8, D10, D12, D20] {
                println!(">>>>> {} ", dice);
                for i in 0..dice.faces() {
                    let mut rng = rng(dice, i as u64);
                    for _ in 0..30 {
                        print!("{:02} ", dice.roll(&mut rng));
                    }
                    println!();
                }
            }
        }

        #[test]
        fn print_dice_rng_step() {
            for dice in [D6] {
                println!(">>>>> Step {} ", dice);
                for i in 0..dice.faces() {
                    let mut rng = step_rng(dice, i as u64, 2);
                    for _ in 0..30 {
                        print!("{:02} ", dice.roll(&mut rng));
                    }
                    println!();
                }
            }
        }

        #[test]
        fn print_dice_rng_seq() {
            for dice in [D6] {
                println!(">>>>> Seq {} ", dice);
                let v: Vec<u64> = vec![1, 2, 3, 4, 5, 6, 7];
                for _ in 0..dice.faces() {
                    let mut rng = seq_rng(dice, v.iter().cloned());
                    for _ in 0..30 {
                        print!("{:02} ", dice.roll(&mut rng));
                    }
                    println!();
                }
            }
        }
    }
}
