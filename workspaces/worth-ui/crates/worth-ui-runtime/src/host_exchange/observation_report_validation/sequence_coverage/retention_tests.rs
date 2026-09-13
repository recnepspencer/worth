use super::*;
use worth_ui_host_contract::*;

#[test]
fn retained_pointer_burst_crosses_structural_and_exact_sequence_admission() {
    let UiHostProtocolNegotiation::Compatible(protocol) =
        UiHostProtocolContract::current().negotiate()
    else {
        panic!("current protocol");
    };
    let presentation = UiHostObservationPresentationBasis::new(
        UiHostSurfaceIdentity::mint_unbound().unwrap(),
        UiMountedFrameIdentity::mint_unbound().unwrap(),
        UiSurfaceBindingGeneration::mint_unbound().unwrap(),
        UiHostPresentationEpoch::issued_by_host(1),
    );
    let retention = UiHostObservationRetention::default();
    retention.register_session(1).unwrap();
    for sequence in 1..=65 {
        let report = UiHostObservationReport::new(
            UiHostObservationSequence::new(sequence),
            UiHostObservationTimeBasis::HostMonotonicMillis(sequence),
            UiHostObservationPayload::PointerMotion {
                pointer: UiHostPointerIdentity::new(1),
                capture_epoch: UiHostPointerCaptureEpoch::new(1),
                pressed_buttons: UiHostPressedPointerButtons::NONE,
                position: UiHostSurfacePosition::viewport_logical(sequence as i64 * 1_000, 0),
            },
        )
        .with_pointer_device_kind(UiHostPointerDeviceKind::Mouse)
        .unwrap();
        let sequence = report.sequence();
        retention
            .retain_latest_pointer_motion(
                UiHostObservationBatch::new(UiHostObservationBatchInput {
                    protocol,
                    host_session: 1,
                    presentation,
                    sequences: UiHostObservationSequenceRange::new(sequence, sequence),
                    loss: UiHostObservationLoss::Complete,
                    reports: vec![report],
                })
                .unwrap(),
            )
            .unwrap();
    }
    let batches = retention.drain(1).into_batches();
    assert_eq!(batches.len(), 1);
    let admitted =
        UiStructurallyAdmittedObservationBatch::admit(batches[0].clone(), protocol).unwrap();
    let covered = UiSequenceCoveredObservationBatch::prove(admitted).unwrap();
    assert_eq!(covered.host_survivor().unwrap().value(), 65);
    assert!(
        matches!(covered.disposition(), UiHostObservationBatchDisposition::Coalesced { replaced, .. } if replaced.first().value() == 1 && replaced.last().value() == 64)
    );
    assert!(
        matches!(covered.reports()[0].payload(), UiHostObservationPayload::PointerMotion { position, .. } if position.x_subpixels() == 65_000)
    );
}
