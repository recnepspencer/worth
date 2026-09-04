use super::ThemeColorValue;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ThemeTokenValue {
    Color(ThemeColorValue),
    Typed(worth_ui_dsl::UiThemeValue),
}

impl ThemeTokenValue {
    pub fn color(value: ThemeColorValue) -> Self {
        Self::Color(value)
    }

    pub fn typed(value: worth_ui_dsl::UiThemeValue) -> Self {
        Self::Typed(value)
    }

    pub(crate) fn is_valid(&self) -> bool {
        match self {
            Self::Color(value) => value.is_valid(),
            Self::Typed(_) => true,
        }
    }

    pub(crate) fn digest_basis(&self) -> String {
        match self {
            Self::Color(value) => format!("color({})", value.digest_basis()),
            Self::Typed(value) => format!("typed({})", typed_digest_basis(*value)),
        }
    }
}

fn typed_digest_basis(value: worth_ui_dsl::UiThemeValue) -> String {
    use worth_ui_dsl::UiThemeValue;
    match value {
        UiThemeValue::Color(color) => format!("color({:?})", color.channels()),
        UiThemeValue::Opacity(opacity) => format!("opacity({})", opacity.units()),
        UiThemeValue::LogicalLength(length) => format!("length({})", length.subpixels()),
        UiThemeValue::CornerRadii(radii) => format!(
            "radii({:?})",
            radii
                .corners()
                .map(worth_ui_dsl::UiLogicalLength::subpixels)
        ),
        UiThemeValue::SolidStroke(stroke) => format!(
            "stroke({:?},{})",
            stroke.color().channels(),
            stroke.width().subpixels()
        ),
        UiThemeValue::SolidOutline(outline) => format!(
            "outline({:?},{},{})",
            outline.stroke().color().channels(),
            outline.stroke().width().subpixels(),
            outline.offset().subpixels()
        ),
    }
}
