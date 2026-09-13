use crate::branch::owner_services::SignalOwnerServiceIssuanceDenial;
use crate::data::graph::SignalGraph;
use crate::data::retained_storage::RetainedStorageCharge as Charge;
use crate::data::retained_storage::SignalConditionalRetentionDenial as Denial;
use crate::data::retained_storage::{
    SignalConditionalRetentionLedger, SignalConditionalRetentionReservation,
};
use crate::logic::transaction::SignalRuntime;
use crate::runtime_policy::{SignalConditionalEvaluationBudget, SignalRuntimePolicy};

fn budget() -> SignalConditionalEvaluationBudget {
    SignalConditionalEvaluationBudget {
        maximum_retained_slots: 2,
        maximum_retained_bytes: 100
            + 2 * std::mem::size_of::<SignalConditionalRetentionReservation>() as u64,
        maximum_attempt_visits: 97,
    }
}

#[test]
fn conditional_retention_reserves_both_axes_and_keeps_custody_after_owner_close() {
    let handle = std::mem::size_of::<SignalConditionalRetentionReservation>() as u64;
    let mut runtime = SignalRuntime::build_for::<()>(SignalGraph::new());
    runtime.set_runtime_policy(
        SignalRuntimePolicy::development().with_conditional_evaluation_budget(budget()),
    );
    let (_, mutation, _) = runtime.owner_port_slots().unwrap();
    let owner = mutation.upgrade_owner().unwrap();
    let weak_owner = std::sync::Arc::downgrade(&owner);
    let ledger = owner.conditional_retention.clone();
    drop(owner);
    assert_eq!(
        ledger.usage(),
        (0, 0),
        "sealing creates no pretend evaluation slot"
    );
    // Resource-owner proof only: these are reservations, not evaluated slots.
    let first = ledger
        .reserve(1, Charge::capacity::<u8>(60).unwrap())
        .unwrap();
    assert!(matches!(
        ledger.reserve(2, Charge::ZERO),
        Err(Denial::CapacityExhausted)
    ));
    assert!(matches!(
        ledger.reserve(1, Charge::capacity::<u8>(41).unwrap()),
        Err(Denial::CapacityExhausted)
    ));
    assert_eq!(
        ledger.usage(),
        (1, 60 + handle),
        "denial changes neither axis"
    );
    let second = ledger
        .reserve(1, Charge::capacity::<u8>(40).unwrap())
        .unwrap();
    assert_eq!(ledger.usage(), (2, 100 + 2 * handle));
    drop(first);
    assert_eq!(ledger.usage(), (1, 40 + handle));
    let returned_evidence_reservation = std::thread::scope(|scope| {
        scope
            .spawn(|| {
                ledger
                    .reserve(1, Charge::capacity::<u8>(60).unwrap())
                    .unwrap()
            })
            .join()
            .unwrap()
    });
    assert_eq!(ledger.usage(), (2, 100 + 2 * handle));
    drop(runtime);
    assert!(
        weak_owner.upgrade().is_none(),
        "reservations do not retain the owner"
    );
    assert!(matches!(
        ledger.reserve(0, Charge::ZERO),
        Err(Denial::Closed)
    ));
    assert_eq!(
        ledger.usage(),
        (2, 100 + 2 * handle),
        "close cannot erase outstanding custody"
    );
    drop(second);
    assert_eq!(ledger.usage(), (1, 60 + handle));
    drop(returned_evidence_reservation);
    assert_eq!(ledger.usage(), (0, 0));
}

