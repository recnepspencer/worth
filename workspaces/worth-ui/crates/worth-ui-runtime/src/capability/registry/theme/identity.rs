#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct UiThemeDefinitionIdentity(Box<str>);

impl UiThemeDefinitionIdentity {
    pub fn new(value: impl Into<Box<str>>) -> Option<Self> {
        let value = value.into();
        (!value.is_empty()).then_some(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
