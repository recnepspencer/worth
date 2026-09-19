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

#[test]
fn mounted_alignment_centers_different_shaped_labels_and_invalidates_reuse() {
    use worth_ui_text::UiTextAlignment;
    let (fonts, _) = worth_ui_text::UiGlobalFontCollection::admit_qualified_profile().unwrap();
    let fonts = Arc::new(fonts);
    let token = crate::capability::ThemeTokenId::new("theme.label").unwrap();
    let bounds = worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
        worth_ui_host_contract::UiMountedCanonicalBoxInput {
            x: 30.0,
            y: 50.0,
            width: 180.0,
            height: 48.0,
            coordinate_space: worth_ui_host_contract::UiMountedCoordinateSpace::HostSurface,
        },
    )
    .unwrap();
    let make = |alignment| {
        let contract =
            crate::capability::ComponentSemanticTextContract::body_default(token.clone(), 1)
                .with_alignment(alignment)
                .with_appearance_foreground();
        super::lower_directive(
            &crate::mounting::UiMountedSemanticTextFormattingDirective::new(
                contract,
                BTreeMap::new(),
            ),
        )
        .unwrap()
    };
    let start = make(UiTextAlignment::Start);
    let centered = make(UiTextAlignment::Center);
    assert!(
        !start.same_layout_as(&centered),
        "alignment changes must requalify, not reuse predecessor placement"
    );
    for source in ["Cancel", "Approve deployment", "JD", "Operational"] {
        let constraints =
            super::super::qualification::paragraph_constraints(bounds, centered.default_row())
                .unwrap();
        let (styles, _) = centered
            .default_row()
            .materialize(source, &constraints)
            .unwrap();
        let layout = worth_ui_text::UiQualifiedTextLayoutRequest::new(
            worth_ui_text::UiTextParagraphAdmissionInput {
                source: Arc::from(source),
                constraints,
                styles,
                profile_generation: super::super::current_text_profile_generation(),
                font_collection_generation: fonts.generation(),
                text_scale_generation: worth_ui_host_contract::UiTextScaleGeneration::new(1)
                    .unwrap(),
            },
            Arc::clone(&fonts),
        )
        .qualify()
        .unwrap();
        assert_eq!(layout.lines().len(), 1);
        let line = layout.lines()[0].logical_bounds();
        assert!(
            (line.left_millipoints() - (180_000 - line.right_millipoints())).abs() <= 1,
            "{source}: actual shaped line has equal horizontal margins"
        );
    }
}
