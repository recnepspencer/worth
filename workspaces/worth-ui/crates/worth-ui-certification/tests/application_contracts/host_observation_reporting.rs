use worth_ui::facade::observation_report::WorthUiHostObservationSessionExt;
use worth_ui::facade::observation_report::{
    UiHostObservationBatchDisposition, UiHostObservationDisposition, UiHostObservationFamily,
    UiHostObservationFrameRelation, UiHostObservationLoss, UiHostObservationPayload,
    UiHostObservationReport, UiHostObservationReportDenial, UiHostObservationReportOutcome,
    UiHostObservationSequence, UiHostObservationSequenceRange, UiHostObservationTimeBasis,
};
use worth_ui_runtime::facade::mounted::{
    UiMountedFrameOutcome, UiMountedFrameRetentionBudget, UiMountedFrameRetentionBudgetInput,
    UiMountedRetentionClassBudget, UiPresentationDeadline,
};
use worth_ui_test_support::WorthUiMountedPublicationCertificationExt;

use super::host_observation_fixture::{batch, report, source, window_focus};
use super::mounted_application_lifecycle::in_flight_presentation_world::prepared;
use super::mounted_application_lifecycle::published_mounted_world::{
    publish, published_observation_world, published_observation_world_with_retention_budget,
    PresentedObservationBasis,
};
use super::mounted_protocol_model::{model_terminal_state, AuthoredMechanicalReport};

#[test]
fn individual_and_host_coalesced_batches_share_terminal_mechanics_and_exact_replaced_range() {
    let mut individual = published_observation_world("observation-individual");
    let authored_trace = (1..=4)
        .map(|sequence| {
            AuthoredMechanicalReport::pointer_motion(
                sequence,
                i64::try_from(sequence * 10).unwrap(),
            )
        })
        .collect::<Vec<_>>();
    let trace = authored_trace
        .iter()
        .map(|authored| {
            report(
                authored.sequence(),
                authored.payload().clone(),
                &individual.current,
            )
        })
        .collect::<Vec<_>>();
    let expected = model_terminal_state(&authored_trace);
    let outcome = individual.session.validate_host_observation_batch(batch(
        source(&individual.session, individual.binding, &individual.current),
        (1, 4),
        UiHostObservationLoss::Complete,
        trace.clone(),
    ));
    let individual_validated = expect_validated(outcome);
    assert_eq!(
        individual_validated.disposition(),
        UiHostObservationBatchDisposition::Complete
    );
    assert_eq!(
        individual_validated
            .reports()
            .last()
            .unwrap()
            .report()
            .payload(),
        &expected.terminal_payload
    );
    assert_eq!(
        individual_validated.reports().last().unwrap().disposition(),
        UiHostObservationDisposition::Coalesced {
            replaced: expected.replaced.unwrap()
        }
    );
    assert_eq!(
        individual.session.retained_host_observation_report_count(),
        expected.retained_reports
    );

    let mut batched = published_observation_world("observation-host-coalesced");
    let terminal = authored_trace.last().unwrap();
    let survivor = report(
        terminal.sequence(),
        terminal.payload().clone(),
        &batched.current,
    );
    let survivor_identity = survivor
        .coalescing_identity()
        .expect("authored pointer motion has an explicit coalescing identity");
    let outcome = batched.session.validate_host_observation_batch(batch(
        source(&batched.session, batched.binding, &batched.current),
        (1, 4),
        UiHostObservationLoss::Coalesced {
            family: UiHostObservationFamily::PointerMotion,
            replaced: range(1, 3),
            survivor: survivor_identity,
        },
        vec![survivor],
    ));
    let batched_validated = expect_validated(outcome);
    assert_eq!(
        batched_validated.disposition(),
        UiHostObservationBatchDisposition::Coalesced {
            family: UiHostObservationFamily::PointerMotion,
            replaced: range(1, 3),
            survivor: survivor_identity,
        }
    );
    assert_eq!(
        batched_validated.reports()[0].report().payload(),
        &expected.terminal_payload
    );
    assert_eq!(
        batched_validated.reports()[0].disposition(),
        UiHostObservationDisposition::Coalesced {
            replaced: range(1, 3)
        }
    );
    assert_eq!(batched.session.retained_host_observation_report_count(), 1);
    let work = batched.session.host_observation_work_report();
    assert_eq!(work.raw_entries_handled(), 1);
    assert_eq!(work.validated_entries(), 1);
    assert_eq!(work.coalesced_entries(), 4);
}

