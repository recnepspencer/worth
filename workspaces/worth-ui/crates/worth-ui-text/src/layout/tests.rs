use std::sync::Arc;

use worth_ui_host_contract::{
    UiFontCollectionGeneration, UiQualifiedTextCostInput, UiQualifiedTextCostRecord,
    UiQualifiedTextWordBoundaryRecord, UiTextOriginalRange, UiTextPoint, UiTextProfileGeneration,
    UiTextScaleGeneration,
};

use super::*;
use crate::{
    font_collection::profile_inputs_from_repository, UiAdmittedTextParagraph,
    UiAnalyzedTextParagraph, UiFallbackTextParagraph, UiGlobalFontCollection,
    UiShapedTextParagraph, UiTextAlignment, UiTextBaseDirection, UiTextOverflow,
    UiTextParagraphAdmissionInput, UiTextParagraphConstraints, UiTextParagraphConstraintsInput,
    UiTextStyleSpan, UiTextWrap,
};

#[test]
pub(crate) fn canonical_layout_identity_changes_when_its_exact_cost_record_changes() {
    let generation = UiFontCollectionGeneration::new(1).unwrap();
    let (fonts, _) =
        UiGlobalFontCollection::admit_profile(generation, profile_inputs_from_repository())
            .unwrap();
    let fonts = Arc::new(fonts);
    let constraints = UiTextParagraphConstraints::new(UiTextParagraphConstraintsInput {
        language: Arc::from("und"),
        base_direction: UiTextBaseDirection::Auto,
        wrap: UiTextWrap::UnicodeWord,
        alignment: UiTextAlignment::Start,
        overflow: UiTextOverflow::Clip,
        font_size_millipoints: 14_000,
        width_millipoints: 80_000,
        line_height_millipoints: 18_000,
        letter_spacing_millipoints: 0,
        word_spacing_millipoints: 0,
        tab_interval_millipoints: 56_000,
        maximum_lines: 1,
    })
    .unwrap();
    let source = "a";
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
    let fallback = UiFallbackTextParagraph::select(analyzed, fonts).unwrap();
    let shaped = UiShapedTextParagraph::shape(fallback).unwrap();
    let identity = |word_boundaries: &[UiQualifiedTextWordBoundaryRecord], cost| {
        identity::for_layout(identity::UiQualifiedTextLayoutIdentityInput {
            shaped: &shaped,
            request_identity: crate::request::UiQualifiedTextRequestParts::of_shaped(
                &shaped,
                shaped.fonts(),
            )
            .identity_at(shaped.constraints().width_millipoints()),
            width_millipoints: shaped.constraints().width_millipoints(),
            word_boundaries,
            logical_runs: &[],
            logical_glyphs: &[],
            styles: &[],
            lines: &[],
            visual_runs: &[],
            positioned_glyphs: &[],
            carets: &[],
            coverage: &[],
            faces: &[],
            cost,
        })
    };
    let changed = UiQualifiedTextCostRecord::from_text_mechanics(UiQualifiedTextCostInput {
        analyzed_bytes: 1,
        graphemes: 0,
        word_boundaries: 0,
        line_opportunities: 0,
        bidi_contexts: 0,
        fallback_clusters: 0,
        coverage_index_queries: 0,
        face_shape_attempts: 0,
        probed_glyphs: 0,
        shaped_runs: 0,
        shaped_scalars: 0,
        emitted_glyphs: 0,
        fitted_units: 0,
        emitted_lines: 0,
        emitted_visual_runs: 0,
        positioned_glyphs: 0,
        emitted_carets: 0,
    });
    assert_ne!(
        identity(&[], UiQualifiedTextCostRecord::default()),
        identity(&[], changed)
    );
    let boundary = UiQualifiedTextWordBoundaryRecord::from_text_mechanics(
        UiTextOriginalRange::new(0, 0).unwrap(),
    );
    assert_ne!(
        identity(&[], UiQualifiedTextCostRecord::default()),
        identity(&[boundary], UiQualifiedTextCostRecord::default())
    );
}

