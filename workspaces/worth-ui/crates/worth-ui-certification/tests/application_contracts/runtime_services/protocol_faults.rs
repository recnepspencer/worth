use worth_ui::facade::{
    interaction::UiHostInteractionIngressOutcome,
    observation_report::{
        UiHostObservationBatch, UiHostObservationBatchInput, UiHostObservationLoss,
        UiHostObservationMountedBasis, UiHostObservationPayload, UiHostObservationReport,
        UiHostObservationReportDenial, UiHostObservationSequence, UiHostObservationSequenceRange,
        UiHostObservationTimeBasis, UI_HOST_OBSERVATION_BATCH_REPORT_LIMIT,
    },
};
use worth_ui_host_contract::{
    UiHostObservationPresentationBasis, UiHostPresentationEpoch, UiSurfaceBindingGeneration,
};
use worth_ui_runtime::facade::mounted::{UiMountedInstanceIdentity, UiMountedNodeReceiptIdentity};

use crate::{
    host_observation_fixture::protocol,
    intent::{
        run_native_runtime_service_scenario,
        runtime_services_kit::{
            run_cancelled_runtime_service_admission_scenario, run_headless_runtime_service_scenario,
        },
    },
    mounted_application_lifecycle::known_empty_surface_world::profile,
    mounted_application_lifecycle::published_mounted_world::{
        published_observation_world, PresentedObservationBasis, PublishedObservationWorld,
    },
};

