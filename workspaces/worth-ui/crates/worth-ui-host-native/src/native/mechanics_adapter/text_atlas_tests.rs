use std::cell::Cell;

use sha2::{Digest, Sha256};

use super::*;
use worth_ui_host_contract::{
    UiAlphaRasterBatchView, UiAlphaRasterRecordView, UiFontCollectionGeneration,
    UiFontCollectionLineageIdentity, UiGlyphRasterAttribution, UiGlyphRasterBatchSink,
    UiGlyphRasterBearing, UiGlyphRasterCallbackDenial, UiGlyphRasterContentDigest,
    UiGlyphRasterDemandBatchView, UiGlyphRasterDemandBatchViewInput, UiGlyphRasterDemandIdentity,
    UiGlyphRasterDemandRecord, UiGlyphRasterExtent, UiGlyphRasterFractionalOrigin,
    UiGlyphRasterKey, UiGlyphRasterKeyInput, UiGlyphRasterLane, UiGlyphRasterMissRasterizer,
    UiGlyphRasterMissSelectionView, UiGlyphRasterPalette, UiGlyphRasterPinRequest,
    UiGlyphRasterPinTransitionView, UiGlyphRasterRecordViewInput, UiGlyphRasterSize,
    UiGlyphRasterSource, UiGlyphRasterTransactionDenial, UiGlyphRasterTransactionOutcome,
    UiGlyphRasterTransactionPending, UiGlyphVariationCoordinates, UiQualifiedFontFaceIdentity,
    UiQualifiedTextLayoutIdentity, UiTextOriginalRange, UiTextProfileGeneration,
    UiTextScaleGeneration,
};

struct RejectingRasterizer {
    calls: Cell<u32>,
}

struct CountingRasterizer {
    calls: Cell<u32>,
}

impl UiGlyphRasterMissRasterizer for CountingRasterizer {
    fn rasterize(
        &mut self,
        _misses: UiGlyphRasterMissSelectionView<'_>,
        _sink: &mut dyn UiGlyphRasterBatchSink,
    ) -> Result<(), UiGlyphRasterCallbackDenial> {
        self.calls.set(self.calls.get() + 1);
        Ok(())
    }
}

impl UiGlyphRasterMissRasterizer for RejectingRasterizer {
    fn rasterize(
        &mut self,
        misses: UiGlyphRasterMissSelectionView<'_>,
        _sink: &mut dyn UiGlyphRasterBatchSink,
    ) -> Result<(), UiGlyphRasterCallbackDenial> {
        self.calls.set(self.calls.get() + 1);
        assert_eq!(misses.records().len(), 1);
        Err(UiGlyphRasterCallbackDenial::Rejected)
    }
}

#[test]
fn adapter_scoped_transaction_rejects_callback_and_releases_reservation() {
    let mut state = crate::native::UiNativeHostState::new();
    let layout = UiQualifiedTextLayoutIdentity::from_text_mechanics([8; 32]);
    let key = key();
    let record = UiGlyphRasterDemandRecord::from_text_mechanics(
        key,
        UiGlyphRasterAttribution::from_text_mechanics(
            layout,
            UiTextOriginalRange::new(0, 1).unwrap(),
        ),
        UiGlyphRasterExtent::new(4, 4).unwrap(),
    )
    .unwrap();
    let records = [record];
    let demand =
        UiGlyphRasterDemandBatchView::from_text_mechanics(UiGlyphRasterDemandBatchViewInput {
            identity: UiGlyphRasterDemandIdentity::from_text_mechanics([9; 32]),
            layout,
            dpi_milli: 1_000,
            text_scale: UiTextScaleGeneration::new(1).unwrap(),
            lane: UiGlyphRasterLane::Ordinary,
            scope: worth_ui_host_contract::UiGlyphRasterDemandScope::DamageFiltered,
            records: &records,
        })
        .unwrap();
    let mut rasterizer = RejectingRasterizer {
        calls: Cell::new(0),
    };
    let outcome = perform(
        &mut state,
        presentation_basis(),
        &[demand],
        UiGlyphRasterPinTransitionView::from_text_mechanics(&[], &[]),
        &mut rasterizer,
    );
    assert_eq!(rasterizer.calls.get(), 1);
    assert_eq!(
        outcome,
        UiGlyphRasterTransactionOutcome::RejectedAfterRasterization(
            UiGlyphRasterTransactionDenial::CallbackRejected
        )
    );
    assert!(state.text_atlas.census().is_zero());
    let signal = state.physical_signal.observation();
    assert_eq!(signal.counters.admissions, 1);
    assert_eq!(signal.counters.rejected_observations, 1);
    assert_eq!(signal.active_requests, 0);
}

