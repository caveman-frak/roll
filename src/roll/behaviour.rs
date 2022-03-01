use {crate::dice::Dice, rand::RngCore, std::iter::Iterator};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Action {
    Discard,
    Reroll(u8),
    Explode(u8, ExType),
    Failure,
    Success,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Value {
    value: u8,
    actions: Vec<Action>,
}

impl Value {
    pub fn new(value: u8) -> Self {
        Self {
            value,
            actions: Vec::new(),
        }
    }

    pub fn value(&self) -> u8 {
        self.value
    }

    fn add(mut self, action: Action) -> Self {
        self.actions.push(action);
        self
    }

    fn update(mut self, value: u8, action: Action) -> Self {
        self.value = value;
        self.actions.push(action);
        self
    }

    pub fn text(&self, dice: &Dice) -> String {
        dice.text(self.value())
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DiscardType {
    Keep,
    Drop,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ExType {
    Standard,
    Compound,
    Penetrating,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Behaviour {
    Keep(usize),
    Drop(usize),
    Reroll(Option<u8>, bool),
    Explode(Option<u8>, ExType),
    Critical(Option<u8>, Option<u8>),
}

impl Behaviour {
    pub fn apply(
        behaviour: Behaviour,
        dice: &Dice,
        values: Vec<Value>,
        rng: &mut dyn RngCore,
    ) -> Vec<Value> {
        match behaviour {
            Self::Keep(number) => Self::apply_discard(number, DiscardType::Keep, values),
            Self::Drop(number) => Self::apply_discard(number, DiscardType::Drop, values),
            Self::Reroll(point, repeat) => Self::apply_reroll(point, repeat, dice, values, rng),
            Self::Explode(point, explode) => Self::apply_explode(point, explode, dice, values, rng),
            Self::Critical(failure, success) => {
                Self::apply_critical(failure, success, dice, values)
            }
            _ => values,
        }
    }

    fn apply_reroll(
        point: Option<u8>,
        repeat: bool,
        dice: &Dice,
        values: Vec<Value>,
        rng: &mut dyn RngCore,
    ) -> Vec<Value> {
        if let Some(p) = point.or(dice.failure()) {
            let mut result = Vec::new();
            for value in values {
                let mut v = value;
                while v.value <= p {
                    v = v.clone().update(dice.roll(rng), Action::Reroll(v.value));
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
        failure: Option<u8>,
        success: Option<u8>,
        dice: &Dice,
        values: Vec<Value>,
    ) -> Vec<Value> {
        let failure = failure.or(dice.failure());
        let success = success.or(dice.success());
        let mut result = Vec::new();
        for value in values {
            result.push(if failure.is_some() && value.value <= failure.unwrap() {
                value.add(Action::Failure)
            } else if success.is_some() && value.value >= success.unwrap() {
                value.add(Action::Success)
            } else {
                value
            });
        }
        result
    }

    fn apply_discard(number: usize, discard: DiscardType, values: Vec<Value>) -> Vec<Value> {
        let mut numbers: Vec<u8> = values.iter().map(|v| v.value).collect();
        numbers.sort_unstable();
        if discard == DiscardType::Drop {
            numbers.reverse();
        }
        let mut discards = numbers[0..number].to_vec();
        let mut results = Vec::new();
        for value in values {
            results.push(if let Ok(index) = discards.binary_search(&value.value) {
                discards.remove(index);
                value.add(Action::Discard)
            } else {
                value
            });
        }
        results
    }

    fn apply_explode(
        point: Option<u8>,
        explode: ExType,
        dice: &Dice,
        values: Vec<Value>,
        rng: &mut dyn RngCore,
    ) -> Vec<Value> {
        if let Some(p) = point.or(dice.success()) {
            let mut result = Vec::new();
            for value in values {
                let mut v = value;
                while v.value >= p {
                    let mut r = dice.roll(rng);
                    v = match explode {
                        ExType::Standard => v.clone().update(r, Action::Explode(v.value, explode)),
                        ExType::Penetrating => {
                            r -= 1;
                            v.clone().update(r, Action::Explode(v.value, explode))
                        }
                        ExType::Compound => {
                            v.clone().update(v.value + r, Action::Explode(r, explode))
                        }
                    };
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
        let values = values(vec![0, 1, 2, 3, 4, 5]);

        let result = Behaviour::apply_critical(None, None, &Dice::D6, values);

        assert_eq!(result.len(), 6);
        let actions = action(&result);
        assert_eq!(actions[0], Some(&Action::Failure));
        assert_eq!(actions[1], None);
        assert_eq!(actions[2], None);
        assert_eq!(actions[3], None);
        assert_eq!(actions[4], None);
        assert_eq!(actions[5], Some(&Action::Success));
    }

    #[test]
    fn check_apply_critical_override() {
        let values = values(vec![0, 1, 2, 3, 4, 5]);

        let result = Behaviour::apply_critical(Some(1), Some(4), &Dice::D6, values);

        assert_eq!(result.len(), 6);
        let actions = action(&result);
        assert_eq!(actions[0], Some(&Action::Failure));
        assert_eq!(actions[1], Some(&Action::Failure));
        assert_eq!(actions[2], None);
        assert_eq!(actions[3], None);
        assert_eq!(actions[4], Some(&Action::Success));
        assert_eq!(actions[5], Some(&Action::Success));
    }

    #[test]
    fn check_apply_reroll_once() {
        let mut rng = rng(Dice::D6, 1);
        let values = values(vec![0, 1, 2, 3, 4, 5]);

        let result = Behaviour::apply_reroll(None, false, &Dice::D6, values, &mut rng);

        assert_eq!(result.len(), 6);
        assert_eq!(result[0].value(), 1);

        let actions = action(&result);
        assert_eq!(actions[0], Some(&Action::Reroll(0)));
        assert_eq!(actions[1], None);
        assert_eq!(actions[2], None);
        assert_eq!(actions[3], None);
        assert_eq!(actions[4], None);
        assert_eq!(actions[5], None);
    }

    #[test]
    fn check_apply_reroll() {
        let mut rng = rng(Dice::D6, 0);
        let values = values(vec![0, 1, 2, 3, 4, 5]);

        let result = Behaviour::apply_reroll(None, true, &Dice::D6, values, &mut rng);

        assert_eq!(result.len(), 6);
        assert_eq!(result[0].value(), 1);

        let actions = actions(&result);
        assert_eq!(actions[0].len(), 2);
        assert_eq!(actions[0][0], Action::Reroll(0));
        assert_eq!(actions[0][1], Action::Reroll(0));
        assert!(actions[1].is_empty());
        assert!(actions[2].is_empty());
        assert!(actions[3].is_empty());
        assert!(actions[4].is_empty());
        assert!(actions[5].is_empty());
    }

    #[test]
    fn check_apply_discard_keep() {
        let values = values(vec![0, 1, 2, 3, 4, 5]);

        let result = Behaviour::apply_discard(2, DiscardType::Keep, values);

        assert_eq!(result.len(), 6);
        let actions = action(&result);
        assert_eq!(actions[0], Some(&Action::Discard));
        assert_eq!(actions[1], Some(&Action::Discard));
        assert_eq!(actions[2], None);
        assert_eq!(actions[3], None);
        assert_eq!(actions[4], None);
        assert_eq!(actions[5], None);
    }

    #[test]
    fn check_apply_discard_keep_duplicates() {
        let values = values(vec![2, 2, 2, 2, 4, 5]);

        let result = Behaviour::apply_discard(2, DiscardType::Keep, values);

        assert_eq!(result.len(), 6);
        let actions = action(&result);
        assert_eq!(actions[0], Some(&Action::Discard));
        assert_eq!(actions[1], Some(&Action::Discard));
        assert_eq!(actions[2], None);
        assert_eq!(actions[3], None);
        assert_eq!(actions[4], None);
        assert_eq!(actions[5], None);
    }

    #[test]
    fn check_apply_discard_drop() {
        let values = values(vec![0, 1, 2, 3, 4, 5]);

        let result = Behaviour::apply_discard(2, DiscardType::Drop, values);

        assert_eq!(result.len(), 6);
        let actions = action(&result);
        assert_eq!(actions[0], None);
        assert_eq!(actions[1], None);
        assert_eq!(actions[2], None);
        assert_eq!(actions[3], None);
        assert_eq!(actions[4], Some(&Action::Discard));
        assert_eq!(actions[5], Some(&Action::Discard));
    }

    #[test]
    fn check_apply_explode_standard() {
        let mut rng = rng(Dice::D6, 5);
        let values = values(vec![0, 1, 2, 3, 4, 5]);

        let result = Behaviour::apply_explode(None, ExType::Standard, &Dice::D6, values, &mut rng);

        assert_eq!(result.len(), 6);
        assert_eq!(result[5].value(), 0);

        let actions = actions(&result);
        assert!(actions[0].is_empty());
        assert!(actions[1].is_empty());
        assert!(actions[2].is_empty());
        assert!(actions[3].is_empty());
        assert!(actions[4].is_empty());
        assert_eq!(actions[5].len(), 2);
        assert_eq!(actions[5][0], Action::Explode(5, ExType::Standard));
        assert_eq!(actions[5][1], Action::Explode(5, ExType::Standard));
    }

    fn values(values: Vec<u8>) -> Vec<Value> {
        values.iter().map(|v| Value::new(*v)).collect()
    }

    fn action<'a>(values: &'a Vec<Value>) -> Vec<Option<&Action>> {
        values.iter().map(|v| v.actions.get(0)).collect()
    }

    fn actions<'a>(values: &'a Vec<Value>) -> Vec<Vec<Action>> {
        values.iter().map(|v| v.actions.clone()).collect()
    }
}
