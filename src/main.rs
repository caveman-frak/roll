mod dice;
mod mock;
mod parse;
mod roll;

use {
    crate::{parse::RollParser, roll::Roll},
    anyhow::Result,
    rand::thread_rng,
};

fn main() -> Result<()> {
    let mut rng = thread_rng();

    let (die, behaviour) = RollParser::foo("20d10r1")?;

    let mut roll = Roll::from_roll(&die, &mut rng);

    roll.apply(behaviour, &mut rng);

    println!("{}", roll);

    Ok(())
}
