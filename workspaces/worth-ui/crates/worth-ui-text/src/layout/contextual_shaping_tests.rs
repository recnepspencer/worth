use std::sync::Arc;

use worth_ui_host_contract::{
    UiFontCollectionGeneration, UiTextProfileGeneration, UiTextScaleGeneration,
};

use crate::{
    font_collection::profile_inputs_from_repository, UiAdmittedTextParagraph,
    UiAnalyzedTextParagraph, UiFallbackTextParagraph, UiGlobalFontCollection,
    UiShapedTextParagraph, UiTextAlignment, UiTextBaseDirection, UiTextOverflow,
    UiTextParagraphAdmissionInput, UiTextParagraphConstraints, UiTextParagraphConstraintsInput,
    UiTextStyleSpan, UiTextWrap,
};

use super::UiQualifiedTextLayout;

#[test]
pub(crate) fn arabic_and_indic_soft_lines_match_independently_shaped_line_segments() {
    let generation = UiFontCollectionGeneration::new(1).unwrap();
    let (fonts, _) =
        UiGlobalFontCollection::admit_profile(generation, profile_inputs_from_repository())
            .unwrap();
    let fonts = Arc::new(fonts);
    for source in ["سلامعليكمورحبا", "ܫܬܡܐܫܬܡܐܫܬܡܐ", "क्षणक्षणक्षण"]
    {
        let wrapped = layout(source, 18_000, 16, UiTextWrap::Grapheme, &fonts);
        assert!(wrapped.lines().len() > 1, "fixture must cross a soft line");
        for line in wrapped.lines() {
            let range = line.original_range();
            if range.is_empty() {
                continue;
            }
            let segment = &source[range.start() as usize..range.end() as usize];
            let independent = layout(segment, 500_000, 1, UiTextWrap::None, &fonts);
            let wrapped_records = wrapped
                .glyphs()
                .iter()
                .filter(|glyph| {
                    let glyph = glyph.original_range();
                    glyph.start() >= range.start() && glyph.end() <= range.end()
                })
                .map(|glyph| normalized_glyph(*glyph, range.start()))
                .collect::<Vec<_>>();
            let independent_records = independent
                .glyphs()
                .iter()
                .map(|glyph| normalized_glyph(*glyph, 0))
                .collect::<Vec<_>>();
            assert_eq!(
                wrapped_records, independent_records,
                "line must own exact glyph IDs, ranges, advances, offsets, and ink"
            );
        }
    }
}

fn normalized_glyph(
    glyph: worth_ui_host_contract::UiQualifiedTextGlyphRecord,
    line_start: u32,
) -> (
    u32,
    u32,
    u32,
    i32,
    i32,
    i32,
    i32,
    worth_ui_host_contract::UiTextFontUnitRect,
) {
    let range = glyph.original_range();
    (
        glyph.glyph_id(),
        range.start() - line_start,
        range.end() - line_start,
        glyph.x_advance_font_units(),
        glyph.y_advance_font_units(),
        glyph.x_offset_font_units(),
        glyph.y_offset_font_units(),
        glyph.ink_bounds_font_units(),
    )
}

#[test]
pub(crate) fn contextual_line_fragments_do_not_consume_logical_run_capacity() {
    let generation = UiFontCollectionGeneration::new(1).unwrap();
    let (fonts, _) =
        UiGlobalFontCollection::admit_profile(generation, profile_inputs_from_repository())
            .unwrap();
    let source = "سلام".repeat(48);
    let layout = layout(&source, 8_000, 128, UiTextWrap::Grapheme, &Arc::new(fonts));
    assert!(
        layout.lines().len() > 32,
        "fixture must exceed the logical run cap"
    );
    assert_eq!(
        layout.artifact().view().logical_runs().len(),
        1,
        "line-local shaping fragments must retain one semantic run identity"
    );
}

#[test]
fn ordinary_sentence_spaces_preserve_one_latin_shaping_run() {
    let (fonts, _) = UiGlobalFontCollection::admit_qualified_profile().unwrap();
    let source = "Please review the changes before proceeding. ".repeat(12);
    let layout = layout(
        &source,
        500_000,
        32,
        UiTextWrap::UnicodeWord,
        &Arc::new(fonts),
    );
    assert_eq!(layout.artifact().view().logical_runs().len(), 1);
    assert!(layout.lines().len() > 1);
    assert_eq!(layout.artifact().view().source(), source.as_str());
}
fn layout(
    source: &str,
    width_millipoints: u32,
    maximum_lines: u32,
    wrap: UiTextWrap,
    fonts: &Arc<UiGlobalFontCollection>,
) -> UiQualifiedTextLayout {
    let generation = UiFontCollectionGeneration::new(1).unwrap();
    let constraints = UiTextParagraphConstraints::new(UiTextParagraphConstraintsInput {
        language: Arc::from("und"),
        base_direction: UiTextBaseDirection::Auto,
        wrap,
        alignment: UiTextAlignment::Start,
        overflow: UiTextOverflow::Clip,
        font_size_millipoints: 14_000,
        width_millipoints,
        line_height_millipoints: 18_000,
        letter_spacing_millipoints: 0,
        word_spacing_millipoints: 0,
        tab_interval_millipoints: 56_000,
        maximum_lines,
    })
    .unwrap();
    let (admitted, _) = UiAdmittedTextParagraph::admit(UiTextParagraphAdmissionInput {
        source: Arc::from(source),
        constraints: constraints.clone(),
        profile_generation: UiTextProfileGeneration::new(1).unwrap(),
        font_collection_generation: generation,
        text_scale_generation: UiTextScaleGeneration::new(1).unwrap(),
        styles: Box::new([UiTextStyleSpan::whole_paragraph(source, &constraints).unwrap()]),
    })
    .unwrap();
    let analyzed = UiAnalyzedTextParagraph::analyze(admitted);
    let fallback = UiFallbackTextParagraph::select(analyzed, Arc::clone(fonts)).unwrap();
    let shaped = UiShapedTextParagraph::shape(fallback).unwrap();
    UiQualifiedTextLayout::layout(shaped).unwrap()
}
