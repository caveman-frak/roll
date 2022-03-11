use {
    crate::dice::Dice,
    colored::Colorize,
    joinery::{separators::Space, JoinableIterator},
    std::fmt::{self, Display},
};

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub enum ExType {
    Standard,
    Compound,
    Penetrating,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Action {
    Discard,
    Reroll(i8),
    Explode(i8, ExType),
    Failure,
    Success,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Value {
    value: i8,
    actions: Vec<Action>,
}

impl Value {
    pub fn new(value: i8) -> Self {
        Self {
            value,
            actions: Vec::new(),
        }
    }

    pub fn value(&self) -> i8 {
        self.value
    }

    pub fn actions(&self) -> &Vec<Action> {
        &self.actions
    }

    pub fn add(mut self, action: Action) -> Self {
        self.actions.push(action);
        self
    }

    pub fn update(mut self, value: i8, action: Action) -> Self {
        self.value = value;
        self.actions.push(action);
        self
    }

    pub fn text(&self, dice: &Dice) -> String {
        self.format_text(dice.text(self.value()))
    }

    fn format_text(&self, text: String) -> String {
        let mut modifiers = (false, false, false, false, false);
        let mut reroll = Vec::new();
        let mut explode = Vec::new();
        let mut text = text.normal();

        for action in self.actions() {
            match action {
                Action::Discard => modifiers.0 = true,
                Action::Failure => modifiers.1 = true,
                Action::Success => modifiers.2 = true,
                Action::Explode(value, _) => {
                    modifiers.3 = true;
                    explode.push(value);
                }
                Action::Reroll(value) => {
                    modifiers.4 = true;
                    reroll.push(value);
                }
            }
        }

        let pre = if reroll.is_empty() {
            "".normal()
        } else {
            format!(
                "({})",
                reroll
                    .iter()
                    .map(|v| v.to_string().strikethrough())
                    .join_with(Space)
            )
            .dimmed()
        };
        let post = if explode.is_empty() {
            "".to_string()
        } else {
            format!(
                "({})",
                explode
                    .iter()
                    .map(|v| v.to_string().bold())
                    .join_with(Space)
            )
        };

        if modifiers.0 {
            // discard
            text = text.strikethrough();
        }
        if modifiers.1 {
            //failure
            text = text.red();
        } else if modifiers.2 {
            //success
            text = text.green();
        }
        if modifiers.3 {
            // exploded
            text = text.green().bold();
        } else if modifiers.4 {
            // rerolled
        }
        format!("{}{}{}", pre, text, post)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.format_text(self.value().to_string()))
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;

    pub(crate) fn values(values: Vec<i8>) -> Vec<Value> {
        values.iter().map(|v| Value::new(*v)).collect()
    }

    pub(crate) fn action<'a>(values: &'a Vec<Value>) -> Vec<Option<Action>> {
        values
            .iter()
            .filter(|v| v.actions().len() < 2)
            .map(|v| v.actions().get(0).map(|v| *v).or(None))
            .collect()
    }

    pub(crate) fn actions<'a>(values: &'a Vec<Value>) -> Vec<Vec<Action>> {
        values.iter().map(|v| v.actions().clone()).collect()
    }
}