#[test]
fn full_physical_signal_capacity_rejects_before_atlas_planning_or_rasterization() {
    let mut state = crate::native::UiNativeHostState::new();
    for seed in 1_u8..=8 {
        let pending = UiGlyphRasterTransactionPending::from_text_mechanics(
            UiGlyphRasterDemandIdentity::from_text_mechanics([seed; 32]),
            u64::from(seed),
            u64::from(seed) + 10,
            1,
        );
        let pins = UiGlyphRasterPinTransitionView::from_text_mechanics(&[], &[]);
        let planning = state
            .physical_signal
            .admit_atlas_planning(presentation_basis(), &[], pins)
            .unwrap();
        let token = state
            .physical_signal
            .take_ready_atlas_planning(planning)
            .unwrap();
        state
            .physical_signal
            .bind_atlas_upload(token, pending)
            .unwrap();
    }
    state.text_atlas.reset_plan_calls();
    let mut rasterizer = CountingRasterizer {
        calls: Cell::new(0),
    };
    let outcome = perform(
        &mut state,
        presentation_basis(),
        &[hostile_upload_demand()],
        UiGlyphRasterPinTransitionView::from_text_mechanics(&[], &[]),
        &mut rasterizer,
    );
    assert_eq!(
        outcome,
        UiGlyphRasterTransactionOutcome::RejectedBeforeEffects(
            UiGlyphRasterTransactionDenial::ReservationConflict
        )
    );
    assert_eq!(state.text_atlas.plan_calls(), 0);
    assert_eq!(rasterizer.calls.get(), 0);
}

#[test]
fn physical_signal_atlas_identity_binds_the_exact_basis_not_cardinality() {
    let mut owner = crate::native::physical_work_signal::UiNativePhysicalSignalOwner::new();
    let pins = UiGlyphRasterPinTransitionView::from_text_mechanics(&[], &[]);
    let first_demand = demand_for(key_for(5), 9);
    let second_demand = demand_for(key_for(6), 10);
    let first_basis = presentation_basis();
    let second_basis = presentation_basis();
    let first = owner
        .admit_atlas_planning(first_basis, &[first_demand], pins)
        .unwrap();
    let second = owner
        .admit_atlas_planning(first_basis, &[second_demand], pins)
        .unwrap();
    let different_attempt = owner
        .admit_atlas_planning(second_basis, &[first_demand], pins)
        .unwrap();
    assert_ne!(first.basis_digest(), second.basis_digest());
    assert_ne!(first.basis_digest(), different_attempt.basis_digest());
    let planning = owner.take_ready_atlas_planning(first).unwrap();
    let pending = UiGlyphRasterTransactionPending::from_text_mechanics(
        first_demand.identity(),
        1,
        2,
        first_basis.host_session_identity(),
    );
    let upload = owner.bind_atlas_upload(planning, pending).unwrap();
    let crate::native::physical_work_signal::UiNativePhysicalSignalWork::AtlasUpload(identity) =
        upload.work()
    else {
        panic!("atlas planning must retain its exact basis through upload");
    };
    assert_eq!(identity.request().presentation_basis(), first_basis);
}

