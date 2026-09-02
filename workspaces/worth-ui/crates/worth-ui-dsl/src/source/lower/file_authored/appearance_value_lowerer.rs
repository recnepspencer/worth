use crate::{
    UiAppearanceAspect, UiLogicalLength, UiThemeColor, UiThemeCornerRadii, UiThemeOpacity,
    UiThemeOutline, UiThemeSolidStroke, UiThemeValue, UiThemeValueKind,
};

pub(super) fn transparent_value(aspect: UiAppearanceAspect) -> UiThemeValue {
    let color = UiThemeColor::from_channels([0, 0, 0, 0]);
    let zero = UiLogicalLength::new(0);
    match aspect.value_kind() {
        UiThemeValueKind::Color => UiThemeValue::Color(color),
        UiThemeValueKind::Opacity => UiThemeValue::Opacity(UiThemeOpacity::ZERO),
        UiThemeValueKind::LogicalLength => UiThemeValue::LogicalLength(zero),
        UiThemeValueKind::CornerRadii => UiThemeValue::CornerRadii(
            UiThemeCornerRadii::new(zero, zero, zero, zero).expect("zero radii are valid"),
        ),
        UiThemeValueKind::SolidStroke => UiThemeValue::SolidStroke(
            UiThemeSolidStroke::new(color, zero).expect("zero stroke is valid"),
        ),
        UiThemeValueKind::SolidOutline => UiThemeValue::SolidOutline(
            UiThemeOutline::new(
                UiThemeSolidStroke::new(color, zero).expect("zero stroke is valid"),
                zero,
            )
            .expect("zero outline is valid"),
        ),
    }
}
