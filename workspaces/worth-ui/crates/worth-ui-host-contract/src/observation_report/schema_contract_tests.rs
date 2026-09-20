use crate::{
    UiHostObservationBatch, UiHostObservationBatchInput, UiHostObservationLoss,
    UiHostObservationMountedBasis, UiHostObservationPayload, UiHostObservationPresentationBasis,
    UiHostObservationReport, UiHostObservationSchemaVersion, UiHostObservationSequence,
    UiHostObservationSequenceRange, UiHostObservationTimeBasis, UiHostPresentationEpoch,
    UiHostProtocolContract, UiHostProtocolDenial, UiHostProtocolNegotiation,
    UiHostProtocolSchemaFamily, UiHostScrollDeltaPhase, UiHostScrollDeltaPrecision,
    UiHostScrollDeltaSource, UiHostScrollDeltaTargetAffinity, UiHostScrollLineCountBasis,
    UiHostSurfaceIdentity, UiMountedFrameIdentity, UiMountedInstanceIdentity,
    UiMountedNodeReceiptIssuer, UiSurfaceBindingGeneration, UI_HOST_SCROLL_DEFAULT_LINES_PER_NOTCH,
};

#[test]
fn observation_schema_v7_is_required_before_batch_authority_exists() {
    let current = UiHostProtocolContract::current();
    assert_eq!(current.observation().revision(), 7);
    assert!(matches!(
        current.negotiate(),
        UiHostProtocolNegotiation::Compatible(_)
    ));
    assert_eq!(
        with_observation_revision(6).negotiate(),
        UiHostProtocolNegotiation::Incompatible(UiHostProtocolDenial::SchemaTooOld(
            UiHostProtocolSchemaFamily::Observation,
        ))
    );
    assert_eq!(
        with_observation_revision(8).negotiate(),
        UiHostProtocolNegotiation::Incompatible(UiHostProtocolDenial::SchemaTooNew(
            UiHostProtocolSchemaFamily::Observation,
        ))
    );
}

#[test]
fn exact_host_presentation_epoch_participates_in_batch_identity() {
    let frame = UiMountedFrameIdentity::mint_unbound().expect("frame identity capacity");
    let binding = UiSurfaceBindingGeneration::mint_unbound().expect("binding identity capacity");
    let surface = UiHostSurfaceIdentity::mint_unbound().expect("host surface capacity");
    let first = batch_for_epoch(surface, frame, binding, 11);
    let second = batch_for_epoch(surface, frame, binding, 12);

    assert_ne!(first.integrity(), second.integrity());
    assert_eq!(
        first.canonical_core().presentation().epoch(),
        UiHostPresentationEpoch::issued_by_host(11)
    );
}

#[test]
fn window_focus_payload_cannot_name_a_surface_other_than_the_batch_presentation() {
    let surface = UiHostSurfaceIdentity::mint_unbound().unwrap();
    let foreign = UiHostSurfaceIdentity::mint_unbound().unwrap();
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let protocol = match UiHostProtocolContract::current().negotiate() {
        UiHostProtocolNegotiation::Compatible(protocol) => protocol,
        UiHostProtocolNegotiation::Incompatible(_) => unreachable!(),
    };
    let sequence = UiHostObservationSequence::new(1);
    let result = UiHostObservationBatch::new(UiHostObservationBatchInput {
        protocol,
        host_session: 1,
        presentation: UiHostObservationPresentationBasis::new(
            surface,
            frame,
            binding,
            UiHostPresentationEpoch::issued_by_host(1),
        ),
        sequences: UiHostObservationSequenceRange::new(sequence, sequence),
        loss: UiHostObservationLoss::Complete,
        reports: vec![UiHostObservationReport::new(
            sequence,
            UiHostObservationTimeBasis::HostMonotonicMillis(1),
            UiHostObservationPayload::WindowFocus {
                surface: foreign,
                focused: true,
            },
        )],
    });
    assert_eq!(
        result,
        Err(crate::UiHostObservationBatchConstructionDenial::PayloadSurfaceMismatch)
    );
}

#[test]
fn scroll_target_cannot_name_a_presentation_other_than_the_batch_presentation() {
    let surface = UiHostSurfaceIdentity::mint_unbound().unwrap();
    let foreign_surface = UiHostSurfaceIdentity::mint_unbound().unwrap();
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let presentation = UiHostObservationPresentationBasis::new(
        surface,
        frame,
        binding,
        UiHostPresentationEpoch::issued_by_host(1),
    );
    let foreign_presentation = UiHostObservationPresentationBasis::new(
        foreign_surface,
        frame,
        binding,
        UiHostPresentationEpoch::issued_by_host(1),
    );
    let protocol = match UiHostProtocolContract::current().negotiate() {
        UiHostProtocolNegotiation::Compatible(protocol) => protocol,
        UiHostProtocolNegotiation::Incompatible(_) => unreachable!(),
    };
    let sequence = UiHostObservationSequence::new(1);
    let result = UiHostObservationBatch::new(UiHostObservationBatchInput {
        protocol,
        host_session: 1,
        presentation,
        sequences: UiHostObservationSequenceRange::new(sequence, sequence),
        loss: UiHostObservationLoss::Complete,
        reports: vec![UiHostObservationReport::new(
            sequence,
            UiHostObservationTimeBasis::HostMonotonicMillis(1),
            UiHostObservationPayload::ScrollDelta {
                source: UiHostScrollDeltaSource::PointerWheel,
                phase: UiHostScrollDeltaPhase::Updated,
                precision: UiHostScrollDeltaPrecision::Line {
                    platform_lines_per_notch: UI_HOST_SCROLL_DEFAULT_LINES_PER_NOTCH,
                    basis: UiHostScrollLineCountBasis::DefaultedAfterMissing,
                },
                target: UiHostScrollDeltaTargetAffinity::presented_surface_fallback(
                    foreign_presentation,
                ),
                x_subpixels: 0,
                y_subpixels: -40_000,
            },
        )],
    });
    assert_eq!(
        result,
        Err(crate::UiHostObservationBatchConstructionDenial::PayloadPresentationMismatch,)
    );
}

