mod dice;
mod mock;
mod roll;

use {
    crate::{
        dice::{Dice, Die},
        roll::Roll,
    },
    rand::thread_rng,
};

fn main() {
    println!(
        "{}",
        Roll::new(&Die::new(Dice::D10, 2), vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
    );

    let mut rng = thread_rng();
    for _ in 0..10 {
        print!("{:?} ", Dice::D10.roll(&mut rng));
    }
    println!();
    println!();
    for _ in 0..10 {
        print!("{:?} ", Die::new(Dice::D10, 2).roll(&mut rng));
    }
    println!();
}