#[test]
pub(crate) fn mixed_direction_layout_carries_visual_runs_carets_hits_and_discontiguous_selection() {
    let source = "abc \u{5e9}\u{5dc}\u{5d5}\u{5dd} \u{1f469}\u{1f3fd}\u{200d}\u{1f4bb} xyz";
    let layout = layout(source, 180_000, 8);

    assert!(!layout.lines().is_empty());
    assert!(layout
        .visual_runs()
        .iter()
        .any(|run| !run.bidi_level().is_multiple_of(2)));
    assert!(layout
        .visual_runs()
        .iter()
        .any(|run| run.bidi_level().is_multiple_of(2)));
    assert!(layout
        .carets()
        .iter()
        .all(|caret| caret.position().original_boundary().is_empty()));
    let emoji_start = u32::try_from(source.find('\u{1f469}').unwrap()).unwrap();
    let emoji_end =
        emoji_start + u32::try_from("\u{1f469}\u{1f3fd}\u{200d}\u{1f4bb}".len()).unwrap();
    let emoji_carets = layout
        .carets()
        .iter()
        .copied()
        .filter(|caret| {
            let position = caret.position();
            (position.original_boundary().start() == emoji_start
                && position.visual_edge() == worth_ui_host_contract::UiTextVisualEdge::Leading)
                || (position.original_boundary().start() == emoji_end
                    && position.visual_edge() == worth_ui_host_contract::UiTextVisualEdge::Trailing)
        })
        .collect::<Vec<_>>();
    assert_eq!(emoji_carets.len(), 2);
    let hit = layout
        .hit_test(UiTextPoint::from_text_mechanics(
            (emoji_carets[0].x_millipoints() + emoji_carets[1].x_millipoints()) / 2,
            (emoji_carets[0].top_millipoints() + emoji_carets[0].bottom_millipoints()) / 2,
        ))
        .unwrap();
    assert_eq!(
        (hit.cluster_range().start(), hit.cluster_range().end()),
        (emoji_start, emoji_end)
    );
    let selection = layout
        .selection_rects(UiTextOriginalRange::from_text_mechanics(0, source.len() as u32).unwrap())
        .unwrap();
    assert!(
        selection.len() >= 3,
        "bidi selection must remain per visual run"
    );
}

#[test]
pub(crate) fn unicode_wrap_preserves_cluster_boundaries_and_exact_line_capacity() {
    let source = "office \u{1f469}\u{1f3fd}\u{200d}\u{1f4bb} office office";
    let layout = layout(source, 55_000, 2);
    assert_eq!(layout.lines().len(), 2);
    assert!(layout.lines().last().unwrap().overflowed());
    for line in layout.lines() {
        assert!(source.is_char_boundary(line.original_range().start() as usize));
        assert!(source.is_char_boundary(line.original_range().end() as usize));
    }
}

#[test]
pub(crate) fn first_cluster_wider_than_the_line_is_reported_as_overflowing() {
    let layout = layout_with_overflow("WW", 1, 2, UiTextOverflow::Clip);
    assert!(layout.lines().len() >= 2);
    assert!(layout.lines()[0].overflowed());
}

#[test]
pub(crate) fn ellipsis_is_a_shaped_cluster_and_never_splits_an_rgi_emoji() {
    let source = "prefix \u{1f468}\u{200d}\u{1f469}\u{200d}\u{1f467}\u{200d}\u{1f466} suffix";
    let layout = layout_with_overflow(source, 52_000, 1, UiTextOverflow::Ellipsis);
    assert!(layout.lines()[0].overflowed());
    let ellipsis = layout
        .positioned_glyphs()
        .last()
        .map(|positioned| layout.glyphs()[positioned.source_glyph_index() as usize])
        .unwrap();
    assert!(ellipsis.original_range().is_empty());
    let emoji_start = u32::try_from(source.find('\u{1f468}').unwrap()).unwrap();
    let emoji_end = emoji_start
        + u32::try_from("\u{1f468}\u{200d}\u{1f469}\u{200d}\u{1f467}\u{200d}\u{1f466}".len())
            .unwrap();
    assert!(layout.positioned_glyphs().iter().all(|positioned| {
        let range = layout.glyphs()[positioned.source_glyph_index() as usize].original_range();
        range.is_empty()
            || range.end() <= emoji_start
            || range.start() >= emoji_end
            || (range.start() == emoji_start && range.end() == emoji_end)
    }));
}

#[test]
fn rtl_ellipsis_occupies_the_visual_end_without_changing_original_boundaries() {
    let source =
        "\u{5e9}\u{5dc}\u{5d5}\u{5dd} \u{5e2}\u{5d5}\u{5dc}\u{5dd} \u{5d0}\u{5e8}\u{5d5}\u{5da}";
    let layout = layout_with_overflow(source, 42_000, 1, UiTextOverflow::Ellipsis);
    let ellipsis = layout
        .positioned_glyphs()
        .iter()
        .copied()
        .find(|positioned| {
            layout.glyphs()[positioned.source_glyph_index() as usize]
                .original_range()
                .is_empty()
        })
        .unwrap();
    let leftmost = layout
        .positioned_glyphs()
        .iter()
        .map(|glyph| glyph.origin_x_millipoints())
        .min()
        .unwrap();
    assert_eq!(ellipsis.origin_x_millipoints(), leftmost);
}

