//! Native reports and separate real retention ledgers; none is a total-heap measurement.
use super::*;
use worth_query_host::facade::primary_graph::{
    advancement_observations_on_this_thread_for_test,
    signal_request_charges_on_this_thread_for_test, WorthQueryExecutionObservation,
};
#[derive(Debug)]
pub(super) struct Retained {
    pub rows: usize,
    pub record_bytes: usize,
    pub required_bytes: usize,
}
pub(super) fn retained(runtime: &Runtime) -> Retained {
    let runtime = runtime.runtime();
    Retained {
        rows: runtime.registry_row_count_for_test(),
        record_bytes: runtime.registry_record_retained_bytes_for_test(),
        required_bytes: runtime
            .required_custody_breakdown_for_test()
            .iter()
            .map(|(_, _, bytes)| bytes)
            .sum(),
    }
}
pub(super) struct Measure {
    _observation: WorthQueryExecutionObservation,
    whole_rows: u64,
}
#[derive(Debug)]
pub(super) struct Work {
    pub execution_work: u64,
    pub execution_span: u64,
    pub signal_checkpoint_work: u64,
    pub phases: usize,
    pub comparison_work: Vec<u64>,
    pub whole_rows: u64,
}
impl Measure {
    pub fn begin(runtime: &Runtime) -> Self {
        let observation = WorthQueryExecutionObservation::install_on_this_thread();
        advancement_observations_on_this_thread_for_test();
        signal_request_charges_on_this_thread_for_test();
        runtime
            .runtime()
            .take_required_set_comparison_work_for_test();
        Self {
            _observation: observation,
            whole_rows: runtime.runtime().whole_registry_rows_visited_for_test(),
        }
    }
    pub fn finish(self, runtime: &Runtime) -> Work {
        // Capture before any diagnostic custody breakdown walks registry records.
        let whole_rows = runtime.runtime().whole_registry_rows_visited_for_test() - self.whole_rows;
        let reports = advancement_observations_on_this_thread_for_test();
        assert!(
            reports.iter().all(|report| report.refusal.is_none()),
            "successful measured path"
        );
        Work {
            execution_work: reports
                .iter()
                .map(|report| report.report.charged_work())
                .sum(),
            execution_span: reports
                .iter()
                .map(|report| report.report.charged_span())
                .sum(),
            // Signal checkpoints exclude computation callback work; not ExecutionReport work.
            signal_checkpoint_work: signal_request_charges_on_this_thread_for_test()
                .into_iter()
                .sum(),
            phases: reports.len(),
            comparison_work: runtime
                .runtime()
                .take_required_set_comparison_work_for_test(),
            whole_rows,
        }
    }
}
pub(super) fn native_allocation(
) -> worth_query_host::facade::runtime::ExecutionResourceLease<'static> {
    // Same finite inline Serial ledger as the ordinary path; no physical pool.
    worth_query_host::facade::runtime::ExecutionResourceLease::serial(
        worth_query_host::facade::runtime::LeaseRequest {
            policy: resources::EXECUTION_POLICY,
            deadline: None,
            cancellation: worth_query_host::facade::runtime::CancellationToken::new(),
        },
    )
    .unwrap()
}

impl Work {
    pub(super) fn empty_comparison(&self) {
        assert!(self.comparison_work.is_empty());
    }
    pub(super) fn one_complete_comparison(&self) -> u64 {
        assert_eq!(
            self.comparison_work.len(),
            1,
            "one complete-set guard, never per-member proxy meters"
        );
        let work = self.comparison_work[0];
        assert!(work > 0);
        work
    }
}
#[derive(Debug)]
pub(super) struct Stages {
    pub traversal: Work,
    pub delivery: Work,
    pub cleanup: Work,
}
impl Stages {
    pub(super) fn execution_work(&self) -> u64 {
        self.traversal.execution_work + self.delivery.execution_work + self.cleanup.execution_work
    }
    pub(super) fn span(&self) -> u64 {
        self.traversal.execution_span + self.delivery.execution_span + self.cleanup.execution_span
    }
    pub(super) fn whole_rows(&self) -> u64 {
        self.traversal.whole_rows + self.delivery.whole_rows + self.cleanup.whole_rows
    }
}