#[derive(Debug, Eq, PartialEq)]
struct HostProtocolFaultEvidence {
    stale_denials: Box<[UiHostObservationReportDenial]>,
    reordered_denial: UiHostObservationReportDenial,
    duplicate_was_suppressed: bool,
    every_fault_world_reached_zero: bool,
    terminal_resources_zero: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DuplicateHostFaultEvidence {
    suppressed: bool,
    terminal_resources_zero: bool,
}

#[test]
fn rs_07_runtime_service_faults_fail_closed_and_reconcile_from_host_truth() {
    let cancellation = run_cancelled_runtime_service_admission_scenario();
    assert!(cancellation.cancelled_for_application_rebind);
    assert_eq!(cancellation.active_attempts_after, 0);
    assert!(cancellation.portal_truth_unchanged);
    assert!(cancellation.focus_truth_unchanged);
    assert!(cancellation.proposals_are_zero);
    assert!(cancellation.terminal_resources_are_zero);

    let host_faults = exercise_host_protocol_faults();
    assert_eq!(
        host_faults.stale_denials.as_ref(),
        &[
            UiHostObservationReportDenial::ForeignHostSession,
            UiHostObservationReportDenial::BindingNotPresented,
            UiHostObservationReportDenial::PresentationEpochMismatch,
            UiHostObservationReportDenial::MountedInstanceNotPresented,
        ]
    );
    assert_eq!(
        host_faults.reordered_denial,
        UiHostObservationReportDenial::SequenceReordered
    );
    assert!(host_faults.duplicate_was_suppressed);
    assert!(host_faults.every_fault_world_reached_zero);
    assert!(host_faults.terminal_resources_zero);

    let headless = run_headless_runtime_service_scenario();
    assert!(headless.hot_rebind_preserved_portal);
    assert!(headless.focus_retargeted_to_successor);
    assert!(headless.semantic.duplicate_was_idempotent);
    assert!(headless.semantic.terminal_resources_are_zero);

    let native = run_native_runtime_service_scenario();
    assert!(native.indeterminate_effect_retained);
    assert!(native.reconciled_from_exact_host_truth);
    assert!(native.predecessor_was_reconstructed);
    assert!(native.semantic.duplicate_was_idempotent);
    assert!(native.semantic.proposals_are_zero);
    assert!(native.semantic.terminal_resources_are_zero);
}

#[test]
fn rs_07_windows_fault_subset_stays_bounded_to_real_native_input_faults() {
    let evidence = crate::phase6_native_lifecycle::verify_native_fault_contract();

    assert!(evidence.qualified_schedules >= 15);
    assert!(evidence.state_event_pairs > 100);
    assert!(evidence.exact_capacity_preserved_sequence);
    assert!(evidence.over_capacity_stopped_before_retention);
    assert!(evidence.invalid_ime_range_stopped_before_retention);
}

fn exercise_host_protocol_faults() -> HostProtocolFaultEvidence {
    let mut ordering = published_observation_world("rs-07-report-ordering");
    let stale_denials = [
        foreign_session_denial(&mut ordering),
        foreign_binding_denial(&mut ordering),
        stale_presentation_denial(&mut ordering),
        stale_incarnation_denial(&mut ordering),
    ];
    let successor_binding = ordering.rebind_surface(profile(2));
    let successor_basis = ordering.current;
    let current_batch = observation_batch(
        &ordering,
        ordering.session.host_session_identity().as_u64(),
        successor_binding,
        successor_basis,
        (1, UI_HOST_OBSERVATION_BATCH_REPORT_LIMIT as u64),
        (1..=UI_HOST_OBSERVATION_BATCH_REPORT_LIMIT as u64)
            .map(|sequence| {
                observation_report(sequence, UiHostObservationPayload::Tick { tick: sequence })
            })
            .collect(),
    );
    assert!(matches!(
        ordering.session.admit_host_interaction_batch(current_batch),
        UiHostInteractionIngressOutcome::Applied(_)
    ));
    let reordered_batch = observation_batch(
        &ordering,
        ordering.session.host_session_identity().as_u64(),
        successor_binding,
        successor_basis,
        (1, 1),
        vec![observation_report(
            1,
            UiHostObservationPayload::Tick { tick: 1 },
        )],
    );
    let reordered_denial = denied(
        ordering
            .session
            .admit_host_interaction_batch(reordered_batch),
    );
    let terminal_resources_zero = shutdown_world_is_exact_zero(ordering);

    let duplicate = duplicate_is_suppressed();

    HostProtocolFaultEvidence {
        stale_denials: stale_denials.into(),
        reordered_denial,
        duplicate_was_suppressed: duplicate.suppressed,
        every_fault_world_reached_zero: terminal_resources_zero
            && duplicate.terminal_resources_zero,
        terminal_resources_zero,
    }
}

fn foreign_session_denial(world: &mut PublishedObservationWorld) -> UiHostObservationReportDenial {
    let batch = observation_batch(
        world,
        world.session.host_session_identity().as_u64() + 1,
        world.binding,
        world.current,
        (1, 1),
        vec![observation_report(
            1,
            UiHostObservationPayload::Tick { tick: 1 },
        )],
    );
    denied(world.session.admit_host_interaction_batch(batch))
}

fn foreign_binding_denial(world: &mut PublishedObservationWorld) -> UiHostObservationReportDenial {
    let batch = observation_batch(
        world,
        world.session.host_session_identity().as_u64(),
        UiSurfaceBindingGeneration::mint_unbound().unwrap(),
        world.current,
        (1, 1),
        vec![observation_report(
            1,
            UiHostObservationPayload::Tick { tick: 1 },
        )],
    );
    denied(world.session.admit_host_interaction_batch(batch))
}

fn stale_presentation_denial(
    world: &mut PublishedObservationWorld,
) -> UiHostObservationReportDenial {
    let mut stale = world.current;
    stale.epoch = UiHostPresentationEpoch::issued_by_host(stale.epoch.diagnostic_value() + 1);
    let batch = observation_batch(
        world,
        world.session.host_session_identity().as_u64(),
        world.binding,
        stale,
        (1, 1),
        vec![observation_report(
            1,
            UiHostObservationPayload::Tick { tick: 1 },
        )],
    );
    denied(world.session.admit_host_interaction_batch(batch))
}

fn stale_incarnation_denial(
    world: &mut PublishedObservationWorld,
) -> UiHostObservationReportDenial {
    let report = observation_report(
        1,
        UiHostObservationPayload::WindowFocus {
            surface: world.current.host_surface,
            focused: true,
        },
    )
    .with_mounted_basis(UiHostObservationMountedBasis::new(
        UiMountedInstanceIdentity::mint_unbound().unwrap(),
        UiMountedNodeReceiptIdentity::mint_unbound().unwrap(),
    ));
    let batch = observation_batch(
        world,
        world.session.host_session_identity().as_u64(),
        world.binding,
        world.current,
        (1, 1),
        vec![report],
    );
    denied(world.session.admit_host_interaction_batch(batch))
}

fn duplicate_is_suppressed() -> DuplicateHostFaultEvidence {
    let mut world = published_observation_world("rs-07-duplicate");
    let batch = observation_batch(
        &world,
        world.session.host_session_identity().as_u64(),
        world.binding,
        world.current,
        (1, 1),
        vec![observation_report(
            1,
            UiHostObservationPayload::Tick { tick: 1 },
        )],
    );
    assert!(matches!(
        world.session.admit_host_interaction_batch(batch.clone()),
        UiHostInteractionIngressOutcome::Applied(_)
    ));
    let duplicate = matches!(
        world.session.admit_host_interaction_batch(batch),
        UiHostInteractionIngressOutcome::Duplicate(_)
    );
    DuplicateHostFaultEvidence {
        suppressed: duplicate,
        terminal_resources_zero: shutdown_world_is_exact_zero(world),
    }
}

fn shutdown_world_is_exact_zero(world: PublishedObservationWorld) -> bool {
    let shutdown = world.session.shutdown();
    shutdown.mounted_presentation().is_empty()
        && shutdown.intent_resource_census().is_empty()
        && shutdown.runtime_service_resource_census().is_empty()
        && shutdown.rebind().is_empty()
        && matches!(
            shutdown.host_session_release(),
            Some(worth_ui_runtime::facade::host::UiHostSessionReleaseOutcome::Released(_))
        )
}

fn observation_batch(
    world: &PublishedObservationWorld,
    host_session: u64,
    binding: UiSurfaceBindingGeneration,
    basis: PresentedObservationBasis,
    sequences: (u64, u64),
    reports: Vec<UiHostObservationReport>,
) -> UiHostObservationBatch {
    UiHostObservationBatch::new(UiHostObservationBatchInput {
        protocol: protocol(),
        host_session,
        presentation: UiHostObservationPresentationBasis::new(
            basis.host_surface,
            basis.frame,
            binding,
            basis.epoch,
        ),
        sequences: UiHostObservationSequenceRange::new(
            UiHostObservationSequence::new(sequences.0),
            UiHostObservationSequence::new(sequences.1),
        ),
        loss: UiHostObservationLoss::Complete,
        reports,
    })
    .unwrap_or_else(|denial| {
        panic!(
            "RS-07 authored an invalid batch for host session {}: {denial:?}",
            world.session.host_session_identity().as_u64()
        )
    })
}

fn observation_report(sequence: u64, payload: UiHostObservationPayload) -> UiHostObservationReport {
    UiHostObservationReport::new(
        UiHostObservationSequence::new(sequence),
        UiHostObservationTimeBasis::HostMonotonicMillis(sequence),
        payload,
    )
}

fn denied(outcome: UiHostInteractionIngressOutcome) -> UiHostObservationReportDenial {
    match outcome {
        UiHostInteractionIngressOutcome::Denied(denial) => denial.denial(),
        other => panic!("stale host evidence opened a runtime path: {other:?}"),
    }
}
