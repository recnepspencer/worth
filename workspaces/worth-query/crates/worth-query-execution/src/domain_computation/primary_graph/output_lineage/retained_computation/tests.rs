//! Ledger refusal and unmeasured state are different record absences.

use super::*;
use crate::domain_computation::primary_graph::application_contribution::{
    sealed_run_for_lineage_test, WorthQueryPartitionedComputationFullCause as Cause,
};

#[test]
fn overflow_is_unmeasured_and_a_real_ledger_refusal_is_evicted() {
    let mut run = sealed_run_for_lineage_test();
    run.state.overflow_bytes_for_test();
    let lineage = WorthQueryApplicationOutputLineage::default();
    assert!(matches!(
        lineage.retain_computation(
            run,
            None,
            None,
            lineage.prepay_computation_fork_scan_for_test()
        ),
        RecordedComputation::Absent(PriorAbsence::Unmeasured)
    ));
    let mut lineage = WorthQueryApplicationOutputLineage::default();
    lineage.retention = super::super::retained_capacity::LineageRetentionLedger::new(0);
    let recorded = lineage.retain_computation(
        sealed_run_for_lineage_test(),
        None,
        None,
        lineage.prepay_computation_fork_scan_for_test(),
    );
    assert!(matches!(
        recorded,
        RecordedComputation::Absent(PriorAbsence::Evicted)
    ));
    assert_eq!(PriorAbsence::Evicted.full_cause(), Cause::Evicted);
}

mod custody;
mod fork_selection;
mod history_window;
