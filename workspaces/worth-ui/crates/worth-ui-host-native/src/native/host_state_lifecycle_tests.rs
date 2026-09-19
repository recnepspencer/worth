use super::host_state::UiNativeHostState;
use crate::native::presentation::{
    reserve_presentation_owners, settle_port_result, UiNativePendingExternalObligation,
    UiNativePresentationFailure, UiNativePresentationPortFailure,
};
use std::cell::Cell;
use std::rc::Rc;
use worth_ui_host_contract::{
    UiFontCollectionGeneration, UiFontCollectionLineageIdentity, UiGlyphRasterFractionalOrigin,
    UiGlyphRasterKey, UiGlyphRasterKeyInput, UiGlyphRasterPalette, UiGlyphRasterSize,
    UiGlyphRasterSource, UiGlyphVariationCoordinates, UiQualifiedFontFaceIdentity,
    UiTextProfileGeneration,
};

struct PendingProbe {
    dropped: Rc<Cell<bool>>,
    settles: Rc<Cell<bool>>,
}

impl UiNativePendingExternalObligation for PendingProbe {
    fn poll_observation(
        &mut self,
        basis: crate::native::physical_work_signal::UiNativePhysicalSignalExternalBasis,
        _device: Option<&wgpu::Device>,
    ) -> crate::native::physical_work_signal::UiNativePhysicalSignalExternalObservation {
        basis.observe(if self.settles.get() {
            crate::native::physical_work_signal::UiNativePhysicalSignalStatus::Completed
        } else {
            crate::native::physical_work_signal::UiNativePhysicalSignalStatus::Pending
        })
    }
}

impl Drop for PendingProbe {
    fn drop(&mut self) {
        self.dropped.set(true);
    }
}

#[test]
pub(crate) fn pending_external_work_retains_cleanup_authority_until_it_settles() {
    let mut state = UiNativeHostState::new();
    let dropped = Rc::new(Cell::new(false));
    let settles = Rc::new(Cell::new(false));
    let owners = reserve_presentation_owners(
        &mut state.resources,
        &mut state.physical_signal,
        crate::native::physical_work_signal::UiNativePhysicalPresentationBasis::test(),
    )
    .unwrap_or_else(|_| panic!("empty registry must reserve presentation owners"));
    let pending = settle_port_result(
        &mut state.resources,
        &mut state.physical_signal,
        owners,
        Err(UiNativePresentationPortFailure::ReadbackUnsettled(
            Box::new(PendingProbe {
                dropped: Rc::clone(&dropped),
                settles: Rc::clone(&settles),
            }),
        )),
    );
    let Err(UiNativePresentationFailure::Pending(pending)) = pending else {
        panic!("unsettled external work must enter pending host state");
    };
    state.pending_presentations.push(pending);

    let pending = state.close();
    assert_eq!(pending.readback_buffers, 1);
    assert_eq!(pending.pending_submissions, 1);
    assert_eq!(pending.pending_presentations, 1);
    assert!(!dropped.get());
    let physical = state.physical_signal.observation();
    assert_eq!(physical.counters.admissions, 1);
    assert_eq!(physical.counters.pending_observations, 1);
    assert_eq!(physical.active_requests, 1);

    settles.set(true);
    let due = state
        .physical_signal
        .next_due_tick()
        .expect("pending presentation must retain one Signal-owned poll wake");
    state
        .physical_signal
        .advance_clock_to(due)
        .expect("the exact pending poll wake must become ready");
    assert!(state.progress_one_physical_signal_ready());
    assert!(state.close().is_zero());
    assert!(dropped.get());
    let physical = state.physical_signal.observation();
    assert_eq!(physical.counters.completed_observations, 1);
    assert_eq!(physical.active_requests, 0);
    assert!(!physical.accepting_admissions);
    assert!(!physical.runtime_owned);
}

#[test]
fn production_observation_history_is_bounded_and_reports_overflow() {
    let mut state = UiNativeHostState::new();
    for _ in 0..=super::host_state::NATIVE_OBSERVATION_HISTORY_CAPACITY {
        state.record_text_pin_frame_observation();
    }

    assert_eq!(
        state.text_pin_frame_counts.len(),
        super::host_state::NATIVE_OBSERVATION_HISTORY_CAPACITY
    );
    assert_eq!(
        state.text_pin_frame_observations.len(),
        super::host_state::NATIVE_OBSERVATION_HISTORY_CAPACITY
    );
    assert!(state.observation_history_overflowed);
    assert_eq!(
        state.compiler_total_peak().text_pin_frame_observations,
        super::host_state::NATIVE_OBSERVATION_HISTORY_CAPACITY
    );
    assert!(state.close().is_zero());
}

#[test]
fn host_state_exposes_one_terminal_text_atlas_census() {
    let mut state = UiNativeHostState::new();
    assert!(state.text_atlas_census().is_zero());
    assert!(state.close().is_zero());
}

