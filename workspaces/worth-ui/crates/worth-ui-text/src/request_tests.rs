use std::sync::Arc;

use worth_ui_host_contract::{UiFontSlant, UiTextOriginalRange};

use crate::{
    UiFontFamilyStack, UiFontVariationCoordinate, UiGlobalFontCollection, UiOpenTypeFeature,
    UiQualifiedTextLayoutRequest, UiTextAlignment, UiTextBaseDirection, UiTextFaceRequest,
    UiTextOverflow, UiTextParagraphAdmissionInput, UiTextParagraphConstraints,
    UiTextParagraphConstraintsInput, UiTextProfileGeneration, UiTextScaleGeneration, UiTextStyle,
    UiTextStyleInput, UiTextStyleSpan, UiTextWrap,
};

#[test]
fn feature_and_variation_domains_cannot_alias_style_or_request_identity() {
    let feature = style(
        Box::new([UiOpenTypeFeature::new(*b"wght", 700).unwrap()]),
        Box::new([]),
    );
    let variation = style(
        Box::new([]),
        Box::new([UiFontVariationCoordinate::new(*b"wght", 700).unwrap()]),
    );
    assert_ne!(feature.identity_digest(), variation.identity_digest());

    let (fonts, _) = UiGlobalFontCollection::admit_qualified_profile().unwrap();
    let fonts = Arc::new(fonts);
    let feature_request = UiQualifiedTextLayoutRequest::new(input(feature, &fonts), fonts.clone());
    let variation_request = UiQualifiedTextLayoutRequest::new(input(variation, &fonts), fonts);
    assert_ne!(feature_request.identity(), variation_request.identity());
}

#[test]
fn a_start_aligned_label_is_one_layout_across_the_widths_it_fits() {
    let fonts = fonts();
    let wide = qualify("office hours", 100_000, UiTextAlignment::Start, &fonts);
    let wider = qualify("office hours", 160_000, UiTextAlignment::Start, &fonts);
    assert_eq!(wide.lines().len(), 1);
    assert_eq!(wide.identity(), wider.identity());
    assert_eq!(
        wide.view().request_identity(),
        wider.view().request_identity()
    );
    assert_eq!(wide.view().width_basis(), wider.view().width_basis());
    assert!(wide.admits_width(160_000) && wider.admits_width(100_000));
    assert_ne!(
        request("office hours", 100_000, UiTextAlignment::Start, &fonts).identity(),
        request("office hours", 160_000, UiTextAlignment::Start, &fonts).identity(),
        "a request still names the width it asks for"
    );
    let raw = request("office hours", 100_000, UiTextAlignment::Start, &fonts);
    assert_eq!(wide.reflow_key(), raw.reflow_key());
}

#[test]
fn a_layout_is_named_by_the_least_width_that_fits_it() {
    let fonts = fonts();
    let wide = qualify("office hours", 100_000, UiTextAlignment::Start, &fonts);
    let fitted = wide.view().width_basis().width_millipoints();
    assert!(fitted < 100_000 && wide.admits_width(fitted) && !wide.admits_width(fitted - 1));
    let exact = request("office hours", fitted, UiTextAlignment::Start, &fonts);
    assert_eq!(wide.view().request_identity(), exact.identity());
    assert_eq!(exact.qualify().unwrap().identity(), wide.identity());
    let narrower = qualify("office hours", fitted - 1, UiTextAlignment::Start, &fonts);
    assert_ne!(narrower.identity(), wide.identity());
    assert_eq!(narrower.lines().len(), 2, "one width less wraps the label");
}

#[test]
fn a_wrapped_label_stands_for_the_widths_below_its_wrap() {
    let fonts = fonts();
    let narrow = qualify("office hours", 40_000, UiTextAlignment::Start, &fonts);
    assert_eq!(narrow.lines().len(), 2);
    let fitted = narrow.view().width_basis().width_millipoints();
    let wide = qualify("office hours", 100_000, UiTextAlignment::Start, &fonts);
    let wraps_below = wide.view().width_basis().width_millipoints();
    assert!(narrow.admits_width(fitted) && narrow.admits_width(wraps_below - 1));
    assert!(
        !narrow.admits_width(wraps_below),
        "the whole label fits here"
    );
    let below = qualify(
        "office hours",
        wraps_below - 1,
        UiTextAlignment::Start,
        &fonts,
    );
    assert_eq!(below.identity(), narrow.identity());
}

#[test]
fn width_read_by_placement_keeps_a_layout_to_its_own_width() {
    let fonts = fonts();
    for (source, alignment) in [
        ("office hours", UiTextAlignment::Center),
        ("office hours", UiTextAlignment::End),
        ("שלום עולם", UiTextAlignment::Start),
        ("office hours\n", UiTextAlignment::Start),
    ] {
        let layout = qualify(source, 100_000, alignment, &fonts);
        assert_eq!(layout.view().width_basis().width_millipoints(), 100_000);
        assert!(layout.admits_width(100_000), "{source:?} {alignment:?}");
        assert!(!layout.admits_width(100_001), "{source:?} {alignment:?}");
        assert!(!layout.admits_width(99_999), "{source:?} {alignment:?}");
        let raw = request(source, 100_000, alignment, &fonts);
        assert_eq!(layout.view().request_identity(), raw.identity());
    }
}

