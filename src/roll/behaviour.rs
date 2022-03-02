use {
    crate::{
        dice::Dice,
        roll::value::{Action, ExType, Value},
    },
    rand::RngCore,
    std::{iter::Iterator, ops::RangeInclusive},
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
    Reroll(Option<i8>, bool),
    Explode(Option<i8>, ExType),
    Critical(Option<i8>, Option<i8>),
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

    fn failure(dice: &Dice, point: &Option<i8>) -> Option<RangeInclusive<i8>> {
        point
            .map(|p| *dice.faces().start()..=p)
            .or_else(|| dice.start())
    }

    fn success(dice: &Dice, point: &Option<i8>) -> Option<RangeInclusive<i8>> {
        point
            .map(|p| p..=*dice.faces().end())
            .or_else(|| dice.end())
    }

    fn apply_reroll(
        point: Option<i8>,
        repeat: bool,
        dice: &Dice,
        values: Vec<Value>,
        rng: &mut dyn RngCore,
    ) -> Vec<Value> {
        if let Some(range) = Self::failure(dice, &point) {
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
        failure: Option<i8>,
        success: Option<i8>,
        dice: &Dice,
        values: Vec<Value>,
    ) -> Vec<Value> {
        let mut result = Vec::new();
        for value in values {
            result.push(
                match (Self::failure(dice, &failure), Self::success(dice, &success)) {
                    (Some(v), _) if v.contains(&value.value()) => value.add(Action::Failure),
                    (_, Some(v)) if v.contains(&value.value()) => value.add(Action::Success),
                    _ => value,
                },
            );
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
        point: Option<i8>,
        explode: ExType,
        dice: &Dice,
        values: Vec<Value>,
        rng: &mut dyn RngCore,
    ) -> Vec<Value> {
        if let Some(range) = Self::success(dice, &point) {
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
}

#[cfg(test)]
mod test {
    use {super::*, crate::mock::rng::*};

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

        let result = Behaviour::apply_critical(Some(2), Some(5), &Dice::D6, values);

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
            Behaviour::Reroll(Some(1), true),
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
                Behaviour::Reroll(Some(1), true),
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

    fn values(values: Vec<i8>) -> Vec<Value> {
        values.iter().map(|v| Value::new(*v)).collect()
    }

    fn action<'a>(values: &'a Vec<Value>) -> Vec<Option<Action>> {
        values
            .iter()
            .filter(|v| v.actions().len() < 2)
            .map(|v| v.actions().get(0).map(|v| *v).or(None))
            .collect()
    }

    fn actions<'a>(values: &'a Vec<Value>) -> Vec<Vec<Action>> {
        values.iter().map(|v| v.actions().clone()).collect()
    }
}