#[test]
pub(crate) fn ellipsis_cannot_publish_a_run_beyond_the_qualified_capacity() {
    let generation = UiFontCollectionGeneration::new(1).unwrap();
    let source = "a".repeat(crate::UiGlobalTextProfile::MAX_RUNS_PER_PARAGRAPH);
    let constraints = UiTextParagraphConstraints::new(UiTextParagraphConstraintsInput {
        language: Arc::from("und"),
        base_direction: UiTextBaseDirection::Auto,
        wrap: UiTextWrap::UnicodeWord,
        alignment: UiTextAlignment::Start,
        overflow: UiTextOverflow::Ellipsis,
        font_size_millipoints: 14_000,
        width_millipoints: 1_000,
        line_height_millipoints: 18_000,
        letter_spacing_millipoints: 0,
        word_spacing_millipoints: 0,
        tab_interval_millipoints: 56_000,
        maximum_lines: 1,
    })
    .unwrap();
    let styles = (0..source.len())
        .map(|offset| {
            UiTextStyleSpan::new(
                UiTextOriginalRange::from_text_mechanics(offset as u32, offset as u32 + 1).unwrap(),
                crate::UiTextStyle::from_paragraph_constraints(&constraints),
            )
            .unwrap()
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let admission = UiAdmittedTextParagraph::admit(UiTextParagraphAdmissionInput {
        source: Arc::from(source),
        constraints,
        profile_generation: UiTextProfileGeneration::new(1).unwrap(),
        font_collection_generation: generation,
        text_scale_generation: UiTextScaleGeneration::new(1).unwrap(),
        styles,
    });
    assert!(matches!(
        admission,
        Err(crate::UiTextParagraphAdmissionDenial::DerivedCapacityExceeded)
    ));
}

pub(crate) fn layout(
    source: &str,
    width_millipoints: u32,
    maximum_lines: u32,
) -> UiQualifiedTextLayout {
    layout_with_overflow(
        source,
        width_millipoints,
        maximum_lines,
        UiTextOverflow::Clip,
    )
}

fn layout_with_overflow(
    source: &str,
    width_millipoints: u32,
    maximum_lines: u32,
    overflow: UiTextOverflow,
) -> UiQualifiedTextLayout {
    let generation = UiFontCollectionGeneration::new(1).unwrap();
    let (fonts, _) =
        UiGlobalFontCollection::admit_profile(generation, profile_inputs_from_repository())
            .unwrap();
    let fonts = Arc::new(fonts);
    layout_with_fonts(
        source,
        width_millipoints,
        maximum_lines,
        overflow,
        generation,
        &fonts,
    )
}

pub(super) fn layout_with_alignment(
    source: &str,
    alignment: UiTextAlignment,
) -> UiQualifiedTextLayout {
    let generation = UiFontCollectionGeneration::new(1).unwrap();
    let (fonts, _) =
        UiGlobalFontCollection::admit_profile(generation, profile_inputs_from_repository())
            .unwrap();
    let fonts = Arc::new(fonts);
    let constraints = constraints(240_000, 4, UiTextOverflow::Clip, alignment);
    layout_with_constraints(source, generation, &fonts, constraints)
}

pub(super) fn layout_with_fonts(
    source: &str,
    width_millipoints: u32,
    maximum_lines: u32,
    overflow: UiTextOverflow,
    generation: UiFontCollectionGeneration,
    fonts: &Arc<UiGlobalFontCollection>,
) -> UiQualifiedTextLayout {
    let constraints = constraints(
        width_millipoints,
        maximum_lines,
        overflow,
        UiTextAlignment::Start,
    );
    layout_with_constraints(source, generation, fonts, constraints)
}

fn constraints(
    width_millipoints: u32,
    maximum_lines: u32,
    overflow: UiTextOverflow,
    alignment: UiTextAlignment,
) -> UiTextParagraphConstraints {
    UiTextParagraphConstraints::new(UiTextParagraphConstraintsInput {
        language: Arc::from("und"),
        base_direction: UiTextBaseDirection::Auto,
        wrap: UiTextWrap::UnicodeWord,
        alignment,
        overflow,
        font_size_millipoints: 14_000,
        width_millipoints,
        line_height_millipoints: 18_000,
        letter_spacing_millipoints: 0,
        word_spacing_millipoints: 0,
        tab_interval_millipoints: 56_000,
        maximum_lines,
    })
    .unwrap()
}

fn layout_with_constraints(
    source: &str,
    generation: UiFontCollectionGeneration,
    fonts: &Arc<UiGlobalFontCollection>,
    constraints: UiTextParagraphConstraints,
) -> UiQualifiedTextLayout {
    let styles: Box<[UiTextStyleSpan]> = if source.is_empty() {
        Box::new([])
    } else {
        Box::new([UiTextStyleSpan::whole_paragraph(source, &constraints).unwrap()])
    };
    let (admitted, _) = UiAdmittedTextParagraph::admit(UiTextParagraphAdmissionInput {
        source: Arc::from(source),
        constraints,
        profile_generation: UiTextProfileGeneration::new(1).unwrap(),
        font_collection_generation: generation,
        text_scale_generation: UiTextScaleGeneration::new(1).unwrap(),
        styles,
    })
    .unwrap();
    let analyzed = UiAnalyzedTextParagraph::analyze(admitted);
    let fallback = UiFallbackTextParagraph::select(analyzed, Arc::clone(fonts)).unwrap();
    let shaped = UiShapedTextParagraph::shape(fallback).unwrap();
    UiQualifiedTextLayout::layout(shaped).unwrap()
}
