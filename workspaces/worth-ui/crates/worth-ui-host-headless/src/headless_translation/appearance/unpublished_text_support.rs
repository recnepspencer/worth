use worth_ui_host_contract::{
    UiFontCollectionGeneration, UiFontSlant, UiQualifiedTextCostRecord,
    UiQualifiedTextLayoutIdentity, UiQualifiedTextLayoutRequestIdentity, UiQualifiedTextLayoutView,
    UiQualifiedTextLayoutViewInput, UiQualifiedTextLayoutWidthBasis, UiQualifiedTextStyleInput,
    UiQualifiedTextStyleRecord, UiTextOriginalRange, UiTextProfileGeneration, UiTextRect,
    UiTextScaleGeneration,
};

pub(super) fn qualified_layout(range: UiTextOriginalRange) -> UiQualifiedTextLayoutView<'static> {
    let source = "ONLINE";
    let styles: &'static [UiQualifiedTextStyleRecord] =
        Box::leak(Box::new([UiQualifiedTextStyleRecord::from_text_mechanics(
            UiQualifiedTextStyleInput {
                original_range: range,
                language: "und".into(),
                font_size_millipoints: 14_000,
                letter_spacing_millipoints: 0,
                word_spacing_millipoints: 0,
                family_stack: Box::new([]),
                weight: 400,
                width_milli_percent: 100_000,
                slant: UiFontSlant::Upright,
                features: Box::new([]),
                variations: Box::new([]),
            },
        )]));
    UiQualifiedTextLayoutView::from_text_mechanics(UiQualifiedTextLayoutViewInput {
        request_identity: UiQualifiedTextLayoutRequestIdentity::from_text_mechanics([106; 32]),
        identity: UiQualifiedTextLayoutIdentity::from_text_mechanics([107; 32]),
        source,
        graphemes: &[],
        word_boundaries: &[],
        styles,
        logical_runs: &[],
        glyphs: &[],
        lines: &[],
        visual_runs: &[],
        positioned_glyphs: &[],
        logical_bounds: UiTextRect::from_text_mechanics(0, 0, 80_000, 18_000).unwrap(),
        ink_bounds: UiTextRect::from_text_mechanics(0, 2_000, 78_000, 16_000).unwrap(),
        carets: &[],
        coverage: &[],
        cost: UiQualifiedTextCostRecord::default(),
        profile: UiTextProfileGeneration::new(1).unwrap(),
        font_collection: UiFontCollectionGeneration::new(1).unwrap(),
        text_scale: UiTextScaleGeneration::new(1).unwrap(),
        width_basis: UiQualifiedTextLayoutWidthBasis::new(80_000).unwrap(),
    })
}