#[test]
fn adapter_rasterizes_only_native_admitted_misses() {
    let mut state = crate::native::UiNativeHostState::new();
    let layout = UiQualifiedTextLayoutIdentity::from_text_mechanics([8; 32]);
    let record = UiGlyphRasterDemandRecord::from_text_mechanics(
        key(),
        UiGlyphRasterAttribution::from_text_mechanics(
            layout,
            UiTextOriginalRange::new(0, 1).unwrap(),
        ),
        UiGlyphRasterExtent::new(4, 4).unwrap(),
    )
    .unwrap();
    let records = [record];
    let demand =
        UiGlyphRasterDemandBatchView::from_text_mechanics(UiGlyphRasterDemandBatchViewInput {
            identity: UiGlyphRasterDemandIdentity::from_text_mechanics([9; 32]),
            layout,
            dpi_milli: 1_000,
            text_scale: UiTextScaleGeneration::new(1).unwrap(),
            lane: UiGlyphRasterLane::Ordinary,
            scope: worth_ui_host_contract::UiGlyphRasterDemandScope::DamageFiltered,
            records: &records,
        })
        .unwrap();
    let pins = UiGlyphRasterPinTransitionView::from_text_mechanics(&[], &[]);
    let native_pins = super::text_atlas_admission::native_pin_transition(pins);
    let plan = state.text_atlas.plan_many(&[demand], &native_pins).unwrap();
    let upload = crate::native::text_atlas::UiNativeTextAtlasUpload::from_text_mechanics(
        key(),
        4,
        4,
        4,
        vec![0; 16],
        [0; 32],
    );
    assert!(matches!(
        state.text_atlas.settle(
            plan,
            &[upload],
            crate::native::text_atlas::UiNativeTextAtlasExternalOutcome::Submitted,
        ),
        crate::native::text_atlas::UiNativeTextAtlasCommitOutcome::Committed(_)
    ));
    let mut rasterizer = CountingRasterizer {
        calls: Cell::new(0),
    };
    let outcome = perform(
        &mut state,
        presentation_basis(),
        &[demand],
        pins,
        &mut rasterizer,
    );
    assert!(matches!(
        outcome,
        UiGlyphRasterTransactionOutcome::Committed(_)
    ));
    assert_eq!(rasterizer.calls.get(), 0);
}

fn key() -> UiGlyphRasterKey {
    key_for(5)
}

fn key_for(glyph_id: u32) -> UiGlyphRasterKey {
    key_for_source(glyph_id, UiGlyphRasterSource::AlphaOutline)
}

fn key_for_source(glyph_id: u32, source: UiGlyphRasterSource) -> UiGlyphRasterKey {
    UiGlyphRasterKey::from_text_mechanics(UiGlyphRasterKeyInput {
        font_collection: UiFontCollectionGeneration::new(1).unwrap(),
        font_collection_lineage: UiFontCollectionLineageIdentity::from_text_mechanics([3; 32]),
        profile: UiTextProfileGeneration::new(1).unwrap(),
        face: UiQualifiedFontFaceIdentity::from_text_mechanics([4; 32], 0),
        glyph_id,
        variations: UiGlyphVariationCoordinates::empty(),
        palette: UiGlyphRasterPalette::new(0),
        size: UiGlyphRasterSize::from_millipoints(12_000).unwrap(),
        source,
        dpi_milli: 1_000,
        origin: UiGlyphRasterFractionalOrigin::from_sixty_fourths(0, 0),
    })
    .unwrap()
}

fn hostile_upload_demand() -> UiGlyphRasterDemandBatchView<'static> {
    demand_for(key(), 9)
}

fn demand_for(key: UiGlyphRasterKey, identity_seed: u8) -> UiGlyphRasterDemandBatchView<'static> {
    let layout = UiQualifiedTextLayoutIdentity::from_text_mechanics([8; 32]);
    let records = Box::leak(Box::new([demand_record(key, layout)]));
    UiGlyphRasterDemandBatchView::from_text_mechanics(UiGlyphRasterDemandBatchViewInput {
        identity: UiGlyphRasterDemandIdentity::from_text_mechanics([identity_seed; 32]),
        layout,
        dpi_milli: 1_000,
        text_scale: UiTextScaleGeneration::new(1).unwrap(),
        lane: UiGlyphRasterLane::Ordinary,
        scope: worth_ui_host_contract::UiGlyphRasterDemandScope::DamageFiltered,
        records,
    })
    .unwrap()
}