#[test]
fn every_width_a_layout_admits_qualifies_that_layout() {
    let fonts = fonts();
    for source in [
        "office hours and more",
        "tab\tstops wrap",
        "ab cd\nef gh ij",
    ] {
        assert_admitted_widths_qualify(source, START, &fonts);
    }
}

/// The same property where fitting reads more than word breaks: contextual
/// shaping remeasured across a grapheme wrap, right-to-left text placed from
/// the left edge, spacing, unwrapped overflow, and an ellipsis.
#[test]
fn every_width_a_layout_admits_qualifies_it_in_every_flow() {
    let fonts = fonts();
    let cases = [
        (
            "\u{645}\u{631}\u{62d}\u{628}\u{627} \u{628}\u{627}\u{644}\u{639}\u{627}\u{644}\u{645}",
            Setting {
                alignment: UiTextAlignment::End,
                wrap: UiTextWrap::Grapheme,
                maximum_lines: 8,
                ..START
            },
        ),
        (
            "\u{5e9}\u{5dc}\u{5d5}\u{5dd} \u{5e2}\u{5d5}\u{5dc}\u{5dd} \u{5d9}\u{5e4}\u{5d4}",
            Setting {
                alignment: UiTextAlignment::End,
                direction: UiTextBaseDirection::RightToLeft,
                maximum_lines: 4,
                ..START
            },
        ),
        (
            "office hours and more",
            Setting {
                wrap: UiTextWrap::Grapheme,
                spacing: (500, 1_500),
                maximum_lines: 8,
                ..START
            },
        ),
        (
            "office hours and more",
            Setting {
                wrap: UiTextWrap::None,
                ..START
            },
        ),
        (
            "office hours and more",
            Setting {
                overflow: UiTextOverflow::Ellipsis,
                maximum_lines: 1,
                ..START
            },
        ),
    ];
    for (source, setting) in cases {
        assert_admitted_widths_qualify(source, setting, &fonts);
    }
}

/// Spacing may pull glyphs together past their own advances. Every flow
/// still lays such a paragraph out, with ordered bounds.
#[test]
fn spacing_that_overlaps_glyphs_still_lays_out() {
    let fonts = fonts();
    let flows = [UiTextOverflow::Clip, UiTextOverflow::Ellipsis].map(|overflow| {
        [
            UiTextWrap::UnicodeWord,
            UiTextWrap::Grapheme,
            UiTextWrap::None,
        ]
        .map(|wrap| (overflow, wrap))
    });
    let placements = [
        UiTextAlignment::Start,
        UiTextAlignment::Center,
        UiTextAlignment::End,
    ]
    .map(|alignment| {
        [
            UiTextBaseDirection::LeftToRight,
            UiTextBaseDirection::RightToLeft,
        ]
        .map(|direction| (alignment, direction))
    });
    for (overflow, wrap) in flows.into_iter().flatten() {
        for (alignment, direction) in placements.into_iter().flatten() {
            let setting = Setting {
                alignment,
                direction,
                wrap,
                overflow,
                spacing: (-12_000, -15_000),
                maximum_lines: 1,
            };
            for source in [
                "office hours and more",
                "\u{5e9}\u{5dc}\u{5d5}\u{5dd} ab",
                "\u{6f22}\u{5b57}\u{6f22}\u{5b57}\u{6f22}\u{5b57}\u{6f22}\u{5b57}\u{6f22}\u{5b57}\u{6f22}\u{5b57}",
                "tab	stops	wrap",
            ] {
                for width in [8_000, 40_000, 160_000] {
                    qualify_with(source, width, setting, &fonts);
                }
            }
        }
    }
}

/// Across a sweep of widths, some of which share a layout, each layout is
/// qualified again at the width that names it, and admits exactly the swept
/// widths that qualify it.
fn assert_admitted_widths_qualify(
    source: &str,
    setting: Setting,
    fonts: &Arc<UiGlobalFontCollection>,
) {
    let sweep = (8_000..=200_000).step_by(4_000).collect::<Vec<u32>>();
    let layouts = sweep
        .iter()
        .map(|width| qualify_with(source, *width, setting, fonts))
        .collect::<Vec<_>>();
    let distinct = layouts
        .iter()
        .map(|layout| layout.identity().digest())
        .collect::<std::collections::BTreeSet<_>>();
    assert!(distinct.len() < layouts.len(), "{source:?} is never reused");
    for layout in &layouts {
        let fitted = layout.view().width_basis().width_millipoints();
        assert!(layout.admits_width(fitted), "{source:?}");
        let at_fitted = qualify_with(source, fitted, setting, fonts);
        assert_eq!(
            at_fitted.identity(),
            layout.identity(),
            "{source:?} {fitted}"
        );
        for (width, other) in sweep.iter().zip(&layouts) {
            assert_eq!(
                layout.admits_width(*width),
                other.identity() == layout.identity(),
                "{source:?} fitted at {fitted}, qualified at {width}"
            );
        }
    }
}

