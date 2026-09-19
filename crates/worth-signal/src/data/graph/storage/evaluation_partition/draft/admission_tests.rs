use super::*;
use crate::branch::owner_services::conditional_execution::SignalRetainedExecutionBasis;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStoragePreparation as Work,
    SignalConditionalRetentionLedger as Ledger,
    SignalConditionalRetentionReservation as Reservation,
};
use crate::runtime_policy::SignalConditionalEvaluationBudget;

#[test]
fn draft_admission_denies_before_activation_and_retains_rejected_inline_custody() {
    let mut graph = SignalGraph::new();
    graph.create_node();
    let bytes = 64 * 1024 * 1024;
    let ledger = Ledger::new(
        SignalConditionalEvaluationBudget {
            maximum_retained_slots: 1,
            maximum_retained_bytes: bytes,
            maximum_attempt_visits: 1_000_000,
        },
        crate::runtime_policy::SignalConditionalTemporalBudget {
            maximum_live_partitions: 1,
            maximum_reserved_active_wakes: 1,
        },
    );
    let basis =
        SignalRetainedExecutionBasis::capture(&mut graph, &ledger, &mut Work::new(1_000_000))
            .unwrap();
    let mut slot = basis
        .new_evaluation_partition(&mut Work::new(1_000_000))
        .unwrap();
    let initial = ledger.usage();
    assert!(matches!(
        ConditionalEvaluationDraft::begin(&mut slot, &mut Work::new(0)),
        Err(SignalError::ConditionalEvaluationWorkExhausted { .. })
    ));
    assert_eq!(ledger.usage(), initial);
    let pressure = ledger
        .reserve(
            0,
            Charge::capacity::<u8>(
                (bytes - initial.1 - std::mem::size_of::<Reservation>() as u64) as usize,
            )
            .unwrap(),
        )
        .unwrap();
    let full = ledger.usage();
    assert!(matches!(
        ConditionalEvaluationDraft::begin(&mut slot, &mut Work::new(1_000_000)),
        Err(SignalError::EvaluationStorageCapacityExhausted)
    ));
    assert_eq!(ledger.usage(), full);
    drop(pressure);
    let mut contacts = 0;
    let draft = ConditionalEvaluationDraft::begin(&mut slot, &mut Work::new(1_000_000)).unwrap();
    draft
        .partition
        .execute(&mut graph, |_| contacts += 1)
        .unwrap();
    draft.install();
    assert_eq!(contacts, 1);
    let installed = ledger.usage();
    for _ in 0..8 {
        ConditionalEvaluationDraft::begin(&mut slot, &mut Work::new(1_000_000))
            .unwrap()
            .install();
        assert_eq!(
            ledger.usage(),
            installed,
            "inline custody does not accumulate with attempts"
        );
    }
    let rejected = ConditionalEvaluationDraft::begin(&mut slot, &mut Work::new(1_000_000))
        .unwrap()
        .reject();
    assert!(ledger.usage().1 > installed.1);
    ledger.close();
    assert!(matches!(
        ConditionalEvaluationDraft::begin(&mut slot, &mut Work::new(1_000_000)),
        Err(SignalError::EvaluationStorageUnavailable)
    ));
    drop(slot);
    drop(basis);
    drop(graph);
    assert_eq!(ledger.usage().0, 0);
    assert!(ledger.usage().1 > 0);
    drop(rejected);
    assert_eq!(ledger.usage(), (0, 0));
}