#[test]
fn exact_scroll_target_receipt_must_match_its_presentation_frame_and_instance() {
    let surface = UiHostSurfaceIdentity::mint_unbound().unwrap();
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let presentation = UiHostObservationPresentationBasis::new(
        surface,
        frame,
        binding,
        UiHostPresentationEpoch::issued_by_host(1),
    );
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let issuer = UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
    let foreign_frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let foreign_frame_issuer = UiMountedNodeReceiptIssuer::mint_for(foreign_frame).unwrap();
    let foreign_instance = UiMountedInstanceIdentity::mint_unbound().unwrap();

    assert_eq!(
        scroll_batch(
            presentation,
            UiHostObservationMountedBasis::new(
                instance,
                foreign_frame_issuer.receipt_for(instance),
            ),
        ),
        Err(crate::UiHostObservationBatchConstructionDenial::PayloadMountedTargetFrameMismatch,)
    );
    assert_eq!(
        scroll_batch(
            presentation,
            UiHostObservationMountedBasis::new(foreign_instance, issuer.receipt_for(instance),),
        ),
        Err(crate::UiHostObservationBatchConstructionDenial::PayloadMountedTargetInstanceMismatch,)
    );
}

fn scroll_batch(
    presentation: UiHostObservationPresentationBasis,
    mounted: UiHostObservationMountedBasis,
) -> Result<UiHostObservationBatch, crate::UiHostObservationBatchConstructionDenial> {
    let protocol = match UiHostProtocolContract::current().negotiate() {
        UiHostProtocolNegotiation::Compatible(protocol) => protocol,
        UiHostProtocolNegotiation::Incompatible(_) => unreachable!(),
    };
    let sequence = UiHostObservationSequence::new(1);
    UiHostObservationBatch::new(UiHostObservationBatchInput {
        protocol,
        host_session: 1,
        presentation,
        sequences: UiHostObservationSequenceRange::new(sequence, sequence),
        loss: UiHostObservationLoss::Complete,
        reports: vec![UiHostObservationReport::new(
            sequence,
            UiHostObservationTimeBasis::HostMonotonicMillis(1),
            UiHostObservationPayload::ScrollDelta {
                source: UiHostScrollDeltaSource::PointerWheel,
                phase: UiHostScrollDeltaPhase::Updated,
                precision: UiHostScrollDeltaPrecision::Pixel,
                target: UiHostScrollDeltaTargetAffinity::exact_mounted_target(
                    presentation,
                    mounted,
                ),
                x_subpixels: 0,
                y_subpixels: -1,
            },
        )],
    })
}

fn batch_for_epoch(
    surface: UiHostSurfaceIdentity,
    frame: UiMountedFrameIdentity,
    binding: UiSurfaceBindingGeneration,
    epoch: u64,
) -> UiHostObservationBatch {
    let protocol = match UiHostProtocolContract::current().negotiate() {
        UiHostProtocolNegotiation::Compatible(agreement) => agreement,
        UiHostProtocolNegotiation::Incompatible(_) => unreachable!(),
    };
    let sequence = UiHostObservationSequence::new(1);
    UiHostObservationBatch::new(UiHostObservationBatchInput {
        protocol,
        host_session: 1,
        presentation: UiHostObservationPresentationBasis::new(
            surface,
            frame,
            binding,
            UiHostPresentationEpoch::issued_by_host(epoch),
        ),
        sequences: UiHostObservationSequenceRange::new(sequence, sequence),
        loss: UiHostObservationLoss::Complete,
        reports: vec![UiHostObservationReport::new(
            sequence,
            UiHostObservationTimeBasis::HostMonotonicMillis(1),
            UiHostObservationPayload::WindowFocus {
                surface,
                focused: true,
            },
        )],
    })
    .expect("single focused observation batch is valid")
}

fn with_observation_revision(revision: u16) -> UiHostProtocolContract {
    let current = UiHostProtocolContract::current();
    UiHostProtocolContract::new(
        current.identity(),
        current.protocol(),
        current.mounted_frame(),
        current.mounted_presentation(),
        UiHostObservationSchemaVersion::new(revision),
        current.measurement(),
        current.solicited_effect(),
    )
}
