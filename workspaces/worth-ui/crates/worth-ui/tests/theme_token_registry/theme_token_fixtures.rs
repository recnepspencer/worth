use worth_ui::facade::declaration::{
    ThemeTokenAlias, ThemeTokenDescriptor, ThemeTokenFamily, ThemeTokenId, ThemeTokenSource,
    ThemeTokenValue, UiThemeColor,
};

pub(crate) fn theme_token_id(raw_text: &str) -> ThemeTokenId {
    ThemeTokenId::new(raw_text).expect("valid theme token id")
}

pub(crate) fn color_theme_token(id: &str, hex: &str) -> ThemeTokenDescriptor {
    ThemeTokenDescriptor::define(
        theme_token_id(id),
        ThemeTokenFamily::text(),
        ThemeTokenSource::application(),
        color_value(hex),
    )
}

pub(crate) fn platform_color_theme_token(id: &str, hex: &str) -> ThemeTokenDescriptor {
    ThemeTokenDescriptor::define(
        theme_token_id(id),
        ThemeTokenFamily::text(),
        ThemeTokenSource::platform(),
        color_value(hex),
    )
}

pub(crate) fn alias_theme_token(id: &str, target_id: &str) -> ThemeTokenDescriptor {
    ThemeTokenDescriptor::alias(
        theme_token_id(id),
        ThemeTokenFamily::text(),
        ThemeTokenSource::application(),
        ThemeTokenAlias::to(theme_token_id(target_id)),
    )
}

pub(crate) fn plugin_custom_theme_token(id: &str, hex: &str) -> ThemeTokenDescriptor {
    ThemeTokenDescriptor::define(
        theme_token_id(id),
        ThemeTokenFamily::accent(),
        ThemeTokenSource::plugin_custom(),
        color_value(hex),
    )
}

pub(crate) fn plugin_alias_theme_token(id: &str, target_id: &str) -> ThemeTokenDescriptor {
    ThemeTokenDescriptor::alias(
        theme_token_id(id),
        ThemeTokenFamily::accent(),
        ThemeTokenSource::plugin_alias(),
        ThemeTokenAlias::to(theme_token_id(target_id)),
    )
}

pub(crate) fn color_value(hex: &str) -> ThemeTokenValue {
    ThemeTokenValue::color(UiThemeColor::parse(hex).expect("valid theme color value"))
}
