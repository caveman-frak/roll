mod dice;
mod mock;
mod roll;

use {
    crate::{
        dice::{Dice, Die},
        roll::{
            behaviour::{Behaviour, DiscardDirection},
            value::ExType,
            Roll,
        },
    },
    rand::thread_rng,
};

fn main() {
    let mut rng = thread_rng();
    for _ in 0..10 {
        println!(
            "{} ",
            Roll::from_roll(&Die::new(Dice::D10, 10), &mut rng).apply(
                vec![
                    // Behaviour::Reroll(None, false),
                    // Behaviour::Explode(None, ExType::Standard),
                    Behaviour::Critical(None, None),
                    Behaviour::Drop(2, DiscardDirection::Low),
                ],
                &mut rng
            )
        );
    }
    println!();
}
