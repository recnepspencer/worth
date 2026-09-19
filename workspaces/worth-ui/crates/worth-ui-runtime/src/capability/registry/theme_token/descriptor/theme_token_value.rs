use super::UiThemeColor;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ThemeTokenValue {
    Color(UiThemeColor),
    LinearGradient(worth_ui_dsl::UiThemeLinearGradient),
}

impl ThemeTokenValue {
    pub fn linear_gradient(value: worth_ui_dsl::UiThemeLinearGradient) -> Self {
        Self::LinearGradient(value)
    }
    pub fn color(value: UiThemeColor) -> Self {
        Self::Color(value)
    }

    pub(crate) fn is_valid(&self) -> bool {
        true
    }

    pub(crate) fn digest_basis(&self) -> String {
        match self {
            Self::Color(value) => format!("color({:?})", value.channels()),
            Self::LinearGradient(value) => format!(
                "linear_gradient({:?},{:?},{:?})",
                value.start().coordinates(),
                value.end().coordinates(),
                value.colors().map(UiThemeColor::channels)
            ),
        }
    }
}
