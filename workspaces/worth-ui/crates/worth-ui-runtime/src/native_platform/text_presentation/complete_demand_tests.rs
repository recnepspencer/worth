//! Real text preparation authenticates borrowed complete demand without effects.

use super::{
    prepare_complete_semantic_text, UiMountedEventTimeDpiAuthority, UiNativeTextAtlasTransaction,
    UiNativeTextPresentationPreparation,
};
use crate::mounting::qualified_text_test_support::inert_qualified_layout;
use worth_ui_host_contract::*;

#[test]
fn complete_command_validation_rejects_missing_duplicate_and_reordered_glyphs() {
    let layout = inert_qualified_layout("W\tW\tW");
    let mechanic = candidate(&layout, "W\tW\tW", UiSemanticTextSlot::Value);
    let command = UiMountedPaintCommandIdentity::semantic_text(&mechanic);
    let candidates = [mechanic];
    let requirement = requirement(&candidates[0]);
    let prepared = prepare(&candidates, requirement, &layout);
    let mut cache = worth_ui_text::UiGlyphRasterCache::default();
    let mut transaction = UiNativeTextAtlasTransaction::prepare(
        &prepared,
        |id| (id == layout.identity()).then_some(layout.as_ref()),
        &mut cache,
    )
    .unwrap();
    let (_, report) = transaction.with_mounted_work(
        UiGlyphRasterPinTransitionView::from_text_mechanics(&[], &[]),
        &[],
        |work| {
            let demand = work.demands()[0];
            let runs = work.glyph_runs();
            assert!(runs.len() > 2);
            let (first, repeated) = runs
                .iter()
                .enumerate()
                .find_map(|(index, run)| {
                    runs[index + 1..]
                        .iter()
                        .position(|other| other.raster_key() == run.raster_key())
                        .map(|offset| (index, index + 1 + offset))
                })
                .expect("tab-aligned repeated glyphs share a raster key");
            let cost = work
                .validate_complete_demand(command, demand, runs)
                .unwrap();
            assert_eq!(cost.demand_sources_checked, 1);
            assert_eq!(cost.demand_records_checked, runs.len());
            assert_eq!(cost.glyph_runs_checked, runs.len());
            assert_eq!(
                work.validate_complete_demand(command, demand, &runs[1..]),
                Err(UiMountedTextDemandValidationDenial::GlyphRunMismatch)
            );
            let mut duplicated = runs.to_vec();
            duplicated[repeated] = runs[first];
            assert_eq!(
                work.validate_complete_demand(command, demand, &duplicated),
                Err(UiMountedTextDemandValidationDenial::GlyphRunMismatch)
            );
            let mut reordered = runs.to_vec();
            reordered.swap(first, repeated);
            assert_eq!(
                work.validate_complete_demand(command, demand, &reordered),
                Err(UiMountedTextDemandValidationDenial::GlyphRunMismatch)
            );
            let filtered_flag = UiGlyphRasterDemandBatchView::from_text_mechanics(
                UiGlyphRasterDemandBatchViewInput {
                    identity: demand.identity(),
                    layout: demand.layout_identity(),
                    dpi_milli: demand.dpi_milli(),
                    text_scale: demand.text_scale_generation(),
                    lane: demand.lane(),
                    scope: UiGlyphRasterDemandScope::DamageFiltered,
                    records: demand.records(),
                },
            )
            .unwrap();
            assert_eq!(
                work.validate_complete_demand(command, filtered_flag, runs),
                Err(UiMountedTextDemandValidationDenial::DemandMismatch)
            );
        },
    );
    assert_eq!(report.rasterized_glyphs(), 0);
    assert_eq!(report.produced_bytes(), 0);
    assert_eq!(transaction.cache_len(), 0);
}

