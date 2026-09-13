use super::completion_frame;
use crate::native::{
    UiNativeClientPresentationAttribution, UiNativePresentationInput,
    UiNativePresentationObservation, UiNativePresentationWorkKind as Kind,
    UiNativeRetainedFrameObservation,
};
use worth_ui_host_contract::{UiHostPresentationCostInput, UiHostPresentationCostReport};

#[test]
fn completion_requires_physical_evidence_and_matches_only_current_component_attribution() {
    let attributed = component_frame();
    let attribution = UiNativeClientPresentationAttribution::reported(
        [
            7,
            attributed.semantic_surface(),
            attributed.binding_generation(),
            4,
            5,
            attributed.presentation_attempt(),
        ],
        [8, 9],
    );
    assert_eq!(
        completion_frame(Some(&attributed), Some(attribution)),
        Ok(attributed.clone())
    );
    assert!(completion_frame(None, None).is_err());
    assert!(completion_frame(Some(&attributed), None).is_err());
    let stale = UiNativeClientPresentationAttribution::reported([1, 2, 3, 4, 5, 6], [8, 9]);
    assert!(completion_frame(Some(&attributed), Some(stale)).is_err());

    // Backdrop pixels have no mounted component attribution. The last component
    // observation must not survive as evidence for these newer frames.
    for kind in [Kind::Initial, Kind::Delta, Kind::Reconstruction] {
        let backdrop = UiNativeRetainedFrameObservation::observed(
            10,
            basis(),
            kind,
            None,
            [[11, 22, 33, 255], [44, 55, 66, 255]],
            cost(),
            2,
            None,
            Box::new([]),
            Box::new([]),
        );
        let completed = completion_frame(Some(&backdrop), None).unwrap();
        assert_eq!(completed, backdrop);
        assert_eq!(completed.port_crossings(), 2);
        assert!(completion_frame(Some(&backdrop), Some(attribution)).is_err());
    }
    let unchanged = UiNativeRetainedFrameObservation::observed(
        11,
        basis(),
        Kind::Unchanged,
        None,
        [[11, 22, 33, 255]; 2],
        Default::default(),
        0,
        None,
        Box::new([]),
        Box::new([]),
    );
    assert_eq!(completion_frame(Some(&unchanged), None), Ok(unchanged));
}

fn component_frame() -> UiNativeRetainedFrameObservation {
    // These diagnostic values test comparison, not authority issuance.
    let basis = basis();
    let observation = UiNativePresentationObservation::new(UiNativePresentationInput {
        client_physical_size: [100, 100],
        scale_factor_milli: 1_000,
        source_rgba8: [1, 2, 3, 255],
        retained_center_rgba8: [1, 2, 3, 255],
        retained_baseline_rgba8: [0; 4],
        presented_frame: 7,
        semantic_surface: basis.surface().diagnostic_value(),
        host_surface: basis.host_surface().diagnostic_value(),
        binding_generation: basis.binding().diagnostic_value(),
        mounted_instance: 4,
        node_receipt: 5,
        presentation_attempt: basis.attempt().diagnostic_value(),
        logical_bounds_milli: [0, 0, 100_000, 100_000],
        order_ordinal: 0,
        port_crossings: 2,
        production_cost: Default::default(),
        cost: cost(),
        alpha_glyphs: Box::new([]),
        intrinsic_glyphs: Box::new([]),
    });
    UiNativeRetainedFrameObservation::observed(
        7,
        basis,
        Kind::Initial,
        None,
        [[0; 4], [1, 2, 3, 255]],
        cost(),
        2,
        Some(observation),
        Box::new([]),
        Box::new([]),
    )
}

fn cost() -> UiHostPresentationCostReport {
    UiHostPresentationCostReport::from_adapter(UiHostPresentationCostInput {
        presents: 1,
        presented_surfaces: 1,
        ..Default::default()
    })
}

fn basis() -> crate::native::physical_work_signal::UiNativePhysicalPresentationBasis {
    crate::native::physical_work_signal::UiNativePhysicalPresentationBasis::test()
}

#[test]
fn current_frame_survives_history_eviction_and_closes_with_its_owner() {
    let mut state = crate::native::UiNativeHostState::new();
    let initial = component_frame();
    state.record_retained_frame_observation(initial.clone());
    for frame in 8..80 {
        state.record_retained_frame_observation(UiNativeRetainedFrameObservation::observed(
            frame,
            basis(),
            Kind::Unchanged,
            None,
            [[11, 22, 33, 255]; 2],
            Default::default(),
            0,
            None,
            Box::new([]),
            Box::new([]),
        ));
    }
    assert!(state.observation_history_overflowed);
    assert!(!state
        .retained_frame_observations
        .iter()
        .any(|frame| frame.cost().presents() != 0));
    let last = completion_frame(state.last_retained_frame.as_ref(), None).unwrap();
    assert_eq!(last.frame(), 79);
    assert_eq!(
        state.current_resource_census().retained_frame_observations,
        state.retained_frame_observations.len() + 1
    );
    assert!(state.close().is_zero());
    assert!(completion_frame(state.last_retained_frame.as_ref(), None).is_err());
}

#[cfg(feature = "certification-support")]
#[test]
fn lost_current_affinity_cannot_reuse_history_as_completion_evidence() {
    let plan = crate::UiNativeQualificationPlan::derived_state_loss_after_completed_presentation(
        1,
        crate::UiNativeDerivedStateLossClass::PresentationAffinity,
    )
    .unwrap();
    let mut state = crate::native::UiNativeHostState::new_for_certification(plan);
    let frame = UiNativeRetainedFrameObservation::observed(
        10,
        basis(),
        Kind::Initial,
        None,
        [[11, 22, 33, 255]; 2],
        cost(),
        2,
        None,
        Box::new([]),
        Box::new([]),
    );
    let binding = frame.binding_generation();
    state.record_retained_frame_observation(frame.clone());
    state.apply_completed_qualified_derived_state_loss(binding);
    assert_eq!(state.retained_frame_observations.last(), Some(&frame));
    assert!(completion_frame(state.last_retained_frame.as_ref(), None).is_err());
    assert!(state.close().is_zero());
}
