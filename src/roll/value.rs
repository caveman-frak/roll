use {
    crate::dice::Dice,
    colored::{ColoredString, Colorize},
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
        self.format_actions(dice.text(self.value())).to_string()
    }

    fn format_actions(&self, text: String) -> ColoredString {
        let mut text = text.normal();
        if self.actions().contains(&Action::Discard) {
            text = text.strikethrough().dimmed()
        }
        if self.actions().contains(&Action::Failure) {
            text = text.red()
        } else if self.actions().contains(&Action::Success) {
            text = text.green()
        }
        text
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.format_actions(self.value().to_string()))
    }
}
