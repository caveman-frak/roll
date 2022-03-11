use rand::{distributions::Uniform, Rng, RngCore};
use {
    anyhow::{anyhow, Error, Result},
    std::{
        fmt::{self, Display},
        ops::RangeInclusive,
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
    D00,
    Fate,
    Other(i8, i8),
}

impl Dice {
    pub fn faces(&self) -> RangeInclusive<i8> {
        match self {
            Dice::D2 => 1..=2,
            Dice::D3 => 1..=3,
            Dice::D4 => 1..=4,
            Dice::D6 => 1..=6,
            Dice::D8 => 1..=8,
            Dice::D10 => 1..=10,
            Dice::D12 => 1..=12,
            Dice::D20 => 1..=20,
            Dice::D100 => 1..=100,
            Dice::D00 => 0..=00,
            Dice::Fate => -1..=1,
            Dice::Other(start, end) => *start..=*end,
        }
    }

    pub(crate) fn critical(&self) -> Option<i8> {
        match self {
            Dice::D100 | Dice::D00 => Some(5),
            Dice::Fate => None,
            _ => Some(1),
        }
    }

    pub(crate) fn start(&self) -> Option<RangeInclusive<i8>> {
        if let Some(crit) = self.critical() {
            let start = *self.faces().start();
            Some(start..=(start + crit - 1))
        } else {
            None
        }
    }

    pub(crate) fn end(&self) -> Option<RangeInclusive<i8>> {
        if let Some(crit) = self.critical() {
            let end = *self.faces().end();
            Some(end + 1 - crit..=end)
        } else {
            None
        }
    }

    pub(crate) fn text(&self, value: i8) -> String {
        let v = self.value(value);
        match self {
            Dice::Fate => match v {
                -1 => String::from("-"),
                1 => String::from("+"),
                _ => String::from("0"),
            },
            _ => {
                if self.faces().end() < &10 {
                    format!("{}", v)
                } else if self.faces().end() > &99 {
                    format!("{:03}", v)
                } else {
                    format!("{:02}", v)
                }
            }
        }
    }

    fn value(&self, value: i8) -> i8 {
        value
    }

    pub fn roll(&self, rng: &mut dyn RngCore) -> i8 {
        rng.gen_range(self.faces())
    }

    fn parse_other(s: &str) -> Result<Dice> {
        let mut parts = s.split(':');
        match (parts.next(), parts.next()) {
            (Some(end), None) => Ok(Dice::Other(1, end.parse()?)),
            (Some(""), Some(end)) => Ok(Dice::Other(1, end.parse()?)),
            (Some(start), Some(end)) => Ok(Dice::Other(start.parse()?, end.parse()?)),
            _ => Err(anyhow!("Unable to parse {}", s)),
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
            "00" => Ok(Dice::D00),
            "%" => Ok(Dice::D00),
            "Fate" => Ok(Dice::Fate),
            "F" => Ok(Dice::Fate),
            _ => Self::parse_other(s),
        }
    }
}

impl Display for Dice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Dice::Other(1, end) => write!(f, "D{}", end),
            Dice::Other(start, end) => write!(f, "D{}:{}", start, end),
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
                    Dice::D00 => "d00",
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

    pub fn roll(&self, rng: &mut dyn RngCore) -> Vec<i8> {
        let faces = self.dice.faces();
        let range = Uniform::new_inclusive(faces.start(), faces.end());
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
        assert_eq!("00".parse::<Dice>()?, Dice::D00);
        assert_eq!("%".parse::<Dice>()?, Dice::D00);
        assert_eq!("F".parse::<Dice>()?, Dice::Fate);
        assert_eq!("Fate".parse::<Dice>()?, Dice::Fate);

        assert!(matches!("7".parse::<Dice>()?, Dice::Other(1, 7)));
        assert!(matches!(":7".parse::<Dice>()?, Dice::Other(1, 7)));
        assert!(matches!("2:7".parse::<Dice>()?, Dice::Other(2, 7)));

        assert!(matches!("S".parse::<Dice>(), Err(_)));

        Ok(())
    }

    #[test]
    fn check_d2_values() {
        assert_eq!(Dice::D2.faces(), 1..=2);
        assert_eq!(Dice::D2.text(1), "1");
        assert_eq!(Dice::D2.text(2), "2");

        assert_eq!(Dice::D2.critical(), Some(1));
        assert_eq!(Dice::D2.start(), Some(1..=1));
        assert_eq!(Dice::D2.end(), Some(2..=2));
    }

    #[test]
    fn check_fate_dice_values() {
        assert_eq!(Dice::Fate.faces(), -1..=1);
        assert_eq!(Dice::Fate.text(-1), "-");
        assert_eq!(Dice::Fate.text(0), "0");
        assert_eq!(Dice::Fate.text(1), "+");

        assert_eq!(Dice::Fate.critical(), None);
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

        assert_eq!(Dice::D10.roll(&mut rng), 1);
        assert_eq!(Dice::D10.roll(&mut rng), 2);
        assert_eq!(Dice::D10.roll(&mut rng), 3);
        assert_eq!(Dice::D10.roll(&mut rng), 4);
        assert_eq!(Dice::D10.roll(&mut rng), 5);
    }

    #[test]
    fn check_die_rolls() {
        let mut rng = rng(Dice::D100, 0);

        assert_eq!(Die::new(Dice::D100, 5).roll(&mut rng), vec![1, 2, 3, 4, 5]);
    }
}