#[test]
fn host_close_retains_pending_atlas_upload_until_external_settlement() {
    let (device, queue, _) = crate::native::text_atlas::qualified_test_device();
    let mut state = UiNativeHostState::new();
    let mut gpu = crate::native::text_atlas::UiNativeTextAtlasGpuPages::new();
    gpu.ensure_page(
        &device,
        &mut state.resources,
        crate::native::text_atlas::UiNativeGpuAtlasKind::Alpha,
    )
    .unwrap();
    let key = UiGlyphRasterKey::from_text_mechanics(UiGlyphRasterKeyInput {
        font_collection: UiFontCollectionGeneration::new(1).unwrap(),
        font_collection_lineage: UiFontCollectionLineageIdentity::from_text_mechanics([3; 32]),
        profile: UiTextProfileGeneration::new(1).unwrap(),
        face: UiQualifiedFontFaceIdentity::from_text_mechanics([4; 32], 0),
        glyph_id: 1,
        variations: UiGlyphVariationCoordinates::empty(),
        palette: UiGlyphRasterPalette::new(0),
        size: UiGlyphRasterSize::from_millipoints(12_000).unwrap(),
        source: UiGlyphRasterSource::AlphaOutline,
        dpi_milli: 1_000,
        origin: UiGlyphRasterFractionalOrigin::from_sixty_fourths(0, 0),
    })
    .unwrap();
    let upload = crate::native::text_atlas::UiNativeTextAtlasUpload::from_text_mechanics(
        key,
        2,
        2,
        2,
        vec![0; 4],
        [0; 32],
    );
    gpu.upload(
        crate::native::text_atlas::UiNativeTextAtlasGpuUploadRequest {
            device: &device,
            queue: &queue,
            resources: &mut state.resources,
            kind: crate::native::text_atlas::UiNativeGpuAtlasKind::Alpha,
            page: 0,
            origin: [0, 0],
            upload: &upload,
        },
    )
    .unwrap();
    state.text_atlas_gpu = Some(gpu);

    let incomplete = state.close();
    assert_eq!(incomplete.atlas_staging_buffers, 1);
    assert_eq!(incomplete.text_atlas_upload_submissions, 1);
    assert!(state.text_atlas_gpu.is_some());

    state
        .text_atlas_gpu
        .as_mut()
        .unwrap()
        .settle_pending(&mut state.resources);
    assert!(state.close().is_zero());
    assert!(state.text_atlas_gpu.is_none());
}

#[test]
fn native_input_reaches_the_mechanics_drain_boundary_with_its_presentation_basis() {
    use winit::dpi::PhysicalPosition;
    use winit::event::{DeviceId, WindowEvent};
    use worth_ui_host_contract::{
        UiHostObservationPayload, UiHostObservationPresentationBasis, UiHostPresentationEpoch,
        UiHostProtocolContract, UiHostProtocolNegotiation, UiMountedFrameIdentity,
        UiSurfaceBindingGeneration, WorthUiHostMechanicsAdapter,
    };

    let host_session = 97;
    let state = std::rc::Rc::new(std::cell::RefCell::new(UiNativeHostState::new()));
    let adapter = super::WorthUiNativeMechanicsAdapter::from_preparation(
        std::rc::Rc::clone(&state),
        crate::UiNativePlatformProfileIdentity::WORTH_UI_WINDOWS_DX12_V2,
    );
    let protocol = match UiHostProtocolContract::current().negotiate() {
        UiHostProtocolNegotiation::Compatible(agreement) => agreement,
        UiHostProtocolNegotiation::Incompatible(_) => unreachable!(),
    };
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let basis = UiHostObservationPresentationBasis::new(
        worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound().unwrap(),
        UiMountedFrameIdentity::mint_unbound().unwrap(),
        binding,
        UiHostPresentationEpoch::issued_by_host(1),
    );
    adapter
        .register_mechanical_host_session(host_session)
        .unwrap();
    {
        let mut state = state.borrow_mut();
        state.lifecycle.install_initial_profile(1.0, [800, 600]);
        assert_eq!(
            state
                .lifecycle
                .record_completed_presentation(protocol, host_session, basis)
                .effect(),
            crate::native::UiNativeLifecycleEffect::PresentationCompleted
        );
        let transition = state.lifecycle.observe_window_event_at(
            &WindowEvent::CursorMoved {
                device_id: DeviceId::dummy(),
                position: PhysicalPosition::new(12.0, 24.0),
            },
            0,
            None,
        );
        assert_eq!(
            transition.effect(),
            crate::native::UiNativeLifecycleEffect::Retained
        );
    }

    let drain = adapter
        .drain_mechanical_host_observations(host_session)
        .unwrap();
    let batches = drain.into_batches();
    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].canonical_core().presentation(), basis);
    assert!(matches!(
        batches[0].reports()[0].payload(),
        UiHostObservationPayload::PointerMotion { .. }
    ));
}