/// How a swept paragraph flows; everything but its width.
#[derive(Clone, Copy)]
struct Setting {
    alignment: UiTextAlignment,
    direction: UiTextBaseDirection,
    wrap: UiTextWrap,
    overflow: UiTextOverflow,
    /// Letter and word spacing.
    spacing: (i32, i32),
    maximum_lines: u32,
}

const START: Setting = Setting {
    alignment: UiTextAlignment::Start,
    direction: UiTextBaseDirection::Auto,
    wrap: UiTextWrap::UnicodeWord,
    overflow: UiTextOverflow::Clip,
    spacing: (0, 0),
    maximum_lines: 2,
};

fn fonts() -> Arc<UiGlobalFontCollection> {
    Arc::new(UiGlobalFontCollection::admit_qualified_profile().unwrap().0)
}

fn qualify(
    source: &str,
    width_millipoints: u32,
    alignment: UiTextAlignment,
    fonts: &Arc<UiGlobalFontCollection>,
) -> crate::UiQualifiedTextLayout {
    request(source, width_millipoints, alignment, fonts)
        .qualify()
        .unwrap()
}

fn qualify_with(
    source: &str,
    width_millipoints: u32,
    setting: Setting,
    fonts: &Arc<UiGlobalFontCollection>,
) -> crate::UiQualifiedTextLayout {
    request_with(source, width_millipoints, setting, fonts)
        .qualify()
        .unwrap()
}

fn request(
    source: &str,
    width_millipoints: u32,
    alignment: UiTextAlignment,
    fonts: &Arc<UiGlobalFontCollection>,
) -> UiQualifiedTextLayoutRequest {
    request_with(
        source,
        width_millipoints,
        Setting { alignment, ..START },
        fonts,
    )
}

fn request_with(
    source: &str,
    width_millipoints: u32,
    setting: Setting,
    fonts: &Arc<UiGlobalFontCollection>,
) -> UiQualifiedTextLayoutRequest {
    let constraints = constraints_with(width_millipoints, setting);
    let input = UiTextParagraphAdmissionInput {
        source: Arc::from(source),
        styles: Box::new([UiTextStyleSpan::whole_paragraph(source, &constraints).unwrap()]),
        constraints,
        profile_generation: UiTextProfileGeneration::new(1).unwrap(),
        font_collection_generation: fonts.generation(),
        text_scale_generation: UiTextScaleGeneration::new(1).unwrap(),
    };
    UiQualifiedTextLayoutRequest::new(input, fonts.clone())
}

fn input(style: UiTextStyle, fonts: &UiGlobalFontCollection) -> UiTextParagraphAdmissionInput {
    let source: Arc<str> = Arc::from("office");
    UiTextParagraphAdmissionInput {
        source: source.clone(),
        constraints: constraints_at(100_000, UiTextAlignment::Start),
        profile_generation: UiTextProfileGeneration::new(1).unwrap(),
        font_collection_generation: fonts.generation(),
        text_scale_generation: UiTextScaleGeneration::new(1).unwrap(),
        styles: Box::new([UiTextStyleSpan::new(
            UiTextOriginalRange::from_text_mechanics(0, source.len() as u32).unwrap(),
            style,
        )
        .unwrap()]),
    }
}

fn constraints_at(
    width_millipoints: u32,
    alignment: UiTextAlignment,
) -> UiTextParagraphConstraints {
    constraints_with(width_millipoints, Setting { alignment, ..START })
}

fn constraints_with(width_millipoints: u32, setting: Setting) -> UiTextParagraphConstraints {
    UiTextParagraphConstraints::new(UiTextParagraphConstraintsInput {
        language: Arc::from("und"),
        base_direction: setting.direction,
        wrap: setting.wrap,
        alignment: setting.alignment,
        overflow: setting.overflow,
        font_size_millipoints: 14_000,
        width_millipoints,
        line_height_millipoints: 18_000,
        letter_spacing_millipoints: setting.spacing.0,
        word_spacing_millipoints: setting.spacing.1,
        tab_interval_millipoints: 56_000,
        maximum_lines: setting.maximum_lines,
    })
    .unwrap()
}

fn style(
    features: Box<[UiOpenTypeFeature]>,
    variations: Box<[UiFontVariationCoordinate]>,
) -> UiTextStyle {
    UiTextStyle::new(UiTextStyleInput {
        language: Arc::from("und"),
        font_size_millipoints: 14_000,
        letter_spacing_millipoints: 0,
        word_spacing_millipoints: 0,
        family_stack: UiFontFamilyStack::profile_sans(),
        face_request: UiTextFaceRequest::new(400, 100_000, UiFontSlant::Upright).unwrap(),
        features,
        variations,
    })
    .unwrap()
}
