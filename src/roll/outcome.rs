use {
    crate::roll::value::{Action, Value},
    std::{collections::HashMap, iter::Iterator},
};

enum Outcomes {
    Total,
    Target(i8),
    Match,
}

impl Outcomes {
    pub fn process(&self, values: Vec<Value>) -> i8 {
        match self {
            Self::Total => values
                .iter()
                .filter(|v| !v.actions().contains(&Action::Discard))
                .map(|v| v.value())
                .sum(),
            Self::Target(point) => values
                .iter()
                .filter(|v| !v.actions().contains(&Action::Discard))
                .filter(|v| v.value() >= *point)
                .count() as i8,
            Self::Match => values
                .iter()
                .filter(|v| !v.actions().contains(&Action::Discard))
                .fold(HashMap::new(), |mut m, v| {
                    *m.entry(v.value()).or_insert(0) += 1i8;
                    m
                })
                .values()
                .filter(|v| *v > &1i8)
                .count() as i8,
        }
    }
}

#[cfg(test)]
mod test {
    use {super::*, crate::roll::value::test::*};

    #[test]
    fn check_process_total() {
        let values = values(vec![1, 2, 2, 3, 3, 3]);

        assert_eq!(Outcomes::Total.process(values), 14);
    }

    #[test]
    fn check_process_target() {
        let values = values(vec![1, 2, 2, 3, 3, 3]);

        assert_eq!(Outcomes::Target(3).process(values), 3);
    }

    #[test]
    fn check_process_match() {
        let values = values(vec![1, 2, 2, 3, 3, 3]);

        assert_eq!(Outcomes::Match.process(values), 2);
    }
}
