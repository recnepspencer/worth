use std::sync::Arc;

use crate::facade::{
    BridgeConditionalDenialKind, BridgeManagedClockInstallationParts,
    BridgeManagedClockObservationOutcome, BridgeManagedClockObservationParts,
    BridgeManagedConditionalExecutionRequest, BridgeManagedTemporalDenialKind,
    BridgeManagedTemporalIntentIdentity, BridgeManagedTemporalIntentLifecycle,
    BridgeManagedTemporalIntentReconciliation, BridgeManagedTemporalIntentReconciliationParts,
};

use super::{always_eligible_contract, install};

fn installed_clock(
    maximum_active_intents: usize,
    maximum_due_wakes_per_observation: usize,
) -> (
    crate::facade::BridgeSealedRuntimeAssembly,
    crate::facade::BridgeManagedClockBinding,
    Arc<crate::facade::BridgeInstalledConditionalLowering>,
) {
    let (mut owner, lowering) = install(always_eligible_contract("query:one"), "managed-time");
    let binding = owner
        .install_managed_clock(BridgeManagedClockInstallationParts {
            lowering: &lowering,
            binding_identity: Arc::from("query:clock:billing"),
            source_identity: Arc::from("host:clock:billing"),
            timeline_identity: Arc::from("timeline:billing:v1"),
            maximum_active_intents,
            maximum_due_wakes_per_observation,
        })
        .expect("exact managed clock installs");
    (owner, binding, lowering)
}

fn active<'a>(
    binding: &'a crate::facade::BridgeManagedClockBinding,
    identity: &str,
    revision: u64,
    due_coordinate: u64,
) -> BridgeManagedTemporalIntentReconciliationParts<'a> {
    BridgeManagedTemporalIntentReconciliationParts {
        binding,
        identity: BridgeManagedTemporalIntentIdentity::declare(Arc::from(identity)).unwrap(),
        revision,
        due_coordinate,
        idempotency_identity: Arc::from(format!("idempotency:{identity}")),
        source_record_identity: crate::facade::RelationalBridgeRecordIdentityParts::entity(0, 1, 1),
        lifecycle: BridgeManagedTemporalIntentLifecycle::Active,
    }
}

#[test]
fn intent_reconciliation_is_revisioned_capacity_bounded_and_effect_safe() {
    let (mut owner, binding, _) = installed_clock(2, 2);
    assert_eq!(
        owner
            .reconcile_managed_temporal_intent(active(&binding, "intent:a", 1, 5))
            .unwrap(),
        BridgeManagedTemporalIntentReconciliation::Installed
    );
    assert_eq!(
        owner
            .reconcile_managed_temporal_intent(active(&binding, "intent:a", 1, 5))
            .unwrap(),
        BridgeManagedTemporalIntentReconciliation::Duplicate
    );
    let conflict = owner
        .reconcile_managed_temporal_intent(active(&binding, "intent:a", 1, 6))
        .expect_err("same revision cannot change due meaning");
    assert_eq!(
        conflict.kind(),
        BridgeManagedTemporalDenialKind::IntentRevisionConflict
    );
    assert_eq!(
        owner
            .reconcile_managed_temporal_intent(active(&binding, "intent:a", 2, 7))
            .unwrap(),
        BridgeManagedTemporalIntentReconciliation::Superseded
    );
    owner
        .reconcile_managed_temporal_intent(active(&binding, "intent:b", 1, 9))
        .unwrap();
    let saturated = owner
        .reconcile_managed_temporal_intent(active(&binding, "intent:c", 1, 11))
        .expect_err("capacity rejects before Signal scheduling");
    assert_eq!(
        saturated.kind(),
        BridgeManagedTemporalDenialKind::IntentCapacityExhausted
    );

    let closure = owner.close_managed_clock(binding).unwrap();
    assert_eq!(closure.active_intents(), 2);
    assert_eq!(closure.scheduled_wakes(), 2);
    assert_eq!(closure.ready_wakes(), 0);
}

#[test]
fn duplicate_observation_drains_only_the_remaining_bounded_due_frontier() {
    let (mut owner, binding, _) = installed_clock(3, 1);
    owner
        .reconcile_managed_temporal_intent(active(&binding, "intent:a", 1, 5))
        .unwrap();
    owner
        .reconcile_managed_temporal_intent(active(&binding, "intent:b", 1, 5))
        .unwrap();

    let observation = || BridgeManagedClockObservationParts {
        binding: &binding,
        source_identity: "host:clock:billing",
        timeline_identity: "timeline:billing:v1",
        sequence: 1,
        observed_coordinate: 5,
    };
    let BridgeManagedClockObservationOutcome::Accepted(first) =
        owner.observe_managed_clock(observation()).unwrap()
    else {
        panic!("first observation must advance the exact clock")
    };
    assert_eq!(first.due().wakes().len(), 1);
    assert!(first.due().due_work_remaining());
    assert_eq!(first.signal_advance_ordinal(), Some(1));

    let BridgeManagedClockObservationOutcome::Duplicate(second) =
        owner.observe_managed_clock(observation()).unwrap()
    else {
        panic!("exact repeated reading must be typed duplicate")
    };
    assert_eq!(second.due().wakes().len(), 1);
    assert!(!second.due().due_work_remaining());
    assert_eq!(second.signal_advance_ordinal(), None);
    assert_ne!(
        first.due().wakes()[0].intent_identity(),
        second.due().wakes()[0].intent_identity()
    );
}

