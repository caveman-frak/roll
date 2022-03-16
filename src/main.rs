mod cli;
mod dice;
mod mock;
mod parse;
mod roll;

use {
    crate::{cli::Args, parse::RollParser, roll::Roll},
    anyhow::Result,
    clap::Parser,
    rand::thread_rng,
};

fn main() -> Result<()> {
    let args = Args::parse();
    let mut rng = thread_rng();

    let (die, behaviour) = RollParser::roll(args.content().unwrap_or("20d10r1"))?;

    let mut roll = Roll::from_roll(&die, &mut rng);

    roll.apply(behaviour, &mut rng);

    println!("{}", roll);

    Ok(())
}
