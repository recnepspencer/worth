use super::ThemeTokenDescriptor;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ThemeTokenKey {
    projection_basis: String,
}

impl ThemeTokenKey {
    pub(crate) fn from_descriptor(descriptor: &ThemeTokenDescriptor) -> Self {
        Self {
            projection_basis: theme_token_projection_basis(descriptor),
        }
    }

    pub fn projection_basis(&self) -> &str {
        &self.projection_basis
    }
}

fn theme_token_projection_basis(descriptor: &ThemeTokenDescriptor) -> String {
    [
        length_prefixed(descriptor.id().as_str()),
        descriptor.family().digest_basis(),
        descriptor.source().digest_basis().to_string(),
        value_basis(descriptor),
        alias_basis(descriptor),
    ]
    .join("|")
}

fn value_basis(descriptor: &ThemeTokenDescriptor) -> String {
    descriptor
        .value()
        .map(|value| format!("value:{}", value.digest_basis()))
        .unwrap_or_else(|| "value:none".to_string())
}

fn alias_basis(descriptor: &ThemeTokenDescriptor) -> String {
    descriptor
        .alias_definition()
        .map(|alias| format!("alias:{}", alias.digest_basis()))
        .unwrap_or_else(|| "alias:none".to_string())
}

fn length_prefixed(value: &str) -> String {
    format!("{}:{value}", value.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::{ThemeTokenFamily, ThemeTokenId, ThemeTokenSource, ThemeTokenValue};
    use worth_ui_dsl::{UiThemeColor, UiThemeGradientPoint, UiThemeLinearGradient};

    #[test]
    fn gradient_token_projection_key_tracks_kind_axis_and_each_stop() {
        let colors = [[255, 0, 0, 255], [0, 0, 255, 0]];
        let make = |end, colors: [[u8; 4]; 2]| {
            ThemeTokenValue::linear_gradient(
                UiThemeLinearGradient::new(
                    UiThemeGradientPoint::new(0, 0).unwrap(),
                    UiThemeGradientPoint::new(end, 10_000).unwrap(),
                    colors.map(UiThemeColor::from_channels),
                )
                .unwrap(),
            )
        };
        let key = |value| {
            ThemeTokenKey::from_descriptor(&ThemeTokenDescriptor::define(
                ThemeTokenId::new("dashboard.gradient").unwrap(),
                ThemeTokenFamily::surface(),
                ThemeTokenSource::application(),
                value,
            ))
        };
        let baseline = key(make(0, colors));
        assert_eq!(baseline, key(make(0, colors)));
        assert_ne!(baseline, key(make(10_000, colors)));
        for stop in 0..2 {
            let mut changed = colors;
            changed[stop][1] = 100;
            assert_ne!(baseline, key(make(0, changed)));
        }
        assert_ne!(
            baseline,
            key(ThemeTokenValue::color(UiThemeColor::from_channels(
                colors[0]
            )))
        );
    }
}