fn demand_record(
    key: UiGlyphRasterKey,
    layout: UiQualifiedTextLayoutIdentity,
) -> UiGlyphRasterDemandRecord {
    UiGlyphRasterDemandRecord::from_text_mechanics(
        key,
        UiGlyphRasterAttribution::from_text_mechanics(
            layout,
            UiTextOriginalRange::new(0, 1).unwrap(),
        ),
        UiGlyphRasterExtent::new(4, 4).unwrap(),
    )
    .unwrap()
}

#[test]
fn pending_physical_staging_denies_before_raster_and_cannot_be_closed_early() {
    let (device, queue, _) = crate::native::text_atlas::qualified_test_device();
    let mut state = crate::native::UiNativeHostState::new();
    let mut gpu = crate::native::text_atlas::UiNativeTextAtlasGpuPages::new();
    gpu.ensure_page(
        &device,
        &mut state.resources,
        crate::native::text_atlas::UiNativeGpuAtlasKind::Color,
    )
    .unwrap();
    let color_key = key_for_source(700, UiGlyphRasterSource::ColorOutline);
    let upload = crate::native::text_atlas::UiNativeTextAtlasUpload::from_text_mechanics(
        color_key,
        512,
        512,
        2_048,
        vec![0; 512 * 512 * 4],
        [0; 32],
    );
    for index in 0..8 {
        gpu.upload(
            crate::native::text_atlas::UiNativeTextAtlasGpuUploadRequest {
                device: &device,
                queue: &queue,
                resources: &mut state.resources,
                kind: crate::native::text_atlas::UiNativeGpuAtlasKind::Color,
                page: 0,
                origin: [(index % 4) * 512, (index / 4) * 512],
                upload: &upload,
            },
        )
        .unwrap();
    }
    assert_eq!(gpu.pending_physical_bytes(), 8 * 1_024 * 1_024);
    state.text_atlas_gpu = Some(gpu);

    let mut rasterizer = CountingRasterizer {
        calls: Cell::new(0),
    };
    let outcome = perform(
        &mut state,
        presentation_basis(),
        &[hostile_upload_demand()],
        UiGlyphRasterPinTransitionView::from_text_mechanics(&[], &[]),
        &mut rasterizer,
    );
    assert_eq!(rasterizer.calls.get(), 0);
    assert_eq!(
        outcome,
        UiGlyphRasterTransactionOutcome::RejectedBeforeEffects(
            UiGlyphRasterTransactionDenial::CapacityExceeded
        )
    );
    let gpu = state.text_atlas_gpu.take().unwrap();
    let mut gpu = match gpu.try_close(&mut state.resources) {
        Err(gpu) => gpu,
        Ok(()) => panic!("pending submissions must retain cleanup authority"),
    };
    assert_eq!(state.resources.current().atlas_staging_buffers, 8);
    gpu.settle_pending(&mut state.resources);
    gpu.try_close(&mut state.resources)
        .unwrap_or_else(|_| panic!("settled uploads must close"));
    assert!(state.resources.current().is_zero());
}

fn presentation_basis() -> crate::native::physical_work_signal::UiNativePhysicalPresentationBasis {
    crate::native::physical_work_signal::UiNativePhysicalPresentationBasis::test()
}

pub(crate) fn seed_pending_atlas_for_event_loop(
    state: &mut crate::native::UiNativeHostState,
) -> UiGlyphRasterTransactionPending {
    let demand = demand_for(key_for(801), 61);
    let mut rasterizer = signal_failure_tests::SubmittingRasterizer;
    let mut upload_port = signal_failure_tests::PendingUploadPort;
    let outcome = perform_with_upload_port(
        state,
        presentation_basis(),
        &[demand],
        UiGlyphRasterPinTransitionView::from_text_mechanics(&[], &[]),
        &mut rasterizer,
        &mut upload_port,
    );
    let UiGlyphRasterTransactionOutcome::Pending(pending) = outcome else {
        panic!("the event-loop fixture must retain admitted atlas work: {outcome:?}");
    };
    pending
}

#[path = "text_atlas_signal_failure_tests.rs"]
mod signal_failure_tests;

#[path = "text_atlas_retry_correlation_tests.rs"]
mod retry_correlation_tests;

#[path = "text_atlas_dx12_tests.rs"]
mod dx12_tests;
