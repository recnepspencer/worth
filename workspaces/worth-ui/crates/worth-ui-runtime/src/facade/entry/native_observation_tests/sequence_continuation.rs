use super::*;
use crate::facade::interaction::UiHostInteractionIngressOutcome;
use crate::facade::observation_report::UiHostObservationReportDenial as Denial;

fn focus_batch(
    protocol: worth_ui_host_contract::UiHostProtocolAgreement,
    host_session: u64,
    presentation: UiHostObservationPresentationBasis,
    sequence: u64,
) -> UiHostObservationBatch {
    let sequence = UiHostObservationSequence::new(sequence);
    UiHostObservationBatch::new(UiHostObservationBatchInput {
        protocol,
        host_session,
        presentation,
        sequences: UiHostObservationSequenceRange::new(sequence, sequence),
        loss: UiHostObservationLoss::Complete,
        reports: vec![UiHostObservationReport::new(
            sequence,
            UiHostObservationTimeBasis::HostMonotonicMillis(sequence.value()),
            UiHostObservationPayload::WindowFocus {
                surface: presentation.host_surface(),
                focused: true,
            },
        )],
    })
    .unwrap()
}

fn denial(outcome: UiHostInteractionIngressOutcome) -> Denial {
    let UiHostInteractionIngressOutcome::Denied(denial) = outcome else {
        panic!("batch must be denied: {outcome:?}");
    };
    denial.denial()
}

#[test]
fn the_host_continues_after_its_denied_batch_but_a_foreign_one_moves_nothing() {
    let host = ScriptedPresentationHost::native_display();
    host.push_native_display_presented();
    let mut shell = source_backed_hover_consumer_app_with_host(host)
        .launch_native_surface()
        .unwrap();
    super::super::native_application_identity_trace_test_support::install_bound_surface_geometry(
        &mut shell,
    );
    let frame = published(shell.present_frame(100, 1), "sequence");
    let presentation = UiHostObservationPresentationBasis::new(
        shell.session.mounted.view().surface_bindings()[0].host_surface_identity(),
        frame.frame(),
        frame.bindings()[0],
        UiHostPresentationEpoch::issued_by_host(1),
    );
    let stale = UiHostObservationPresentationBasis::new(
        presentation.host_surface(),
        presentation.frame(),
        presentation.binding(),
        UiHostPresentationEpoch::issued_by_host(9),
    );
    let host_session = shell.session.host_session.identity().as_u64();
    let protocol = shell.session.host_session.protocol();

    let foreign = focus_batch(protocol, host_session + 1, presentation, 1);
    assert_eq!(
        denial(shell.session.admit_host_interaction_batch(foreign)),
        Denial::ForeignHostSession
    );
    let past_foreign = focus_batch(protocol, host_session, presentation, 2);
    assert_eq!(
        denial(shell.session.admit_host_interaction_batch(past_foreign)),
        Denial::SequenceGap,
        "a foreign batch cannot open the host stream's next position"
    );

    let lost = focus_batch(protocol, host_session, stale, 1);
    assert_eq!(
        denial(shell.session.admit_host_interaction_batch(lost)),
        Denial::PresentationEpochMismatch
    );
    let continued = focus_batch(protocol, host_session, presentation, 2);
    assert!(
        matches!(
            shell.session.admit_host_interaction_batch(continued),
            UiHostInteractionIngressOutcome::Applied(_)
        ),
        "the host never resends a denied batch, so its successor is next"
    );
    let reordered = focus_batch(protocol, host_session, presentation, 1);
    assert_eq!(
        denial(shell.session.admit_host_interaction_batch(reordered)),
        Denial::SequenceReordered
    );
}
