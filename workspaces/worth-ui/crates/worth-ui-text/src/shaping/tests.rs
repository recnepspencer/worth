use std::sync::Arc;

use super::*;
use crate::{
    font_collection::profile_inputs_from_repository, UiAdmittedTextParagraph,
    UiAnalyzedTextParagraph, UiGlobalFontCollection, UiTextAlignment, UiTextBaseDirection,
    UiTextOverflow, UiTextParagraphAdmissionInput, UiTextParagraphConstraints,
    UiTextParagraphConstraintsInput, UiTextProfileGeneration, UiTextScaleGeneration, UiTextWrap,
};
use worth_ui_host_contract::UiFontCollectionGeneration;

#[test]
pub(crate) fn mixed_latin_arabic_indic_and_emoji_shape_in_exhaustive_runs_with_original_ranges() {
    let generation = UiFontCollectionGeneration::new(1).unwrap();
    let (fonts, _) =
        UiGlobalFontCollection::admit_profile(generation, profile_inputs_from_repository())
            .unwrap();
    let fonts = Arc::new(fonts);
    let source = "office \u{645}\u{631}\u{62D}\u{628}\u{627} \u{915}\u{94D}\u{937} \u{1F469}\u{1F3FD}\u{200D}\u{1F4BB}";
    let fallback =
        UiFallbackTextParagraph::select(analyze(source, generation), Arc::clone(&fonts)).unwrap();
    let shaped = UiShapedTextParagraph::shape(fallback).unwrap();
    assert!(shaped.runs().len() >= 4);
    assert!(shaped.glyphs().iter().all(|glyph| glyph.glyph_id() != 0));
    assert!(shaped.glyphs().iter().all(|glyph| {
        let range = glyph.original_range();
        range.start() < range.end() && range.end() <= source.len() as u32
    }));
    assert_eq!(shaped.cost.runs_shaped(), shaped.runs().len() as u32);
    assert_eq!(shaped.cost.glyphs_emitted(), shaped.glyphs().len() as u32);
}

#[test]
pub(crate) fn each_complete_missing_cluster_emits_one_glyph_with_its_exact_original_range() {
    let generation = UiFontCollectionGeneration::new(1).unwrap();
    let (fonts, _) =
        UiGlobalFontCollection::admit_profile(generation, profile_inputs_from_repository())
            .unwrap();
    let fonts = Arc::new(fonts);
    let source = "\u{0378}\u{0301}\u{0379}\u{0300}";
    let fallback =
        UiFallbackTextParagraph::select(analyze(source, generation), Arc::clone(&fonts)).unwrap();
    let missing_ranges = fallback
        .clusters()
        .iter()
        .map(|cluster| {
            assert_eq!(
                cluster.coverage(),
                crate::UiTextCoverageDisposition::MissingCluster
            );
            cluster.original_range()
        })
        .collect::<Vec<_>>();
    assert_eq!(missing_ranges.len(), 2);

    let shaped = UiShapedTextParagraph::shape(fallback).unwrap();

    assert_eq!(shaped.runs().len(), missing_ranges.len());
    assert_eq!(shaped.glyphs().len(), missing_ranges.len());
    assert_eq!(shaped.cost.glyphs_emitted(), 2);
    for (glyph, expected_range) in shaped.glyphs().iter().zip(missing_ranges) {
        assert_ne!(glyph.glyph_id(), 0);
        assert_eq!(glyph.original_range(), expected_range);
    }
}

#[test]
pub(crate) fn authored_feature_spans_partition_runs_and_change_real_glyph_formation() {
    let generation = UiFontCollectionGeneration::new(1).unwrap();
    let (fonts, _) =
        UiGlobalFontCollection::admit_profile(generation, profile_inputs_from_repository())
            .unwrap();
    let fonts = Arc::new(fonts);
    let source = "ffiffi";
    let paragraph_constraints = constraints();
    let style = |start, end, enabled| {
        crate::UiTextStyleSpan::new(
            worth_ui_host_contract::UiTextOriginalRange::from_text_mechanics(start, end).unwrap(),
            crate::UiTextStyle::new(crate::UiTextStyleInput {
                language: Arc::from("en"),
                font_size_millipoints: 14_000,
                letter_spacing_millipoints: 0,
                word_spacing_millipoints: 0,
                family_stack: crate::UiFontFamilyStack::profile_sans(),
                face_request: crate::UiTextFaceRequest::regular(),
                features: Box::new([
                    crate::UiOpenTypeFeature::new(*b"liga", u32::from(enabled)).unwrap()
                ]),
                variations: Box::new([]),
            })
            .unwrap(),
        )
        .unwrap()
    };
    let (admitted, _) = UiAdmittedTextParagraph::admit(UiTextParagraphAdmissionInput {
        source: Arc::from(source),
        constraints: paragraph_constraints,
        profile_generation: UiTextProfileGeneration::new(1).unwrap(),
        font_collection_generation: generation,
        text_scale_generation: UiTextScaleGeneration::new(1).unwrap(),
        styles: Box::new([style(0, 3, false), style(3, 6, true)]),
    })
    .unwrap();
    let fallback = UiFallbackTextParagraph::select(
        UiAnalyzedTextParagraph::analyze(admitted),
        Arc::clone(&fonts),
    )
    .unwrap();
    let shaped = UiShapedTextParagraph::shape(fallback).unwrap();

    assert_eq!(shaped.runs().len(), 2);
    assert_eq!(shaped.runs()[0].style_index(), 0);
    assert_eq!(shaped.runs()[1].style_index(), 1);
    assert_eq!(shaped.runs()[0].glyph_range().len(), 3);
    assert!(shaped.runs()[1].glyph_range().len() < 3);
}

