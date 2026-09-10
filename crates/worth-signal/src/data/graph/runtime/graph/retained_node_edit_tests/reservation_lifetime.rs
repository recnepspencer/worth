use super::*;
use crate::data::retained_storage::{
    SignalConditionalRetentionDenial, SignalConditionalRetentionLedger,
};

#[test]
fn staged_root_capacity_denial_preserves_performed_output_without_installing() {
    let arena = prepared_arena();
    let original = arena.clone();
    let maximum = Charge::capacity::<u8>(16 * 1024 * 1024).unwrap();
    let probe = test_ledger();
    let prepared = arena
        .prepare_retained_node_edits(
            &probe,
            &[0],
            maximum,
            &mut Work::new(PREPARATION_WORK),
            |_| 7,
        )
        .unwrap();
    assert!(matches!(&prepared, RetainedNodeEditPreparation::Ready(_)));
    let peak = probe.usage().1;
    drop(prepared);
    assert_eq!(probe.usage(), (0, 0));
    let ledger = SignalConditionalRetentionLedger::new(
        crate::runtime_policy::SignalConditionalEvaluationBudget {
            maximum_retained_slots: 1,
            maximum_retained_bytes: peak - 1,
            maximum_attempt_visits: PREPARATION_WORK,
        },
        crate::runtime_policy::SignalRuntimePolicy::development().conditional_temporal_budget,
    );
    let result = arena
        .prepare_retained_node_edits(
            &ledger,
            &[0],
            maximum,
            &mut Work::new(PREPARATION_WORK),
            |_| 7,
        )
        .unwrap();
    assert!(matches!(
        result,
        RetainedNodeEditPreparation::Rejected {
            output: 7,
            denial: RetainedNodeEditDenial::Retention(
                SignalConditionalRetentionDenial::CapacityExhausted
            )
        }
    ));
    assert!(arena.hot.shares_storage_with(&original.hot));
    assert!(arena.warm.shares_storage_with(&original.warm));
    assert!(arena.cold.shares_storage_with(&original.cold));
    assert_eq!(ledger.usage(), (0, 0));
}

#[test]
fn owner_capacity_denial_precedes_payload_edit_and_preserves_roots() {
    let arena = prepared_arena();
    let original = arena.clone();
    let ledger = SignalConditionalRetentionLedger::new(
        crate::runtime_policy::SignalConditionalEvaluationBudget {
            maximum_retained_slots: 1,
            maximum_retained_bytes: 1,
            maximum_attempt_visits: PREPARATION_WORK,
        },
        crate::runtime_policy::SignalRuntimePolicy::development().conditional_temporal_budget,
    );
    let result = arena.prepare_retained_node_edits(
        &ledger,
        &[0],
        representation_charge(&arena),
        &mut Work::new(PREPARATION_WORK),
        |_| panic!("owner capacity denial must precede cloning and the callback"),
    );
    assert!(matches!(
        result,
        Err(RetainedNodeEditDenial::Retention(
            SignalConditionalRetentionDenial::CapacityExhausted
        ))
    ));
    assert!(arena.hot.shares_storage_with(&original.hot));
    assert!(arena.warm.shares_storage_with(&original.warm));
    assert!(arena.cold.shares_storage_with(&original.cold));
    assert_eq!(ledger.usage(), (0, 0));
}

#[test]
fn installed_custody_follows_shared_roots_but_not_operational_materialization() {
    let mut arena = prepared_arena();
    let ledger = test_ledger();
    let maximum = Charge::capacity::<u8>(16 * 1024 * 1024).unwrap();
    let RetainedNodeEditPreparation::Ready(prepared) = arena
        .prepare_retained_node_edits(
            &ledger,
            &[0],
            maximum,
            &mut Work::new(PREPARATION_WORK),
            |payloads| {
                payloads[0].hot.state = NodeState::Clean;
            },
        )
        .unwrap()
    else {
        panic!("admitted roots")
    };
    let prepared_usage = ledger.usage();
    assert!(prepared_usage.1 > 0);
    let RetainedNodeEditOutcome::Installed { charge, .. } = prepared.install(&mut arena) else {
        panic!("fresh storage installs")
    };
    let installed_usage = ledger.usage();
    assert!(installed_usage.1 >= charge.bytes());
    assert!(
        installed_usage.1 < prepared_usage.1,
        "temporary preparation custody is released"
    );
    let retained = arena.fork_persistent();
    let cloned = retained.clone();
    let materialized = arena.operational_clone();
    assert_eq!(
        ledger.usage(),
        installed_usage,
        "sharing roots shares custody"
    );
    drop(arena);
    assert_eq!(ledger.usage(), installed_usage);
    drop(retained);
    assert_eq!(ledger.usage(), installed_usage);
    drop(cloned);
    assert_eq!(ledger.usage(), (0, 0));
    assert_eq!(
        materialized.hot[0].as_ref().unwrap().state,
        NodeState::Clean
    );
    drop(materialized);
    assert_eq!(ledger.usage(), (0, 0));
}

#[test]
fn rejected_staged_growth_and_closed_owner_release_preparation_custody() {
    let arena = prepared_arena();
    let ledger = test_ledger();
    let result = arena
        .prepare_retained_node_edits(
            &ledger,
            &[0],
            Charge::ZERO,
            &mut Work::new(PREPARATION_WORK),
            |_| 7,
        )
        .unwrap();
    assert!(matches!(
        result,
        RetainedNodeEditPreparation::Rejected {
            output: 7,
            denial: RetainedNodeEditDenial::CapacityExhausted { .. }
        }
    ));
    assert_eq!(ledger.usage(), (0, 0));
    ledger.close();
    let result = arena.prepare_retained_node_edits(
        &ledger,
        &[0],
        representation_charge(&arena),
        &mut Work::new(PREPARATION_WORK),
        |_| panic!("closed owner must deny before the edit"),
    );
    assert!(matches!(
        result,
        Err(RetainedNodeEditDenial::Retention(
            SignalConditionalRetentionDenial::Closed
        ))
    ));
    assert_eq!(ledger.usage(), (0, 0));
}
