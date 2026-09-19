mod trigger_oracle;
use super::super::super::*;
use super::{budget, layout_oracle as layout, Owner};
use crate::policy::BridgeConditionalRetentionBudget;
use std::sync::Arc;

pub(super) fn decision_and_reentry(factory: &impl Fn(BridgeConditionalRetentionBudget) -> Owner) {
    ordinary_decision(factory);
    managed_trigger(factory);
    reentry_boundary(factory);
}

fn ordinary_decision(factory: &impl Fn(BridgeConditionalRetentionBudget) -> Owner) {
    let decision = layout::decision("snapshot:one", "execution:one", "binding:one");
    for ceiling in [
        layout::baseline() + decision,
        layout::baseline() + decision - 1,
    ] {
        let (owner, lowering) = factory(budget(ceiling));
        let basis = owner
            .admit_conditional_signal_basis(&lowering, owner.admitted_signal_basis())
            .unwrap();
        let source = crate::truth_identity_fixtures::truth_snapshot(1, 1);
        let mut contacts = 0usize;
        let result = owner.execute(
            &basis,
            BridgeConditionalExecutionRequest {
                lowering: &lowering,
                query_binding_identity: "binding:one",
                query_capability_identity: 1,
                snapshot_identity: "snapshot:one",
                truth_branch_identity: Some("main"),
                bridge_snapshot_identity: Some(&source),
                execution_identity: "execution:one",
                attempt: 1,
            },
            &mut contacts,
        );
        assert_eq!(result.is_ok(), ceiling == layout::baseline() + decision);
        assert_eq!(
            contacts,
            usize::from(result.is_ok()),
            "byte denial precedes compute"
        );
        if let Ok(evidence) = result {
            let ledger = Arc::clone(&owner.test_runtime().retention);
            let seed = evidence.retain_for_reentry();
            drop(evidence);
            drop(owner);
            assert_eq!(
                ledger.usage().2,
                layout::core("snapshot:one", "execution:one"),
                "seed retains the core's allocated bytes"
            );
            drop(seed);
            assert_eq!(ledger.usage(), (0, 0, 0));
        }
    }
}

fn managed_trigger(factory: &impl Fn(BridgeConditionalRetentionBudget) -> Owner) {
    let resident = layout::clock(1) + layout::binding() + layout::baseline() + layout::due(1);
    let execution = "managed-wake:intent:one:revision=1:signal=0:scheduled=1:ready=2";
    let decision = layout::decision("snapshot:one", execution, "binding:one");
    for deny_trigger in [false, true] {
        let (owner, lowering) = factory(if deny_trigger {
            budget(resident)
        } else {
            BridgeConditionalRetentionBudget::development()
        });
        let ledger = Arc::clone(&owner.test_runtime().retention);
        let basis = owner
            .admit_conditional_signal_basis(&lowering, owner.admitted_signal_basis())
            .unwrap();
        let worth_proof::TransitionOutcome::Success(trigger) =
            owner.deliver_owned_authoritative_change(&basis, 0).unwrap()
        else {
            panic!("actual source delivers");
        };
        let trigger_bytes = trigger_oracle::expected(trigger.change_set());
        if !deny_trigger {
            trigger_preparation_boundary(&trigger, trigger_bytes);
        }
        let binding = super::install(&owner, &lowering, "clock:one", 1).unwrap();
        owner
            .reconcile_managed_temporal_intent(super::active(&binding, "intent:one", 1, 5))
            .unwrap();
        let BridgeManagedClockObservationOutcome::Accepted(accepted) =
            super::observe(&owner, &binding).unwrap()
        else {
            panic!("due");
        };
        let mut wakes = accepted.into_due().into_wakes();
        let due = wakes.pop().unwrap();
        drop(wakes);
        assert_eq!(
            (
                due.signal_wake_id.get(),
                due.signal_scheduled_ordinal(),
                due.signal_ready_ordinal()
            ),
            (0, 1, 2)
        );
        let source = crate::truth_identity_fixtures::truth_snapshot(1, 1);
        let mut contacts = 0usize;
        let result = owner.execute_managed_due_wake(
            BridgeManagedConditionalExecutionRequest {
                due_wake: &due,
                lowering: &lowering,
                signal_basis: &basis,
                query_binding_identity: "binding:one",
                query_capability_identity: 1,
                snapshot_identity: "snapshot:one",
                truth_branch_identity: Some("main"),
                bridge_snapshot_identity: Some(&source),
                triggering_correspondence: Some(&trigger),
                attempt: 1,
            },
            &mut contacts,
        );
        if deny_trigger {
            assert!(
                matches!(result, Err(ref error) if error.kind() == BridgeConditionalDenialKind::ConditionalRetentionCapacity)
            );
            assert_eq!(contacts, 0);
            assert_eq!(
                ledger.usage().2,
                resident,
                "trigger denial precedes provider and leaves no copied trigger"
            );
            continue;
        }
        let evidence = result.unwrap();
        assert_eq!(contacts, 1);
        assert!(evidence.retains_triggering_correspondence(trigger.change_set()));
        assert_eq!(ledger.usage().2, resident + trigger_bytes + decision);
        let seed = evidence.retain_for_reentry();
        drop(evidence);
        let reentered = owner
            .reenter_retained_conditional_decision(BridgeConditionalDecisionReentryRequest {
                seed: &seed,
                lowering: &lowering,
                query_binding_identity: "binding:retained-after-trigger",
                query_capability_identity: 2,
                snapshot_identity: "snapshot:one",
                bridge_snapshot_identity: Some(&source),
            })
            .unwrap();
        assert!(reentered.retains_triggering_correspondence(trigger.change_set()));
        assert_eq!(contacts, 1, "trigger reentry does not call the provider");
        drop(reentered);
        owner.close_managed_clock(binding).unwrap();
        drop((due, owner, trigger));
        assert_eq!(
            ledger.usage().2,
            trigger_bytes + layout::core("snapshot:one", execution),
            "last seed owns trigger and core after runtime close"
        );
        drop(seed);
        assert_eq!(ledger.usage(), (0, 0, 0));
    }
}