#[test]
fn complete_empty_commands_authenticate_individually_without_raster_or_pins() {
    // Independent command identities test text correspondence only; the native
    // finalizer must separately admit a common frame and surface binding.
    let layout = inert_qualified_layout(" ");
    let candidates = [
        candidate(&layout, " ", UiSemanticTextSlot::Value),
        candidate(&layout, " ", UiSemanticTextSlot::Posture),
    ];
    let requirement = requirement(&candidates[0]);
    let prepared = prepare(&candidates, requirement, &layout);
    let mut cache = worth_ui_text::UiGlyphRasterCache::default();
    let mut transaction = UiNativeTextAtlasTransaction::prepare(
        &prepared,
        |id| (id == layout.identity()).then_some(layout.as_ref()),
        &mut cache,
    )
    .unwrap();
    let (_, report) = transaction.with_mounted_work(
        UiGlyphRasterPinTransitionView::from_text_mechanics(&[], &[]),
        &[],
        |work| {
            assert_eq!(work.demands().len(), 2);
            assert_eq!(work.demands()[0], work.demands()[1]);
            assert!(work.glyph_runs().is_empty());
            for (index, candidate) in candidates.iter().enumerate() {
                let command = UiMountedPaintCommandIdentity::semantic_text(candidate);
                let cost = work
                    .validate_complete_demand(command, work.demands()[index], &[])
                    .unwrap();
                assert_eq!(cost.demand_sources_checked, index + 1);
                assert_eq!(cost.glyph_runs_checked, 0);
            }
            let absent = candidate(&layout, " ", UiSemanticTextSlot::Value);
            assert_eq!(
                work.validate_complete_demand(
                    UiMountedPaintCommandIdentity::semantic_text(&absent),
                    work.demands()[0],
                    &[]
                ),
                Err(UiMountedTextDemandValidationDenial::MissingCommand)
            );
        },
    );
    assert_eq!(report.rasterized_glyphs(), 0);
    assert_eq!(transaction.cache_len(), 0);
}

fn prepare<'a>(
    candidates: &'a [UiMountedSemanticTextMechanic],
    requirement: UiMountedSurfaceBindingRequirement,
    layout: &'a worth_ui_text::UiQualifiedTextLayout,
) -> super::UiNativeTextPresentationPrepared {
    let UiNativeTextPresentationPreparation::Prepared(prepared) = prepare_complete_semantic_text(
        candidates,
        UiMountedEventTimeDpiAuthority::from_requirement(requirement).unwrap(),
        UiGlyphRasterLane::Ordinary,
        |id| (id == layout.identity()).then_some(layout),
    )
    .unwrap() else {
        panic!("qualified candidate must prepare");
    };
    prepared
}

fn requirement(candidate: &UiMountedSemanticTextMechanic) -> UiMountedSurfaceBindingRequirement {
    UiMountedSurfaceBindingRequirement::new(
        candidate.surface(),
        UiHostSurfaceIdentity::mint_unbound().unwrap(),
        candidate.binding(),
        WorthUiHostCapabilityObservationGeneration::new(7),
        11,
        UiHostSurfacePresentationMode::NativeDisplay,
    )
}

fn candidate(
    layout: &worth_ui_text::UiQualifiedTextLayout,
    source: &str,
    slot: UiSemanticTextSlot,
) -> UiMountedSemanticTextMechanic {
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let bounds = UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: 0.0,
        y: 0.0,
        width: 160.0,
        height: 48.0,
        coordinate_space: UiMountedCoordinateSpace::HostSurface,
    })
    .unwrap();
    UiMountedSemanticTextMechanic::complete_from_runtime_mounting(
        UiMountedSemanticTextCompletionInput {
            content_generation: UiMountedContentGeneration::mint_unbound().unwrap(),
            frame,
            surface: UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
            binding: UiSurfaceBindingGeneration::mint_unbound().unwrap(),
            mounted_instance: instance,
            node_receipt: UiMountedNodeReceiptIssuer::mint_for(frame)
                .unwrap()
                .receipt_for(instance),
            allocation_basis: UiMountedAllocationBasis::new(
                1,
                1,
                1,
                UiMountedTransformProjection::Identity,
            ),
            bounds,
            clip_bounds: bounds,
            origin_x: 0.0,
            origin_y: 0.0,
            text: std::sync::Arc::from(source),
            layout: layout.view(),
            slot,
            collection_row: None,
            foregrounds: std::sync::Arc::from([
                UiMountedTextForegroundSpan::from_runtime_mounting(
                    UiTextOriginalRange::new(0, source.len() as u32).unwrap(),
                    UiMountedRgba8::new(255, 255, 255, 255),
                    UiMountedTextPaintSpanIdentity::from_runtime_mounting([17; 32]),
                ),
            ]),
            profile: UiSemanticTextProfile::BodyDefault,
            layer_semantic_order: 1,
            capability_generation: WorthUiHostCapabilityObservationGeneration::new(7),
            capability_profile_digest: 11,
        },
    )
    .unwrap()
}
