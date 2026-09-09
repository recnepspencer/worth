use worth_ui_host_contract::{
    UiGlyphRasterDemandBatchView, UiGlyphRasterDemandBatchViewInput, UiGlyphRasterDemandIdentity,
    UiGlyphRasterDemandRecord, UiGlyphRasterLane, UiQualifiedTextLayoutIdentity,
    UiTextScaleGeneration,
};

fn escape_demand_view<'a>() -> UiGlyphRasterDemandBatchView<'a> {
    let records: [UiGlyphRasterDemandRecord; 0] = [];
    UiGlyphRasterDemandBatchView::from_text_mechanics(UiGlyphRasterDemandBatchViewInput {
        identity: UiGlyphRasterDemandIdentity::from_text_mechanics([1; 32]),
        layout: UiQualifiedTextLayoutIdentity::from_text_mechanics([2; 32]),
        dpi_milli: 96,
        text_scale: UiTextScaleGeneration::new(1).unwrap(),
        lane: UiGlyphRasterLane::Ordinary,
        scope: worth_ui_host_contract::UiGlyphRasterDemandScope::DamageFiltered,
        records: &records,
    })
    .unwrap()
}

fn main() {
    let _ = escape_demand_view;
}