#[test]
fn observation_affinity_and_ordering_fail_without_due_progress() {
    let (mut owner, binding, _) = installed_clock(1, 1);
    owner
        .reconcile_managed_temporal_intent(active(&binding, "intent:a", 1, 5))
        .unwrap();
    let accepted = owner
        .observe_managed_clock(BridgeManagedClockObservationParts {
            binding: &binding,
            source_identity: "host:clock:billing",
            timeline_identity: "timeline:billing:v1",
            sequence: 4,
            observed_coordinate: 3,
        })
        .unwrap();
    assert!(matches!(
        accepted,
        BridgeManagedClockObservationOutcome::Accepted(_)
    ));
    let stale = owner
        .observe_managed_clock(BridgeManagedClockObservationParts {
            binding: &binding,
            source_identity: "host:clock:billing",
            timeline_identity: "timeline:billing:v1",
            sequence: 3,
            observed_coordinate: 5,
        })
        .unwrap();
    assert!(matches!(stale, BridgeManagedClockObservationOutcome::Stale));
    let reordered = owner
        .observe_managed_clock(BridgeManagedClockObservationParts {
            binding: &binding,
            source_identity: "host:clock:billing",
            timeline_identity: "timeline:billing:v1",
            sequence: 5,
            observed_coordinate: 2,
        })
        .unwrap();
    assert!(matches!(
        reordered,
        BridgeManagedClockObservationOutcome::Reordered
    ));
    let foreign = match owner.observe_managed_clock(BridgeManagedClockObservationParts {
        binding: &binding,
        source_identity: "host:clock:foreign",
        timeline_identity: "timeline:billing:v1",
        sequence: 5,
        observed_coordinate: 5,
    }) {
        Ok(_) => panic!("foreign source cannot advance or promote"),
        Err(denial) => denial,
    };
    assert_eq!(
        foreign.kind(),
        BridgeManagedTemporalDenialKind::ForeignClockSource
    );
}

#[test]
fn managed_due_wake_executes_only_its_exact_conditional_lowering() {
    let (mut owner, binding, lowering) = installed_clock(1, 1);
    let signal_basis = owner
        .admit_conditional_signal_basis(&lowering, owner.admitted_signal_basis())
        .unwrap();
    owner
        .reconcile_managed_temporal_intent(active(&binding, "intent:a", 1, 5))
        .unwrap();
    let BridgeManagedClockObservationOutcome::Accepted(accepted) = owner
        .observe_managed_clock(BridgeManagedClockObservationParts {
            binding: &binding,
            source_identity: "host:clock:billing",
            timeline_identity: "timeline:billing:v1",
            sequence: 1,
            observed_coordinate: 5,
        })
        .unwrap()
    else {
        panic!("clock observation should promote one exact managed wake");
    };
    let mut wakes = accepted.into_due().into_wakes();
    let due = wakes.pop().expect("one due wake");

    let missing = owner
        .execute_managed_due_wake(
            BridgeManagedConditionalExecutionRequest {
                due_wake: &due,
                lowering: &lowering,
                signal_basis: &signal_basis,
                query_binding_identity: "query-binding:one",
                query_capability_identity: 1,
                snapshot_identity: "snapshot:one",
                truth_branch_identity: Some("main"),
                bridge_snapshot_identity: None,
                triggering_correspondence: None,
                attempt: 1,
            },
            &mut (),
        )
        .err()
        .expect("the due record cannot stand in for its source observation");
    assert_eq!(
        missing.kind(),
        BridgeConditionalDenialKind::MissingSourceObservation
    );
    assert_eq!(missing.bridge_execution_counters(), Default::default());
    assert_eq!(missing.signal_counters(), Default::default());

    let decision = owner
        .execute_managed_due_wake(
            BridgeManagedConditionalExecutionRequest {
                due_wake: &due,
                lowering: &lowering,
                signal_basis: &signal_basis,
                query_binding_identity: "query-binding:one",
                query_capability_identity: 1,
                snapshot_identity: "snapshot:one",
                truth_branch_identity: Some("main"),
                bridge_snapshot_identity: Some(&crate::truth_identity_fixtures::truth_snapshot(
                    1, 1,
                )),
                triggering_correspondence: None,
                attempt: 1,
            },
            &mut (),
        )
        .expect("exact due wake reaches its installed Signal conditional");
    assert_eq!(
        decision.signal().class(),
        worth_signal::facade::SignalConditionalDecisionClass::ComputedChanged
    );

    let (_foreign_owner, foreign_lowering) =
        install(always_eligible_contract("query:one"), "managed-time");
    let result = owner.execute_managed_due_wake(
        BridgeManagedConditionalExecutionRequest {
            due_wake: &due,
            lowering: &foreign_lowering,
            signal_basis: &signal_basis,
            query_binding_identity: "query-binding:one",
            query_capability_identity: 1,
            snapshot_identity: "snapshot:one",
            truth_branch_identity: Some("main"),
            bridge_snapshot_identity: None,
            triggering_correspondence: None,
            attempt: 2,
        },
        &mut (),
    );
    let Err(denial) = result else {
        panic!("a due wake cannot cross to another conditional lowering");
    };
    assert_eq!(
        denial.kind(),
        BridgeConditionalDenialKind::ManagedWakeMismatch
    );
}

mod resource_bounds;
