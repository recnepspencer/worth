use crate::native::physical_work_signal::UiNativePhysicalPresentationBasis;
use crate::native::{
    UiNativePresentationInput, UiNativePresentationObservation,
    UiNativePresentationWorkKind as Kind, UiNativeRetainedFrameObservation,
};
use worth_ui_host_contract::{
    UiHostObservationPresentationBasis, UiHostPresentationEpoch, UiMountedFrameIdentity,
};

#[test]
fn attribution_distinguishes_publication_from_current_sample_and_rejects_mixed_reports() {
    // Synthetic reporting records exercise comparison only. The runtime owns
    // acceptance; the native dashboard journey checks that production handoff.
    let publication = UiNativePhysicalPresentationBasis::test();
    let sample = publication.test_successor();
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let current = UiHostObservationPresentationBasis::new(
        sample.host_surface(),
        frame,
        sample.binding(),
        UiHostPresentationEpoch::issued_by_host(42),
    );
    let report = report(sample, frame, Kind::Sample, Some(current.epoch()));
    assert_ne!(sample.attempt(), publication.attempt());
    assert!(report.matches_runtime_attribution_basis(publication.attempt(), current));

    let later = UiHostObservationPresentationBasis::new(
        current.host_surface(),
        frame,
        current.binding(),
        UiHostPresentationEpoch::issued_by_host(43),
    );
    assert!(!report.matches_runtime_attribution_basis(publication.attempt(), later));
    let mut missing_epoch = report.clone();
    missing_epoch.sample_presentation_epoch = None;
    assert!(!missing_epoch.matches_runtime_attribution_basis(publication.attempt(), current));

    for kind in [
        Kind::Initial,
        Kind::Delta,
        Kind::Reconstruction,
        Kind::Unchanged,
    ] {
        let ordinary = self::report(sample, frame, kind, None);
        assert!(!ordinary.matches_runtime_attribution_basis(publication.attempt(), current));
        let ordinary = self::report(publication, frame, kind, None);
        assert!(ordinary.matches_runtime_attribution_basis(publication.attempt(), current));
    }
    for field in 0..5 {
        let mut mixed = report.clone();
        let nested = mixed.presentation.as_mut().unwrap();
        match field {
            0 => nested.presented_frame += 1,
            1 => nested.semantic_surface += 1,
            2 => nested.host_surface += 1,
            3 => nested.binding_generation += 1,
            _ => nested.presentation_attempt = publication.attempt().diagnostic_value(),
        }
        assert!(!mixed.matches_runtime_attribution_basis(publication.attempt(), current));
    }
}

fn report(
    basis: UiNativePhysicalPresentationBasis,
    frame: UiMountedFrameIdentity,
    kind: Kind,
    epoch: Option<UiHostPresentationEpoch>,
) -> UiNativeRetainedFrameObservation {
    let observation = UiNativePresentationObservation::new(UiNativePresentationInput {
        client_physical_size: [100, 100],
        scale_factor_milli: 1_000,
        source_rgba8: [1, 2, 3, 255],
        retained_center_rgba8: [1, 2, 3, 255],
        retained_baseline_rgba8: [0; 4],
        presented_frame: frame.diagnostic_value(),
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
        cost: Default::default(),
        alpha_glyphs: Box::new([]),
        intrinsic_glyphs: Box::new([]),
    });
    UiNativeRetainedFrameObservation::observed(
        frame.diagnostic_value(),
        basis,
        kind,
        epoch,
        [[0; 4], [1, 2, 3, 255]],
        Default::default(),
        2,
        Some(observation),
        Box::new([]),
        Box::new([]),
    )
}
