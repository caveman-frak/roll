use {
    crate::{
        dice::{bound::Bounded, Dice},
        roll::value::{Action, ExType, Value},
    },
    anyhow::{anyhow, Error, Result},
    rand::RngCore,
    std::{iter::Iterator, ops::RangeBounds, str::FromStr},
};

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub enum DiscardDirection {
    High,
    Low,
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub enum DiscardType {
    Keep(DiscardDirection),
    Drop(DiscardDirection),
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub enum Behaviour {
    Reroll(Option<Bounded>, bool),
    Explode(Option<Bounded>, ExType),
    Critical(Option<Bounded>, Option<Bounded>),
    Keep(usize, DiscardDirection),
    Drop(usize, DiscardDirection),
}

impl Behaviour {
    pub fn apply(
        behaviour: Behaviour,
        dice: &Dice,
        values: Vec<Value>,
        rng: &mut dyn RngCore,
    ) -> Vec<Value> {
        match behaviour {
            Self::Keep(number, direction) => {
                Self::apply_discard(number, DiscardType::Keep(direction), values)
            }
            Self::Drop(number, direction) => {
                Self::apply_discard(number, DiscardType::Drop(direction), values)
            }
            Self::Reroll(point, repeat) => Self::apply_reroll(point, repeat, dice, values, rng),
            Self::Explode(point, explode) => Self::apply_explode(point, explode, dice, values, rng),
            Self::Critical(failure, success) => {
                Self::apply_critical(failure, success, dice, values)
            }
        }
    }

    pub fn apply_all(
        behaviours: Vec<Behaviour>,
        dice: &Dice,
        values: Vec<Value>,
        rng: &mut dyn RngCore,
    ) -> Vec<Value> {
        let mut values = values;
        let mut behaviours = behaviours;
        behaviours.sort_unstable();
        for behaviour in behaviours {
            values = Self::apply(behaviour, dice, values, rng);
        }
        values
    }

    fn failure(dice: &Dice, point: Option<Bounded>) -> Option<Bounded> {
        point.or_else(|| dice.start())
    }

    fn success(dice: &Dice, point: Option<Bounded>) -> Option<Bounded> {
        point.or_else(|| dice.end())
    }

    fn apply_reroll(
        point: Option<Bounded>,
        repeat: bool,
        dice: &Dice,
        values: Vec<Value>,
        rng: &mut dyn RngCore,
    ) -> Vec<Value> {
        if let Some(range) = Self::failure(dice, point) {
            let mut result = Vec::new();
            for value in values {
                let mut v = value;
                while range.contains(&v.value()) {
                    v = v.clone().update(dice.roll(rng), Action::Reroll(v.value()));
                    if !repeat {
                        break;
                    }
                }
                result.push(v);
            }
            result
        } else {
            values
        }
    }

    fn apply_critical(
        failure: Option<Bounded>,
        success: Option<Bounded>,
        dice: &Dice,
        values: Vec<Value>,
    ) -> Vec<Value> {
        let failure = Self::failure(dice, failure);
        let success = Self::success(dice, success);
        let mut result = Vec::new();
        for value in values {
            result.push(match (&failure, &success) {
                (Some(v), _) if v.contains(&value.value()) => value.add(Action::Failure),
                (_, Some(v)) if v.contains(&value.value()) => value.add(Action::Success),
                _ => value,
            });
        }
        result
    }

    fn apply_discard(number: usize, discard: DiscardType, values: Vec<Value>) -> Vec<Value> {
        let mut numbers: Vec<i8> = values
            .iter()
            .filter(|v| !v.actions().contains(&Action::Discard))
            .map(|v| v.value())
            .collect();
        numbers.sort_unstable();
        if let DiscardType::Drop(DiscardDirection::High)
        | DiscardType::Keep(DiscardDirection::High) = discard
        {
            numbers.reverse();
        }
        let mut discards = if let DiscardType::Drop(_) = discard {
            numbers[..number].to_vec()
        } else {
            numbers[number..].to_vec()
        };
        discards.sort_unstable();
        let mut results = Vec::new();
        for value in values {
            results.push(if value.actions().contains(&Action::Discard) {
                value
            } else if let Ok(index) = discards.binary_search(&value.value()) {
                discards.remove(index);
                value.add(Action::Discard)
            } else {
                value
            });
        }
        results
    }

    fn apply_explode(
        point: Option<Bounded>,
        explode: ExType,
        dice: &Dice,
        values: Vec<Value>,
        rng: &mut dyn RngCore,
    ) -> Vec<Value> {
        if let Some(range) = Self::success(dice, point) {
            let mut result = Vec::new();
            for value in values {
                let mut first = true;
                let mut v = value;
                let mut r = v.value();
                while range.contains(&r) {
                    r = dice.roll(rng);
                    v = match explode {
                        ExType::Standard => {
                            v.clone().update(r, Action::Explode(v.value(), explode))
                        }
                        ExType::Penetrating => {
                            r = (r - 1).max(*dice.faces().start());
                            v.clone().update(r, Action::Explode(v.value(), explode))
                        }
                        ExType::Compound => {
                            if first {
                                v = v.clone().add(Action::Explode(v.value(), explode));
                            }
                            v.clone().update(v.value() + r, Action::Explode(r, explode))
                        }
                    };
                    first = false;
                }
                result.push(v);
            }
            result
        } else {
            values
        }
    }

    fn parse_reroll(s: &str) -> Result<Behaviour> {
        let point = if s.is_empty() {
            None
        } else {
            Some(Bounded::range_to(s.parse()?))
        };
        Ok(Behaviour::Reroll(point, true))
    }

    fn parse_explode(s: &str) -> Result<Behaviour> {
        if s.is_empty() {
            Ok(Behaviour::Explode(None, ExType::Standard))
        } else {
            let (range, extype) = match &s[..1] {
                "!" | "c" => (
                    if s[1..].is_empty() {
                        None
                    } else {
                        Some(Bounded::range_from(s[1..].parse()?))
                    },
                    ExType::Compound,
                ),
                "p" => (
                    if s[1..].is_empty() {
                        None
                    } else {
                        Some(Bounded::range_from(s[1..].parse()?))
                    },
                    ExType::Penetrating,
                ),
                "" => (None, ExType::Standard),
                _ => (Some(Bounded::range_from(s.parse()?)), ExType::Standard),
            };
            Ok(Behaviour::Explode(range, extype))
        }
    }

    fn parse_critical(s: &str) -> Result<Behaviour> {
        match &s[..1] {
            "s" => Ok(Behaviour::Critical(
                None,
                if s[1..].is_empty() {
                    None
                } else {
                    Some(Bounded::range_from(s[1..].parse()?))
                },
            )),
            "f" => Ok(Behaviour::Critical(
                if s[1..].is_empty() {
                    None
                } else {
                    Some(Bounded::range_to(s[1..].parse()?))
                },
                None,
            )),
            _ => Err(anyhow!("Unable to parse Critical Behaviour '{}'", s)),
        }
    }

    fn parse_keep(s: &str) -> Result<Behaviour> {
        let (number, direction) = match &s[..1] {
            "h" => (s[1..].parse()?, DiscardDirection::High),
            "l" => (s[1..].parse()?, DiscardDirection::Low),
            _ => (s.parse()?, DiscardDirection::High),
        };
        Ok(Behaviour::Keep(number, direction))
    }

    fn parse_drop(s: &str) -> Result<Behaviour> {
        let (number, direction) = match &s[..1] {
            "h" => (s[1..].parse()?, DiscardDirection::High),
            "l" => (s[1..].parse()?, DiscardDirection::Low),
            _ => (s.parse()?, DiscardDirection::Low),
        };
        Ok(Behaviour::Drop(number, direction))
    }
}

impl FromStr for Behaviour {
    type Err = Error;

    fn from_str(s: &str) -> Result<Behaviour> {
        match &s[..1] {
            "r" => Ok(Self::parse_reroll(&s[1..])?),
            "!" | "x" => Ok(Self::parse_explode(&s[1..])?),
            "c" if s.len() > 1 => Ok(Self::parse_critical(&s[1..])?),
            "k" => Ok(Self::parse_keep(&s[1..])?),
            "d" => Ok(Self::parse_drop(&s[1..])?),
            _ => Err(anyhow!("Unable to parse {} as Behaviour", s)),
        }
    }
}

#[cfg(test)]
mod test {
    use {
        super::*,
        crate::{mock::rng::*, roll::value::test::*},
    };

    #[test]
    fn check_apply_critical() {
        let values = values(vec![1, 2, 3, 4, 5, 6]);

        let result = Behaviour::apply_critical(None, None, &Dice::D6, values);

        assert_eq!(
            action(&result),
            vec![
                Some(Action::Failure),
                None,
                None,
                None,
                None,
                Some(Action::Success)
            ]
        );
    }

    #[test]
    fn check_apply_critical_override() {
        let values = values(vec![1, 2, 3, 4, 5, 6]);

        let result = Behaviour::apply_critical(
            Some(Bounded::from_range(..=2)),
            Some(Bounded::from_range(5..)),
            &Dice::D6,
            values,
        );

        assert_eq!(
            action(&result),
            vec![
                Some(Action::Failure),
                Some(Action::Failure),
                None,
                None,
                Some(Action::Success),
                Some(Action::Success)
            ]
        );
    }

    #[test]
    fn check_apply_reroll_once() {
        let mut rng = rng(Dice::D6, 1);
        let values = values(vec![1, 2, 3, 4, 5, 6]);

        let result = Behaviour::apply_reroll(None, false, &Dice::D6, values, &mut rng);

        assert_eq!(result.len(), 6);
        assert_eq!(result[0].value(), 2);

        assert_eq!(
            action(&result),
            vec![Some(Action::Reroll(1)), None, None, None, None, None,]
        );
    }

    #[test]
    fn check_apply_reroll() {
        let mut rng = rng(Dice::D6, 0);
        let values = values(vec![1, 2, 3, 4, 5, 6]);

        let result = Behaviour::apply_reroll(None, true, &Dice::D6, values, &mut rng);

        assert_eq!(result.len(), 6);
        assert_eq!(result[0].value(), 2);

        let actions = actions(&result);

        assert_eq!(actions[0], vec![Action::Reroll(1), Action::Reroll(1)]);
        assert!(actions[1].is_empty());
        assert!(actions[2].is_empty());
        assert!(actions[3].is_empty());
        assert!(actions[4].is_empty());
        assert!(actions[5].is_empty());
    }

    #[test]
    fn check_apply_discard_keep_high() {
        let values = values(vec![1, 2, 3, 4, 5, 6]);

        let result = Behaviour::apply_discard(4, DiscardType::Keep(DiscardDirection::High), values);

        assert_eq!(
            action(&result),
            vec![
                Some(Action::Discard),
                Some(Action::Discard),
                None,
                None,
                None,
                None,
            ]
        );
    }

    #[test]
    fn check_apply_discard_keep_low() {
        let values = values(vec![1, 2, 3, 4, 5, 6]);

        let result = Behaviour::apply_discard(4, DiscardType::Keep(DiscardDirection::Low), values);

        assert_eq!(
            action(&result),
            vec![
                None,
                None,
                None,
                None,
                Some(Action::Discard),
                Some(Action::Discard),
            ]
        );
    }

    #[test]
    fn check_apply_discard_keep_duplicates() {
        let values = values(vec![2, 2, 2, 2, 4, 5]);

        let result = Behaviour::apply_discard(4, DiscardType::Keep(DiscardDirection::High), values);

        assert_eq!(
            action(&result),
            vec![
                Some(Action::Discard),
                Some(Action::Discard),
                None,
                None,
                None,
                None,
            ]
        );
    }

    #[test]
    fn check_apply_discard_drop_low() {
        let values = values(vec![1, 2, 3, 4, 5, 6]);

        let result = Behaviour::apply_discard(2, DiscardType::Drop(DiscardDirection::Low), values);

        assert_eq!(
            action(&result),
            vec![
                Some(Action::Discard),
                Some(Action::Discard),
                None,
                None,
                None,
                None,
            ]
        );
    }

    #[test]
    fn check_apply_discard_drop_high() {
        let values = values(vec![1, 2, 3, 4, 5, 6]);

        let result = Behaviour::apply_discard(2, DiscardType::Drop(DiscardDirection::High), values);

        assert_eq!(
            action(&result),
            vec![
                None,
                None,
                None,
                None,
                Some(Action::Discard),
                Some(Action::Discard),
            ]
        );
    }

    #[test]
    fn check_apply_discard_overlap() {
        let values = values(vec![1, 2, 3, 4, 5, 6]);

        let mut result =
            Behaviour::apply_discard(2, DiscardType::Drop(DiscardDirection::High), values);
        result = Behaviour::apply_discard(2, DiscardType::Drop(DiscardDirection::High), result);

        assert_eq!(
            action(&result),
            vec![
                None,
                None,
                Some(Action::Discard),
                Some(Action::Discard),
                Some(Action::Discard),
                Some(Action::Discard),
            ]
        );
    }

    #[test]
    fn check_apply_explode_standard() {
        let mut rng = rng(Dice::D6, 5);
        let values = values(vec![1, 2, 3, 4, 5, 6]);

        let result = Behaviour::apply_explode(None, ExType::Standard, &Dice::D6, values, &mut rng);

        assert_eq!(result.len(), 6);
        assert_eq!(result[5].value(), 1);

        let actions = actions(&result);
        assert!(actions[0].is_empty());
        assert!(actions[1].is_empty());
        assert!(actions[2].is_empty());
        assert!(actions[3].is_empty());
        assert!(actions[4].is_empty());

        assert_eq!(
            actions[5],
            vec![
                Action::Explode(6, ExType::Standard),
                Action::Explode(6, ExType::Standard)
            ]
        );
    }

    #[test]
    fn check_apply_explode_compound() {
        let mut rng = rng(Dice::D6, 5);
        let values = values(vec![1, 2, 3, 4, 5, 6]);

        let result = Behaviour::apply_explode(None, ExType::Compound, &Dice::D6, values, &mut rng);

        assert_eq!(result.len(), 6);
        assert_eq!(result[5].value(), 13);

        let actions = actions(&result);
        assert!(actions[0].is_empty());
        assert!(actions[1].is_empty());
        assert!(actions[2].is_empty());
        assert!(actions[3].is_empty());
        assert!(actions[4].is_empty());
        assert_eq!(
            actions[5],
            vec![
                Action::Explode(6, ExType::Compound),
                Action::Explode(6, ExType::Compound),
                Action::Explode(1, ExType::Compound)
            ]
        );
    }

    #[test]
    fn check_apply_explode_pentrating() {
        let mut rng = rng(Dice::D6, 5);
        let values = values(vec![1, 2, 3, 4, 5, 6]);

        let result =
            Behaviour::apply_explode(None, ExType::Penetrating, &Dice::D6, values, &mut rng);

        assert_eq!(result.len(), 6);
        assert_eq!(result[5].value(), 5);

        let actions = actions(&result);
        assert!(actions[0].is_empty());
        assert!(actions[1].is_empty());
        assert!(actions[2].is_empty());
        assert!(actions[3].is_empty());
        assert!(actions[4].is_empty());
        assert_eq!(actions[5], vec![Action::Explode(6, ExType::Penetrating)]);
    }

    #[test]
    fn check_behaviour_ordering() {
        let mut v = vec![
            Behaviour::Reroll(Some(Bounded::from_range(..1)), true),
            Behaviour::Critical(None, None),
            Behaviour::Drop(1, DiscardDirection::Low),
            Behaviour::Explode(None, ExType::Penetrating),
            Behaviour::Keep(2, DiscardDirection::High),
            Behaviour::Reroll(None, false),
        ];
        v.sort();

        assert_eq!(
            v,
            vec![
                Behaviour::Reroll(None, false),
                Behaviour::Reroll(Some(Bounded::from_range(..1)), true),
                Behaviour::Explode(None, ExType::Penetrating),
                Behaviour::Critical(None, None),
                Behaviour::Keep(2, DiscardDirection::High),
                Behaviour::Drop(1, DiscardDirection::Low),
            ]
        );
    }

    #[test]
    fn check_apply() {
        let mut rng = rng(Dice::D6, 5);
        let values = values(vec![1, 2, 3, 4, 5, 6]);

        let result = Behaviour::apply(
            Behaviour::Drop(2, DiscardDirection::High),
            &Dice::D6,
            values,
            &mut rng,
        );

        assert_eq!(
            action(&result),
            vec![
                None,
                None,
                None,
                None,
                Some(Action::Discard),
                Some(Action::Discard),
            ]
        );
    }

    #[test]
    fn check_apply_all() {
        let mut rng = rng(Dice::D6, 5);
        let values = values(vec![1, 2, 3, 4, 5, 6]);

        let result = Behaviour::apply_all(
            vec![
                Behaviour::Keep(4, DiscardDirection::High),
                Behaviour::Drop(2, DiscardDirection::High),
            ],
            &Dice::D6,
            values,
            &mut rng,
        );

        assert_eq!(
            action(&result),
            vec![
                Some(Action::Discard),
                Some(Action::Discard),
                None,
                None,
                Some(Action::Discard),
                Some(Action::Discard),
            ]
        );
    }

    #[test]
    fn check_parse_reroll() -> Result<()> {
        assert_eq!(Behaviour::from_str("r")?, Behaviour::Reroll(None, true));
        assert_eq!(
            Behaviour::from_str("r2")?,
            Behaviour::Reroll(Some(Bounded::from_range(..=2)), true)
        );
        assert!(matches!(Behaviour::from_str("rq"), Err(_)));

        Ok(())
    }

    #[test]
    fn check_parse_explode() -> Result<()> {
        println!("!");
        assert_eq!(
            Behaviour::from_str("!")?,
            Behaviour::Explode(None, ExType::Standard)
        );
        println!("!2");
        assert_eq!(
            Behaviour::from_str("!2")?,
            Behaviour::Explode(Some(Bounded::from_range(2..)), ExType::Standard)
        );
        println!("!!");
        assert_eq!(
            Behaviour::from_str("!!")?,
            Behaviour::Explode(None, ExType::Compound)
        );
        println!("!!2");
        assert_eq!(
            Behaviour::from_str("!!2")?,
            Behaviour::Explode(Some(Bounded::from_range(2..)), ExType::Compound)
        );
        println!("!p");
        assert_eq!(
            Behaviour::from_str("!p")?,
            Behaviour::Explode(None, ExType::Penetrating)
        );
        println!("!p2");
        assert_eq!(
            Behaviour::from_str("!p2")?,
            Behaviour::Explode(Some(Bounded::from_range(2..)), ExType::Penetrating)
        );
        println!("!q");
        // assert!(matches!(Behaviour::from_str("!q"), Err(_)));

        Ok(())
    }

    #[test]
    fn check_parse_critical() -> Result<()> {
        assert_eq!(Behaviour::from_str("cs")?, Behaviour::Critical(None, None));
        assert_eq!(
            Behaviour::from_str("cs2")?,
            Behaviour::Critical(None, Some(Bounded::from_range(2..)))
        );
        assert_eq!(Behaviour::from_str("cf")?, Behaviour::Critical(None, None));
        assert_eq!(
            Behaviour::from_str("cf2")?,
            Behaviour::Critical(Some(Bounded::from_range(..=2)), None)
        );
        assert!(matches!(Behaviour::from_str("c"), Err(_)));
        assert!(matches!(Behaviour::from_str("cq"), Err(_)));

        Ok(())
    }

    #[test]
    fn check_parse_keep() -> Result<()> {
        assert_eq!(
            Behaviour::from_str("k1")?,
            Behaviour::Keep(1, DiscardDirection::High)
        );
        assert_eq!(
            Behaviour::from_str("kh1")?,
            Behaviour::Keep(1, DiscardDirection::High)
        );
        assert_eq!(
            Behaviour::from_str("kl1")?,
            Behaviour::Keep(1, DiscardDirection::Low)
        );
        assert!(matches!(Behaviour::from_str("kq"), Err(_)));

        Ok(())
    }

    #[test]
    fn check_parse_drop() -> Result<()> {
        assert_eq!(
            Behaviour::from_str("d1")?,
            Behaviour::Drop(1, DiscardDirection::Low)
        );
        assert_eq!(
            Behaviour::from_str("dh1")?,
            Behaviour::Drop(1, DiscardDirection::High)
        );
        assert_eq!(
            Behaviour::from_str("dl1")?,
            Behaviour::Drop(1, DiscardDirection::Low)
        );
        assert!(matches!(Behaviour::from_str("dq"), Err(_)));

        Ok(())
    }
}
