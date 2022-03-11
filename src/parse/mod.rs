use {
    crate::{dice::Die, roll::behaviour::Behaviour},
    anyhow::Result,
    pest::Parser,
    pest_derive::Parser,
};

#[derive(Parser)]
#[grammar = "parse/roll.pest"]
pub struct RollParser {}

impl RollParser {
    pub fn roll(s: &str) -> Result<(Die, Vec<Behaviour>)> {
        let mut roll = RollParser::parse(Rule::roll, s)?;
        let mut die: Option<Die> = None;
        let mut behaviours: Vec<Behaviour> = Vec::new();

        let r = roll.next().unwrap();

        for record in r.into_inner() {
            match record.as_rule() {
                Rule::die => die = Some(record.as_str().parse()?),
                _ => behaviours.push(record.as_str().parse()?),
            }
        }

        Ok((die.unwrap(), behaviours))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_parse_dice() {
        let result = RollParser::parse(Rule::roll, "d1:8");

        println!("{:?}", result);
        assert!(matches!(result, Ok(_)));
    }
}