fn trigger_preparation_boundary(
    trigger: &crate::correspondence::BridgeCorrespondenceDeliveryReceipt,
    expected: u64,
) {
    for ceiling in [expected, expected - 1] {
        let ledger =
            crate::conditional_execution::retention::BridgeRetentionLedger::new(budget(ceiling))
                .unwrap();
        let retained = crate::conditional_execution::retention::BridgeRetainedTrigger::prepare(
            &ledger,
            trigger.change_set(),
        );
        assert_eq!(retained.is_ok(), ceiling == expected);
        if let Err(error) = &retained {
            assert_eq!(
                error.kind(),
                BridgeConditionalDenialKind::ConditionalRetentionCapacity
            );
        }
        if let Ok(retained) = retained {
            assert!(retained.retains_same_delivery_as(trigger.change_set()));
            assert_eq!(ledger.usage().2, expected);
            ledger.close();
            assert_eq!(ledger.usage().2, expected);
            drop(retained);
        }
        assert_eq!(ledger.usage(), (0, 0, 0));
    }
}

fn reentry_boundary(factory: &impl Fn(BridgeConditionalRetentionBudget) -> Owner) {
    let binding = "retained-query-binding".repeat(64);
    let core = layout::core("snapshot:one", "execution:one");
    let required = core + layout::reentry(&binding);
    assert!(layout::reentry(&binding) > layout::baseline() + layout::evidence("binding:one"));
    for ceiling in [required, required - 1] {
        let (owner, lowering) = factory(budget(ceiling));
        let basis = owner
            .admit_conditional_signal_basis(&lowering, owner.admitted_signal_basis())
            .unwrap();
        let source = crate::truth_identity_fixtures::truth_snapshot(1, 1);
        let mut contacts = 0usize;
        let evidence = owner
            .execute(
                &basis,
                BridgeConditionalExecutionRequest {
                    lowering: &lowering,
                    query_binding_identity: "binding:one",
                    query_capability_identity: 1,
                    snapshot_identity: "snapshot:one",
                    truth_branch_identity: Some("main"),
                    bridge_snapshot_identity: Some(&source),
                    execution_identity: "execution:one",
                    attempt: 1,
                },
                &mut contacts,
            )
            .unwrap();
        let seed = evidence.retain_for_reentry();
        drop(evidence);
        let result =
            owner.reenter_retained_conditional_decision(BridgeConditionalDecisionReentryRequest {
                seed: &seed,
                lowering: &lowering,
                query_binding_identity: &binding,
                query_capability_identity: 2,
                snapshot_identity: "snapshot:one",
                bridge_snapshot_identity: Some(&source),
            });
        assert_eq!(result.is_ok(), ceiling == required);
        if let Err(error) = &result {
            assert_eq!(
                error.kind(),
                BridgeConditionalDenialKind::ConditionalRetentionCapacity
            );
        }
        assert_eq!(contacts, 1);
        if result.is_ok() {
            assert_eq!(owner.test_runtime().retention.usage().2, required);
        }
        drop(result);
        assert_eq!(owner.test_runtime().retention.usage().2, core);
        drop(seed);
        assert_eq!(owner.test_runtime().retention.usage(), (0, 0, 0));
    }
}