#[test]
fn conditional_retention_growth_and_prepaid_split_preserve_exact_custody() {
    let handle = std::mem::size_of::<SignalConditionalRetentionReservation>();
    let maximum = 100 + 3 * handle;
    let ledger = SignalConditionalRetentionLedger::new(
        SignalConditionalEvaluationBudget {
            maximum_retained_bytes: maximum as u64,
            ..budget()
        },
        crate::runtime_policy::SignalRuntimePolicy::development().conditional_temporal_budget,
    );
    // The initial payload reservation prepays two future reservation handles.
    let mut parent = ledger
        .reserve(2, Charge::capacity::<u8>(20 + 2 * handle).unwrap())
        .unwrap();
    parent.grow(Charge::capacity::<u8>(80).unwrap()).unwrap();
    assert_eq!(ledger.usage(), (2, maximum as u64));
    assert_eq!(
        parent.grow(Charge::capacity::<u8>(1).unwrap()),
        Err(Denial::CapacityExhausted)
    );
    assert!(matches!(
        parent.split(3, Charge::ZERO),
        Err(Denial::InvalidTransfer)
    ));
    assert!(matches!(
        parent.split(0, Charge::capacity::<u8>(maximum).unwrap()),
        Err(Denial::InvalidTransfer)
    ));
    assert_eq!(ledger.usage(), (2, maximum as u64));
    let slot = parent
        .split(1, Charge::capacity::<u8>(30).unwrap())
        .unwrap();
    assert_eq!(ledger.usage(), (2, maximum as u64));
    ledger.close();
    assert_eq!(parent.grow(Charge::ZERO), Err(Denial::Closed));
    let evidence = parent
        .split(0, Charge::capacity::<u8>(20).unwrap())
        .unwrap();
    assert_eq!(
        ledger.usage(),
        (2, maximum as u64),
        "closed owner permits only prepaid transfer"
    );
    drop(parent);
    assert_eq!(ledger.usage(), (1, (50 + 2 * handle) as u64));
    drop(slot);
    assert_eq!(ledger.usage(), (0, (20 + handle) as u64));
    drop(evidence);
    assert_eq!(ledger.usage(), (0, 0));
}

#[test]
fn conditional_retention_ceiling_mismatch_denies_sealing_before_state_moves() {
    for slots_mismatch in [true, false] {
        let mut runtime = SignalRuntime::build_for::<()>(SignalGraph::new());
        let common = budget();
        runtime.set_runtime_policy(
            SignalRuntimePolicy::development().with_conditional_evaluation_budget(common),
        );
        let a = runtime.current_branch();
        let basis = runtime.observe_signal_branch_basis(a.clone()).unwrap();
        let b = runtime
            .fork_signal_branch("budget-b", &basis)
            .unwrap()
            .created_branch()
            .clone();
        runtime.switch_branch(b.clone()).unwrap();
        let mut changed = common;
        if slots_mismatch {
            changed.maximum_retained_slots += 1;
        } else {
            changed.maximum_retained_bytes += 1;
        }
        runtime.set_runtime_policy(
            SignalRuntimePolicy::development().with_conditional_evaluation_budget(changed),
        );
        assert!(matches!(
            runtime.owner_port_slots(),
            Err(SignalOwnerServiceIssuanceDenial::ConditionalRetentionBudgetMismatch)
        ));
        assert_eq!(runtime.current_branch().id, b.id);
        // Successful switching proves the rejected sealing did not consume the
        // legacy active/stored owners. Repair only the incompatible ceilings.
        runtime.switch_branch(a).unwrap();
        runtime.switch_branch(b).unwrap();
        let work_only = SignalConditionalEvaluationBudget {
            maximum_attempt_visits: 13,
            ..common
        };
        runtime.set_runtime_policy(
            SignalRuntimePolicy::development().with_conditional_evaluation_budget(work_only),
        );
        runtime.owner_port_slots().unwrap();
    }
}

#[test]
fn temporal_quota_mismatch_denies_sealing_before_state_moves() {
    for partitions_mismatch in [true, false] {
        let mut runtime = SignalRuntime::build_for::<()>(SignalGraph::new());
        let common = SignalRuntimePolicy::development();
        runtime.set_runtime_policy(common);
        let a = runtime.current_branch();
        let basis = runtime.observe_signal_branch_basis(a.clone()).unwrap();
        let b = runtime
            .fork_signal_branch("temporal-budget-b", &basis)
            .unwrap()
            .created_branch()
            .clone();
        runtime.switch_branch(b.clone()).unwrap();
        let mut changed = common.conditional_temporal_budget;
        if partitions_mismatch {
            changed.maximum_live_partitions += 1;
        } else {
            changed.maximum_reserved_active_wakes += 1;
        }
        runtime.set_runtime_policy(common.with_conditional_temporal_budget(changed));
        assert!(matches!(
            runtime.owner_port_slots(),
            Err(SignalOwnerServiceIssuanceDenial::ConditionalRetentionBudgetMismatch)
        ));
        assert_eq!(runtime.current_branch().id, b.id);
        runtime.switch_branch(a).unwrap();
        runtime.switch_branch(b).unwrap();
        runtime.set_runtime_policy(common);
        runtime.owner_port_slots().unwrap();
    }
}
