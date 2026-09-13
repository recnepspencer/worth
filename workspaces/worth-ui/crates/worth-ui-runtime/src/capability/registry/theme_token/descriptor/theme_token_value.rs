use super::UiThemeColor;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ThemeTokenValue {
    Color(UiThemeColor),
}

impl ThemeTokenValue {
    pub fn color(value: UiThemeColor) -> Self {
        Self::Color(value)
    }

    pub(crate) fn is_valid(&self) -> bool {
        true
    }

    pub(crate) fn digest_basis(&self) -> String {
        match self {
            Self::Color(value) => format!("color({:?})", value.channels()),
        }
    }
}
