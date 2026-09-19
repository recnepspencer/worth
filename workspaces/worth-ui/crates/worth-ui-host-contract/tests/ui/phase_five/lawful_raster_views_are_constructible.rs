use worth_ui_host_contract::{
    UiFontCollectionGeneration, UiFontCollectionLineageIdentity, UiGlyphRasterDemandBatchView,
    UiGlyphRasterDemandBatchViewInput,
    UiGlyphRasterDemandIdentity, UiGlyphRasterFractionalOrigin, UiGlyphRasterKey,
    UiGlyphRasterKeyInput, UiGlyphRasterLane, UiGlyphRasterPalette, UiGlyphRasterSize,
    UiGlyphRasterSource, UiGlyphVariationCoordinates, UiQualifiedFontFaceIdentity,
    UiQualifiedTextLayoutIdentity, UiTextProfileGeneration, UiTextScaleGeneration,
};

fn main() {
    let key = UiGlyphRasterKey::from_text_mechanics(UiGlyphRasterKeyInput {
        font_collection: UiFontCollectionGeneration::new(1).unwrap(),
        font_collection_lineage: UiFontCollectionLineageIdentity::from_text_mechanics([4; 32]),
        profile: UiTextProfileGeneration::new(1).unwrap(),
        face: UiQualifiedFontFaceIdentity::from_text_mechanics([1; 32], 0),
        glyph_id: 1,
        variations: UiGlyphVariationCoordinates::empty(),
        palette: UiGlyphRasterPalette::new(0),
        size: UiGlyphRasterSize::from_millipoints(12_000).unwrap(),
        source: UiGlyphRasterSource::AlphaOutline,
        dpi_milli: 96,
        origin: UiGlyphRasterFractionalOrigin::from_sixty_fourths(0, 0),
    })
    .unwrap();
    let records = [];
    let view = UiGlyphRasterDemandBatchView::from_text_mechanics(UiGlyphRasterDemandBatchViewInput {
        identity: UiGlyphRasterDemandIdentity::from_text_mechanics([2; 32]),
        layout: UiQualifiedTextLayoutIdentity::from_text_mechanics([3; 32]),
        dpi_milli: 96,
        text_scale: UiTextScaleGeneration::new(1).unwrap(),
        lane: UiGlyphRasterLane::Ordinary,
        scope: worth_ui_host_contract::UiGlyphRasterDemandScope::DamageFiltered,
        records: &records,
    })
    .unwrap();
    assert_eq!(view.records().len(), 0);
    assert_eq!(key.glyph_id(), 1);
}
