//! Demand scope must survive derivation, borrowing, and raster admission.

use worth_ui_host_contract::{
    UiGlyphRasterDemandScope, UiMountedRgba8, UiMountedTextForegroundSpan,
    UiMountedTextPaintSpanIdentity, UiTextOriginalRange,
};

use super::demand_alpha_tests::{full_damage, layout_for};
use super::*;

fn derive(
    layout: &crate::UiQualifiedTextLayout,
    source: &str,
    selection: UiGlyphRasterDemandSelection<'_>,
) -> UiGlyphRasterDemandBatch {
    let paint = UiMountedTextForegroundSpan::from_runtime_mounting(
        UiTextOriginalRange::new(0, source.len() as u32).unwrap(),
        UiMountedRgba8::new(255, 255, 255, 255),
        UiMountedTextPaintSpanIdentity::from_runtime_mounting([31; 32]),
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

#[test]
fn equal_records_cannot_reuse_admission_across_selection_scopes() {
    let source = "W";
    let layout = layout_for(source);
    let complete = derive(
        &layout,
        source,
        UiGlyphRasterDemandSelection::CompleteLayout,
    );
    let filtered = derive(
        &layout,
        source,
        UiGlyphRasterDemandSelection::LogicalDamage(&[full_damage()]),
    );
    assert!(!complete.records().is_empty());
    assert_eq!(complete.records(), filtered.records());
    assert_ne!(complete.identity(), filtered.identity());
    assert_eq!(
        complete.as_view().scope(),
        UiGlyphRasterDemandScope::CompleteLayout
    );
    assert_eq!(
        filtered.as_view().scope(),
        UiGlyphRasterDemandScope::DamageFiltered
    );
    assert_eq!(complete.lane(), UiGlyphRasterLane::Ordinary);
    let admission = admit_alpha_outline_transaction(&[(&layout, &filtered)]).unwrap();
    assert!(matches!(
        rasterize_alpha_outline_transaction(&[(&layout, &complete)], &admission),
        Err(UiGlyphRasterizationDenial::ForeignDemandRecord)
    ));
    let admitted = admit_alpha_outline_transaction(&[(&layout, &complete)]).unwrap();
    let raster = rasterize_alpha_outline_transaction(&[(&layout, &complete)], &admitted).unwrap();
    assert!(!raster.batches()[0].batch().records().is_empty());
    assert_eq!(
        raster.completion().batches()[0].demand_identity(),
        complete.identity()
    );
}

#[test]
fn empty_damage_is_not_complete_layout_even_when_layout_has_no_images() {
    for source in ["W", " "] {
        let layout = layout_for(source);
        let complete = derive(
            &layout,
            source,
            UiGlyphRasterDemandSelection::CompleteLayout,
        );
        let filtered = derive(
            &layout,
            source,
            UiGlyphRasterDemandSelection::LogicalDamage(&[]),
        );
        assert!(filtered.records().is_empty());
        assert_eq!(complete.records().is_empty(), source == " ");
        assert_ne!(complete.identity(), filtered.identity());
        assert_eq!(
            complete.as_view().scope(),
            UiGlyphRasterDemandScope::CompleteLayout
        );
        assert_eq!(
            filtered.as_view().scope(),
            UiGlyphRasterDemandScope::DamageFiltered
        );
        assert_eq!(complete.cost().ordinary().rasterized_glyphs(), 0);
    }
}