#[test]
fn current_retained_expired_rejected_never_presented_and_indeterminate_bases_are_distinct() {
    let mut world = published_observation_world("observation-basis-current-retained");
    let retained_batch = batch(
        source(&world.session, world.binding, &world.predecessor),
        (1, 1),
        UiHostObservationLoss::Complete,
        vec![report(
            1,
            window_focus(&world.predecessor, true),
            &world.predecessor,
        )],
    );
    assert_eq!(
        expect_validated(
            world
                .session
                .validate_host_observation_batch(retained_batch)
        )
        .frame_relation(),
        UiHostObservationFrameRelation::SupersededPresented
    );
    let current_batch = batch(
        source(&world.session, world.binding, &world.current),
        (2, 2),
        UiHostObservationLoss::Complete,
        vec![report(
            2,
            window_focus(&world.current, false),
            &world.current,
        )],
    );
    assert_eq!(
        expect_validated(world.session.validate_host_observation_batch(current_batch))
            .frame_relation(),
        UiHostObservationFrameRelation::CurrentPresented
    );

    let mut expired = published_observation_world_with_retention_budget(
        "observation-basis-expired",
        one_predecessor_budget(),
    );
    let instance = expired.current.instance;
    expired.current = publish(&mut expired.session, &expired.host, instance);
    let expired_batch = batch(
        source(&expired.session, expired.binding, &expired.predecessor),
        (1, 1),
        UiHostObservationLoss::Complete,
        vec![report(
            1,
            UiHostObservationPayload::Tick { tick: 1 },
            &expired.predecessor,
        )],
    );
    assert_denial(
        expired
            .session
            .validate_host_observation_batch(expired_batch),
        UiHostObservationReportDenial::ExpiredFrame,
    );

    assert_terminal_basis(
        "observation-basis-rejected",
        TerminalBasis::Rejected,
        UiHostObservationReportDenial::RejectedFrame,
    );
    assert_terminal_basis(
        "observation-basis-never-presented",
        TerminalBasis::NeverPresented,
        UiHostObservationReportDenial::NeverPresentedFrame,
    );
    assert_indeterminate_quarantine();
}

fn one_predecessor_budget() -> UiMountedFrameRetentionBudget {
    let defaults = UiMountedFrameRetentionBudget::default();
    UiMountedFrameRetentionBudget::new(UiMountedFrameRetentionBudgetInput {
        current: defaults.current(),
        in_flight: defaults.in_flight(),
        observation_basis: defaults.observation_basis(),
        predecessor_inspection: UiMountedRetentionClassBudget::new(1, 256 * 1024 * 1024),
        diagnostic: defaults.diagnostic(),
        visual_snapshot: defaults.visual_snapshot(),
        visual_overlay: defaults.visual_overlay(),
        expired_identity_limit: defaults.expired_identity_limit(),
    })
}

#[derive(Clone, Copy)]
enum TerminalBasis {
    Rejected,
    NeverPresented,
}

