//! Demand derivation reports its work to the presentation work ledger: one
//! glyph per positioned glyph it visits, one map insert per demanded key, and
//! the bytes its identity digests. Nothing else is charged to it.

use worth_ui_host_contract::{
    take_presentation_work, UiMountedRgba8, UiMountedTextForegroundSpan,
    UiMountedTextPaintSpanIdentity, UiPresentationWorkStage, UiTextOriginalRange,
};

use super::demand_alpha_tests::layout_for;
use super::*;

fn derive(
    layout: &crate::UiQualifiedTextLayout,
    source: &str,
    selection: UiGlyphRasterDemandSelection<'_>,
) -> UiGlyphRasterDemandBatch {
    let paint = UiMountedTextForegroundSpan::from_runtime_mounting(
        UiTextOriginalRange::new(0, source.len() as u32).unwrap(),
        UiMountedRgba8::new(255, 255, 255, 255),
        UiMountedTextPaintSpanIdentity::from_runtime_mounting([37; 32]),
    );
    derive_glyph_raster_demand(
        layout,
        UiGlyphRasterDemandRequest {
            paint_spans: &[paint],
            selection,
            scale: UiGlyphRasterScale::new(1_000, layout.view().text_scale_generation()).unwrap(),
            placement: UiGlyphRasterPlacement::default(),
            lane: UiGlyphRasterLane::Ordinary,
        },
    )
    .unwrap()
}

fn other_stages_are_idle(work: worth_ui_host_contract::UiPresentationWorkCounts) {
    for stage in UiPresentationWorkStage::ALL {
        if stage != UiPresentationWorkStage::DemandDerivation {
            assert_eq!(work.glyphs(stage), 0, "{}", stage.name());
        }
    }
}

#[test]
fn deriving_a_complete_layout_counts_each_visit_and_each_demanded_key() {
    let source = "Wave";
    let layout = layout_for(source);
    let visited = layout.positioned_glyphs().len() as u64;
    take_presentation_work();
    let demand = derive(
        &layout,
        source,
        UiGlyphRasterDemandSelection::CompleteLayout,
    );
    let work = take_presentation_work();
    assert_eq!(demand.records().len(), 4);
    assert_eq!(
        work.glyphs(UiPresentationWorkStage::DemandDerivation),
        visited
    );
    assert_eq!(work.map_inserts(), demand.records().len() as u64);
    assert!(work.digested_bytes() > 0);
    other_stages_are_idle(work);
}

#[test]
fn damage_that_excludes_every_glyph_still_counts_the_visits_but_no_keys() {
    let source = "Wave";
    let layout = layout_for(source);
    let visited = layout.positioned_glyphs().len() as u64;
    take_presentation_work();
    let demand = derive(
        &layout,
        source,
        UiGlyphRasterDemandSelection::LogicalDamage(&[]),
    );
    let work = take_presentation_work();
    assert!(demand.records().is_empty());
    assert_eq!(
        work.glyphs(UiPresentationWorkStage::DemandDerivation),
        visited
    );
    assert_eq!(work.map_inserts(), 0);
    other_stages_are_idle(work);
}
