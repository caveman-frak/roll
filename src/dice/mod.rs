use rand::{distributions::Uniform, Rng, RngCore};
use {
    anyhow::{Error, Result},
    std::{
        fmt::{self, Display},
        str::FromStr,
    },
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Dice {
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
    Other(u8),
}

impl Dice {
    pub fn faces(&self) -> u8 {
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

    pub(crate) fn failure(&self) -> Option<u8> {
        match self {
            Dice::D100 => Some(4),
            Dice::Fate => None,
            _ => Some(0),
        }
    }

    pub(crate) fn success(&self) -> Option<u8> {
        match self {
            Dice::D100 => Some(95),
            Dice::Fate => None,
            _ => Some(self.faces() - 1),
        }
    }

    pub(crate) fn text(&self, value: u8) -> String {
        let v = self.value(value);
        match self {
            Dice::Fate => match v {
                -1 => String::from("-"),
                1 => String::from("+"),
                _ => String::from("0"),
            },
            _ => {
                if self.faces() < 10 {
                    format!("{}", v)
                } else {
                    format!("{:02}", v)
                }
            }
        }
    }

    fn value(&self, value: u8) -> i8 {
        match self {
            Dice::Fate => match value {
                0 | 1 => -1,
                4 | 5 => 1,
                _ => 0,
            },
            Dice::D100 => value as i8,
            _ => value as i8 + 1,
        }
    }

    pub fn roll(&self, rng: &mut dyn RngCore) -> u8 {
        rng.gen_range(0..self.faces())
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
            _ => Ok(Dice::Other(s.parse()?)),
        }
    }
}

impl Display for Dice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Dice::Other(faces) => write!(f, "D{}", faces),
            _ => write!(
                f,
                "{}",
                match self {
                    Dice::D2 => "d2",
                    Dice::D3 => "d3",
                    Dice::D4 => "d4",
                    Dice::D6 => "d6",
                    Dice::D8 => "d8",
                    Dice::D10 => "d10",
                    Dice::D12 => "d12",
                    Dice::D20 => "d20",
                    Dice::D100 => "d100",
                    Dice::Fate => "Fate",
                    _ => "",
                }
            ),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Die {
    dice: Dice,
    count: u8,
}

impl Die {
    pub(crate) fn new(dice: Dice, count: u8) -> Self {
        Self { dice, count }
    }

    pub(crate) fn dice(&self) -> &Dice {
        &self.dice
    }

    pub fn roll(&self, rng: &mut dyn RngCore) -> Vec<u8> {
        let range = Uniform::new(0, self.dice.faces());
        rng.sample_iter(range).take(self.count as usize).collect()
    }
}

impl FromStr for Die {
    type Err = Error;

    fn from_str(s: &str) -> Result<Die> {
        let mut parts = s.split('d');

        let count: u8 = parts.next().unwrap_or("1").parse().unwrap_or(1);
        let dice: Dice = parts.next().unwrap_or("").parse()?;
        Ok(Die::new(dice, count))
    }
}

impl Display for Die {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.count, self.dice)
    }
}

#[cfg(test)]
mod test {
    use {super::*, crate::mock::rng::*};

    #[test]
    fn check_dice_parsing() -> Result<()> {
        assert_eq!("2".parse::<Dice>()?, Dice::D2);
        assert_eq!("3".parse::<Dice>()?, Dice::D3);
        assert_eq!("4".parse::<Dice>()?, Dice::D4);
        assert_eq!("6".parse::<Dice>()?, Dice::D6);
        assert_eq!("8".parse::<Dice>()?, Dice::D8);
        assert_eq!("10".parse::<Dice>()?, Dice::D10);
        assert_eq!("12".parse::<Dice>()?, Dice::D12);
        assert_eq!("20".parse::<Dice>()?, Dice::D20);
        assert_eq!("100".parse::<Dice>()?, Dice::D100);
        assert_eq!("%".parse::<Dice>()?, Dice::D100);
        assert_eq!("F".parse::<Dice>()?, Dice::Fate);
        assert_eq!("Fate".parse::<Dice>()?, Dice::Fate);

        assert!(matches!("1".parse::<Dice>()?, Dice::Other(1)));
        assert!(matches!("5".parse::<Dice>()?, Dice::Other(5)));

        assert!(matches!("S".parse::<Dice>(), Err(_)));

        Ok(())
    }

    #[test]
    fn check_d2_values() {
        assert_eq!(Dice::D2.faces(), 2);
        assert_eq!(Dice::D2.value(0), 1);
        assert_eq!(Dice::D2.value(1), 2);

        assert_eq!(Dice::D2.failure(), Some(0));
        assert_eq!(Dice::D2.success(), Some(1));
    }

    #[test]
    fn check_fate_dice_values() {
        assert_eq!(Dice::Fate.faces(), 6);
        assert_eq!(Dice::Fate.value(0), -1);
        assert_eq!(Dice::Fate.value(1), -1);
        assert_eq!(Dice::Fate.value(2), 0);
        assert_eq!(Dice::Fate.value(3), 0);
        assert_eq!(Dice::Fate.value(4), 1);
        assert_eq!(Dice::Fate.value(5), 1);

        assert_eq!(Dice::Fate.failure(), None);
        assert_eq!(Dice::Fate.success(), None);
    }

    #[test]
    fn check_die_parsing() -> Result<()> {
        assert_eq!("d10".parse::<Die>()?, Die::new(Dice::D10, 1));
        assert_eq!("1d10".parse::<Die>()?, Die::new(Dice::D10, 1));
        assert_eq!("2d10".parse::<Die>()?, Die::new(Dice::D10, 2));

        assert!(matches!("2d".parse::<Die>(), Err(_)));
        assert!(matches!("d".parse::<Die>(), Err(_)));
        assert!(matches!("2".parse::<Die>(), Err(_)));

        Ok(())
    }

    #[test]
    fn check_dice_roll() {
        let mut rng = rng(Dice::D10, 0);

        assert_eq!(Dice::D10.roll(&mut rng), 0);
        assert_eq!(Dice::D10.roll(&mut rng), 1);
        assert_eq!(Dice::D10.roll(&mut rng), 2);
        assert_eq!(Dice::D10.roll(&mut rng), 3);
        assert_eq!(Dice::D10.roll(&mut rng), 4);
    }

    #[test]
    fn check_die_rolls() {
        let mut rng = rng(Dice::D100, 0);

        assert_eq!(Die::new(Dice::D100, 5).roll(&mut rng), vec![0, 1, 2, 3, 4]);
    }
}
