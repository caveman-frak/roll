use {
    anyhow::{Error, Result},
    colored::{ColoredString, Colorize},
    std::{
        fmt::{self, Display},
        str::FromStr,
    },
};

fn main() {
    println!(
        "{}\n{}\n{}\n{}\n{}",
        Roll::new(Dice::D10, 2, true),
        Roll::new(Dice::D100, 2, false),
        Roll::new(Dice::D10, 0, true),
        Roll::new(Dice::D10, 9, true),
        Roll::new(Dice::Fate, 5, true)
    );
}

#[derive(Debug, PartialEq)]
enum Dice {
    D2,
    D3,
    D4,
    D6,
    D8,
    D10,
    D12,
    D20,
    D100,
    Fate,
    Other(u32),
}

impl Dice {
    fn faces(&self) -> u32 {
        match self {
            Dice::D2 => 2,
            Dice::D3 => 3,
            Dice::D4 => 4,
            Dice::D6 => 6,
            Dice::D8 => 8,
            Dice::D10 => 10,
            Dice::D12 => 12,
            Dice::D20 => 20,
            Dice::D100 => 100,
            Dice::Fate => 6,
            Dice::Other(faces) => *faces,
        }
    }
}

impl Dice {
    fn critical_failure(&self, value: u32) -> bool {
        match self {
            Dice::D10 | Dice::D12 | Dice::D20 => value == 0,
            Dice::D100 => value < 5,
            _ => false,
        }
    }

    fn critical_success(&self, value: u32) -> bool {
        match self {
            Dice::D10 | Dice::D12 | Dice::D20 => value == self.faces() - 1,
            Dice::D100 => value >= 95,
            _ => false,
        }
    }

    fn text(&self, value: u32) -> String {
        let v = self.value(value);
        match self {
            Dice::Fate => match v {
                -1 => String::from("-"),
                1 => String::from("+"),
                _ => String::from("0"),
            },
            _ => v.to_string(),
        }
    }

    fn value(&self, value: u32) -> i32 {
        match self {
            Dice::Fate => match value {
                0 | 1 => -1,
                4 | 5 => 1,
                _ => 0,
            },
            Dice::D100 => value as i32,
            _ => value as i32 + 1,
        }
    }
}

impl FromStr for Dice {
    type Err = Error;

    fn from_str(s: &str) -> Result<Dice> {
        match s {
            "2" => Ok(Dice::D2),
            "3" => Ok(Dice::D3),
            "4" => Ok(Dice::D4),
            "6" => Ok(Dice::D6),
            "8" => Ok(Dice::D8),
            "10" => Ok(Dice::D10),
            "12" => Ok(Dice::D12),
            "20" => Ok(Dice::D20),
            "100" => Ok(Dice::D100),
            "%" => Ok(Dice::D100),
            "Fate" => Ok(Dice::Fate),
            "F" => Ok(Dice::Fate),
            _ => Ok(Dice::Other(u32::from_str(s)?)),
        }
    }
}

#[derive(Debug, PartialEq)]
struct Roll {
    dice: Dice,
    value: u32,
    included: bool,
}

impl Roll {
    fn new(dice: Dice, value: u32, included: bool) -> Self {
        Self {
            dice,
            value,
            included,
        }
    }

    fn failure(&self) -> bool {
        self.dice.critical_failure(self.value)
    }

    fn success(&self) -> bool {
        self.dice.critical_success(self.value)
    }

    fn included(&self) -> bool {
        self.included
    }

    fn text(&self) -> ColoredString {
        let s = self.dice.text(self.value);

        let c = if self.failure() {
            s.red()
        } else if self.success() {
            s.green()
        } else {
            s.normal()
        };

        if !self.included() {
            c.strikethrough()
        } else {
            c
        }
    }
}

impl Display for Roll {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text())
    }
}

#[derive(Debug, PartialEq)]
struct Die {
    count: u32,
    dice: Dice,
}