fn assert_terminal_basis(
    label: &str,
    posture: TerminalBasis,
    expected: UiHostObservationReportDenial,
) {
    let mut world = published_observation_world(label);
    let frame = prepared(&mut world.session);
    let frame_identity = frame.canonical_core().frame();
    let outcome = match posture {
        TerminalBasis::Rejected => {
            world.host.push_rejected();
            world.session.present_prepared_mounted_frame(
                frame,
                UiPresentationDeadline::at_tick(100),
                0,
            )
        }
        TerminalBasis::NeverPresented => world.session.present_prepared_mounted_frame(
            frame,
            UiPresentationDeadline::at_tick(0),
            0,
        ),
    };
    assert!(matches!(
        (posture, outcome),
        (
            TerminalBasis::Rejected,
            UiMountedFrameOutcome::RejectedBeforeEffects(_)
        ) | (
            TerminalBasis::NeverPresented,
            UiMountedFrameOutcome::AdmissionDenied(_)
        )
    ));
    let basis = PresentedObservationBasis {
        host_surface: world.current.host_surface,
        frame: frame_identity,
        epoch: world.current.epoch,
        instance: world.current.instance,
        receipt: world.current.receipt,
    };
    let raw = UiHostObservationReport::new(
        UiHostObservationSequence::new(1),
        UiHostObservationTimeBasis::HostMonotonicMillis(1),
        UiHostObservationPayload::Tick { tick: 1 },
    );
    let terminal_batch = batch(
        source(&world.session, world.binding, &basis),
        (1, 1),
        UiHostObservationLoss::Complete,
        vec![raw],
    );
    assert_denial(
        world
            .session
            .validate_host_observation_batch(terminal_batch),
        expected,
    );
}

fn assert_indeterminate_quarantine() {
    let mut world = published_observation_world("observation-basis-indeterminate");
    let frame = prepared(&mut world.session);
    let frame_identity = frame.canonical_core().frame();
    world.host.push_presentation(
        worth_ui_runtime::facade::mounted::UiHostSurfacePresentationOutcome::PresentationIndeterminate,
    );
    assert!(matches!(
        world.session.present_prepared_mounted_frame(
            frame,
            UiPresentationDeadline::at_tick(100),
            0,
        ),
        UiMountedFrameOutcome::PresentationIndeterminate(_)
    ));
    let basis = PresentedObservationBasis {
        host_surface: world.current.host_surface,
        frame: frame_identity,
        epoch: world.current.epoch,
        instance: world.current.instance,
        receipt: world.current.receipt,
    };
    let raw = UiHostObservationReport::new(
        UiHostObservationSequence::new(1),
        UiHostObservationTimeBasis::HostMonotonicMillis(1),
        UiHostObservationPayload::Tick { tick: 1 },
    );
    let indeterminate_batch = batch(
        source(&world.session, world.binding, &basis),
        (1, 1),
        UiHostObservationLoss::Complete,
        vec![raw],
    );
    let outcome = world
        .session
        .validate_host_observation_batch(indeterminate_batch.clone());
    assert!(matches!(
        outcome,
        UiHostObservationReportOutcome::Quarantined(_)
    ));
    assert_eq!(world.session.quarantined_host_observation_batch_count(), 1);
    assert!(matches!(
        world
            .session
            .validate_host_observation_batch(indeterminate_batch),
        UiHostObservationReportOutcome::Duplicate(_)
    ));
    assert_eq!(
        world.session.quarantined_host_observation_batch_count(),
        1,
        "an exact replay cannot consume another quarantine slot"
    );
    assert_eq!(world.session.retained_host_observation_report_count(), 0);
}

fn expect_validated(
    outcome: UiHostObservationReportOutcome,
) -> worth_ui::facade::observation_report::UiValidatedHostObservationBatch {
    match outcome {
        UiHostObservationReportOutcome::Validated(batch) => batch,
        other => panic!("expected validated batch, observed {other:?}"),
    }
}

fn assert_denial(outcome: UiHostObservationReportOutcome, expected: UiHostObservationReportDenial) {
    assert_eq!(outcome, UiHostObservationReportOutcome::Denied(expected));
}

fn range(first: u64, last: u64) -> UiHostObservationSequenceRange {
    UiHostObservationSequenceRange::new(
        UiHostObservationSequence::new(first),
        UiHostObservationSequence::new(last),
    )
}
