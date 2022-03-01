pub mod behaviour;

use {
    crate::{
        dice::{Dice, Die},
        roll::behaviour::Value,
    },
    joinery::{separators::Space, JoinableIterator},
    rand::RngCore,
    std::fmt::{self, Display},
};

#[derive(Debug, PartialEq)]
pub struct Roll<'a> {
    die: &'a Die,
    values: Vec<Value>,
}

impl<'a> Roll<'a> {
    pub fn new(die: &'a Die, values: Vec<u8>) -> Self {
        Self {
            die,
            values: values.iter().map(|v| Value::new(*v)).collect(),
        }
    }

    pub fn roll(die: &'a Die, rng: &mut dyn RngCore) -> Self {
        let values = die.roll(rng);
        Self::new(die, values)
    }

    fn dice(&self) -> &Dice {
        self.die.dice()
    }

    fn text(&self) -> String {
        self.values
            .iter()
            .map(|v| v.text(self.dice()))
            .join_with(Space)
            .to_string()
    }
}

impl<'a> Display for Roll<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text())
    }
}
