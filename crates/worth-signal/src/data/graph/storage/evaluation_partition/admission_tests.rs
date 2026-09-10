//! Initial storage admission and custody through native lifetime boundaries.
use super::{draft::ConditionalEvaluationDraft, SignalEvaluationPartition};
use crate::branch::owner_services::conditional_execution::SignalRetainedExecutionBasis;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStoragePreparation as Work,
    SignalConditionalRetentionLedger,
};
use crate::logic::transaction::SignalObservationRequest;
use crate::runtime_policy::SignalConditionalEvaluationBudget;
use std::sync::Arc;

fn fixture() -> (
    SignalGraph,
    SignalRetainedExecutionBasis,
    Arc<SignalConditionalRetentionLedger>,
) {
    let mut graph = SignalGraph::new();
    graph.create_node();
    let ledger = SignalConditionalRetentionLedger::new(
        SignalConditionalEvaluationBudget {
            maximum_retained_slots: 1,
            maximum_retained_bytes: 64 * 1024 * 1024,
            maximum_attempt_visits: 100_000,
        },
        crate::runtime_policy::SignalRuntimePolicy::development().conditional_temporal_budget,
    );
    let basis = SignalRetainedExecutionBasis::capture(&mut graph, &ledger, &mut Work::new(100_000))
        .unwrap();
    (graph, basis, ledger)
}

#[test]
fn partition_admission_denials_preserve_capacity_and_dropped_slots_can_be_reused() {
    let (_graph, basis, ledger) = fixture();
    let initial = ledger.usage();
    assert!(matches!(
        basis.new_evaluation_partition(&mut Work::new(0)),
        Err(SignalError::ConditionalEvaluationWorkExhausted { .. })
    ));
    assert_eq!(ledger.usage(), initial);
    let slot = basis
        .new_evaluation_partition(&mut Work::new(100_000))
        .unwrap();
    assert_eq!(ledger.usage().0, 1);
    assert!(ledger.usage().1 > initial.1);
    let full = ledger.usage();
    assert!(matches!(
        basis.new_evaluation_partition(&mut Work::new(100_000)),
        Err(SignalError::EvaluationStorageCapacityExhausted)
    ));
    assert_eq!(ledger.usage(), full);
    drop(slot);
    assert_eq!(ledger.usage(), initial);
    drop(
        basis
            .new_evaluation_partition(&mut Work::new(100_000))
            .unwrap(),
    );
    assert_eq!(ledger.usage(), initial);
    // Leave only enough byte capacity for the slot handle, then fail byte admission.
    let bytes = 64 * 1024 * 1024
        - initial.1
        - 2 * std::mem::size_of::<
            crate::data::retained_storage::SignalConditionalRetentionReservation,
        >() as u64;
    let pressure = ledger
        .reserve(0, Charge::capacity::<u8>(bytes as usize).unwrap())
        .unwrap();
    let pressured = ledger.usage();
    assert!(matches!(
        basis.new_evaluation_partition(&mut Work::new(100_000)),
        Err(SignalError::EvaluationStorageCapacityExhausted)
    ));
    assert_eq!(
        ledger.usage(),
        pressured,
        "failed byte admission releases slot custody"
    );
    drop(pressure);
    ledger.close();
    assert!(matches!(
        basis.new_evaluation_partition(&mut Work::new(100_000)),
        Err(SignalError::EvaluationStorageUnavailable)
    ));
    assert_eq!(ledger.usage(), initial);
}

#[test]
fn partition_admission_rejected_roots_retain_bytes_after_slot_and_seed_drop() {
    let (graph, basis, ledger) = fixture();
    let mut slot = basis
        .new_evaluation_partition(&mut Work::new(100_000))
        .unwrap();
    let rejected = ConditionalEvaluationDraft::begin(
        &mut slot,
        &mut crate::data::retained_storage::RetainedStoragePreparation::new(1_000_000),
    )
    .unwrap()
    .reject();
    drop(slot);
    drop(basis);
    drop(graph);
    assert_eq!(ledger.usage().0, 0);
    assert!(ledger.usage().1 > 0);
    drop(rejected);
    assert_eq!(ledger.usage(), (0, 0));
}

#[test]
fn partition_admission_escaped_session_retains_cleanup_until_drop() {
    let (mut graph, basis, ledger) = fixture();
    let mut slot = basis
        .new_evaluation_partition(&mut Work::new(100_000))
        .unwrap();
    let session = slot
        .execute(&mut graph, |selected| {
            selected
                .begin_observation_session(SignalObservationRequest::counters())
                .unwrap()
        })
        .unwrap();
    drop(slot);
    drop(basis);
    drop(graph);
    assert_eq!(ledger.usage().0, 0);
    assert!(ledger.usage().1 > 0);
    drop(session);
    assert_eq!(ledger.usage(), (0, 0));
}

#[test]
fn partition_admission_unwind_restores_custody_and_persistent_fork_retains_it() {
    let (mut graph, basis, ledger) = fixture();
    let mut slot: SignalEvaluationPartition = basis
        .new_evaluation_partition(&mut Work::new(100_000))
        .unwrap();
    let cold = ledger.usage();
    // The first admitted fork converts exclusive roots to shared backings. The
    // original partition retains that conversion even when the draft is rejected.
    drop(ConditionalEvaluationDraft::begin(&mut slot, &mut Work::new(1_000_000)).unwrap());
    let converted = ledger.usage();
    assert_eq!(converted.0, cold.0);
    assert!(converted.1 > cold.1);
    for _ in 0..2 {
        let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let draft =
                ConditionalEvaluationDraft::begin(&mut slot, &mut Work::new(1_000_000)).unwrap();
            let _ = draft
                .partition
                .execute(&mut graph, |_| panic!("provider unwind"));
        }));
        assert!(unwind.is_err());
        assert_eq!(
            ledger.usage(),
            converted,
            "unwind must not accumulate candidate custody"
        );
    }
    let fork = slot
        .execute(&mut graph, |selected| selected.fork_persistent().0)
        .unwrap();
    drop(slot);
    drop(basis);
    drop(graph);
    assert_eq!(ledger.usage().0, 0);
    assert!(ledger.usage().1 > 0);
    drop(fork);
    assert_eq!(ledger.usage(), (0, 0));
}
