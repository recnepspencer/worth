use std::{collections::BTreeMap, sync::Arc};

use super::lower_directive;

#[test]
fn explicit_line_height_reaches_mounted_row_formatting() {
    let token = crate::capability::ThemeTokenId::new("theme.platform_pulse.heading").unwrap();
    let constraints = worth_ui_text::UiTextParagraphConstraints::new(
        worth_ui_text::UiTextParagraphConstraintsInput {
            language: Arc::from("und"),
            base_direction: worth_ui_text::UiTextBaseDirection::Auto,
            wrap: worth_ui_text::UiTextWrap::UnicodeWord,
            alignment: worth_ui_text::UiTextAlignment::Start,
            overflow: worth_ui_text::UiTextOverflow::Clip,
            font_size_millipoints: 28_000,
            width_millipoints: 320_000,
            line_height_millipoints: 36_000,
            letter_spacing_millipoints: 0,
            word_spacing_millipoints: 0,
            tab_interval_millipoints: 112_000,
            maximum_lines: 1,
        },
    )
    .unwrap();
    let contract = crate::capability::ComponentSemanticTextContract::qualified_with_line_height(
        token.clone(),
        1,
        worth_ui_text::UiTextStyle::from_paragraph_constraints(&constraints),
        36_000,
    )
    .unwrap();
    let directive = crate::mounting::UiMountedSemanticTextFormattingDirective::new(
        contract,
        BTreeMap::from([(
            token,
            crate::capability::ThemeTokenValue::color(
                crate::capability::UiThemeColor::parse("#241F2B").unwrap(),
            ),
        )]),
    );

    let formatting = lower_directive(&directive).unwrap();

    assert_eq!(
        formatting.default_row().line_height_millipoints(),
        Some(36_000),
    );
}
