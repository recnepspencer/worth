#[allow(unused_imports)]
use crate::latency_interference::*;

#[test]
fn real_scheduler_assessment_certifies_without_laundering_sampled_rows() {
    let evidence = certified_latency_evidence(4);
    let queue_depth = evidence
        .rows()
        .iter()
        .find(|row| row.name() == worth_store_io_scheduler::InterferenceCounterName::QueuePeakDepth)
        .expect("scheduler assessment should carry queue-depth row");

    assert_eq!(queue_depth.value(), 4);
    assert_eq!(
        queue_depth.strength(),
        worth_store_budgets::CounterEvidenceStrength::Sampled
    );
    assert!(!receipt_rows(&evidence)
        .iter()
        .any(|(name, _)| name == "store.s6.queue.peak-depth"));
    assert!(receipt_rows(&evidence)
        .iter()
        .any(|(name, count)| name == "store.s6.queue.submitted-units"
            && *count == point_read_units(point_read_budget())));
    assert_eq!(
        receipt_rows(&evidence).len(),
        evidence
            .rows()
            .iter()
            .filter(|row| row.strength() == worth_store_budgets::CounterEvidenceStrength::Exact)
            .count()
    );
}
#[test]
fn certification_replay_preserves_policy_counter_and_proof_topology() {
    let first = certified_latency_evidence(3);
    let second = certified_latency_evidence(3);

    assert_eq!(first.status(), second.status());
    assert_eq!(first.rows(), second.rows());
    assert_eq!(receipt_rows(&first), receipt_rows(&second));
}