#[test]
pub(crate) fn qualified_width_axes_change_real_advances_and_out_of_range_axes_are_denied() {
    let generation = UiFontCollectionGeneration::new(1).unwrap();
    let (fonts, _) =
        UiGlobalFontCollection::admit_profile(generation, profile_inputs_from_repository())
            .unwrap();
    let fonts = Arc::new(fonts);
    let source = "MMMMMMMM";
    let paragraph_constraints = constraints();
    let style = |start, end, width_milli| {
        crate::UiTextStyleSpan::new(
            worth_ui_host_contract::UiTextOriginalRange::from_text_mechanics(start, end).unwrap(),
            crate::UiTextStyle::new(crate::UiTextStyleInput {
                language: Arc::from("en"),
                font_size_millipoints: 14_000,
                letter_spacing_millipoints: 0,
                word_spacing_millipoints: 0,
                family_stack: crate::UiFontFamilyStack::profile_sans(),
                face_request: crate::UiTextFaceRequest::new(
                    400,
                    u32::try_from(width_milli).unwrap(),
                    worth_ui_host_contract::UiFontSlant::Upright,
                )
                .unwrap(),
                features: Box::new([]),
                variations: Box::new([]),
            })
            .unwrap(),
        )
        .unwrap()
    };
    let (admitted, _) = UiAdmittedTextParagraph::admit(UiTextParagraphAdmissionInput {
        source: Arc::from(source),
        constraints: paragraph_constraints,
        profile_generation: UiTextProfileGeneration::new(1).unwrap(),
        font_collection_generation: generation,
        text_scale_generation: UiTextScaleGeneration::new(1).unwrap(),
        styles: Box::new([style(0, 4, 62_500), style(4, 8, 100_000)]),
    })
    .unwrap();
    let fallback = UiFallbackTextParagraph::select(
        UiAnalyzedTextParagraph::analyze(admitted),
        Arc::clone(&fonts),
    )
    .unwrap();
    let shaped = UiShapedTextParagraph::shape(fallback).unwrap();

    assert_eq!(shaped.runs().len(), 2);
    assert_eq!(shaped.runs()[0].style_index(), 0);
    assert_eq!(shaped.runs()[1].style_index(), 1);
    let advance = |run_index: usize| {
        let range = shaped.runs()[run_index].glyph_range();
        shaped.glyphs()[range.start as usize..range.end as usize]
            .iter()
            .map(|glyph| glyph.x_advance_font_units())
            .sum::<i32>()
    };
    assert!(advance(0) < advance(1));

    let constraints = constraints();
    let explicit_out_of_range = crate::UiTextStyleSpan::new(
        worth_ui_host_contract::UiTextOriginalRange::from_text_mechanics(0, 1).unwrap(),
        crate::UiTextStyle::new(crate::UiTextStyleInput {
            language: Arc::from("en"),
            font_size_millipoints: 14_000,
            letter_spacing_millipoints: 0,
            word_spacing_millipoints: 0,
            family_stack: crate::UiFontFamilyStack::profile_sans(),
            face_request: crate::UiTextFaceRequest::regular(),
            features: Box::new([]),
            variations: Box::new([
                crate::UiFontVariationCoordinate::new(*b"wdth", 200_000).unwrap()
            ]),
        })
        .unwrap(),
    )
    .unwrap();
    let (admitted, _) = UiAdmittedTextParagraph::admit(UiTextParagraphAdmissionInput {
        source: Arc::from("M"),
        constraints,
        profile_generation: UiTextProfileGeneration::new(1).unwrap(),
        font_collection_generation: generation,
        text_scale_generation: UiTextScaleGeneration::new(1).unwrap(),
        styles: Box::new([explicit_out_of_range]),
    })
    .unwrap();
    let denial = match UiFallbackTextParagraph::select(
        UiAnalyzedTextParagraph::analyze(admitted),
        Arc::clone(&fonts),
    ) {
        Ok(_) => panic!("out-of-range width coordinate was admitted"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial,
        crate::UiTextFallbackDenial::UnsupportedVariationCoordinate
    );
}

fn analyze(source: &str, generation: UiFontCollectionGeneration) -> UiAnalyzedTextParagraph {
    let constraints = constraints();
    let styles = Box::new([
        crate::UiTextStyleSpan::whole_paragraph(source, &constraints).expect("nonempty source"),
    ]);
    let (admitted, _) = UiAdmittedTextParagraph::admit(UiTextParagraphAdmissionInput {
        source: Arc::from(source),
        constraints,
        profile_generation: UiTextProfileGeneration::new(1).unwrap(),
        font_collection_generation: generation,
        text_scale_generation: UiTextScaleGeneration::new(1).unwrap(),
        styles,
    })
    .unwrap();
    UiAnalyzedTextParagraph::analyze(admitted)
}

fn constraints() -> UiTextParagraphConstraints {
    UiTextParagraphConstraints::new(UiTextParagraphConstraintsInput {
        language: Arc::from("und"),
        base_direction: UiTextBaseDirection::Auto,
        wrap: UiTextWrap::UnicodeWord,
        alignment: UiTextAlignment::Start,
        overflow: UiTextOverflow::Clip,
        font_size_millipoints: 14_000,
        width_millipoints: 320_000,
        line_height_millipoints: 18_000,
        letter_spacing_millipoints: 0,
        word_spacing_millipoints: 0,
        tab_interval_millipoints: 56_000,
        maximum_lines: 16,
    })
    .unwrap()
}
