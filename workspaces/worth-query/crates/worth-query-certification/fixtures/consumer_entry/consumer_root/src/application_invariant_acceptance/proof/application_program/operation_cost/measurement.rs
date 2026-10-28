//! Native execution reports and actual registry population.
use super::*;
use worth_query_host::facade::primary_graph::{
    advancement_observations_on_this_thread_for_test,
    signal_request_charges_on_this_thread_for_test, WorthQueryExecutionObservation,
};
#[derive(Debug)]
pub(super) struct Retained {
    pub rows: usize,
}
pub(super) fn retained(runtime: &Runtime) -> Retained {
    let runtime = runtime.runtime();
    Retained {
        rows: runtime.registry_row_count_for_test(),
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
    pub whole_rows: u64,
}
impl Measure {
    pub fn begin(runtime: &Runtime) -> Self {
        let observation = WorthQueryExecutionObservation::install_on_this_thread();
        advancement_observations_on_this_thread_for_test();
        signal_request_charges_on_this_thread_for_test();
        Self {
            _observation: observation,
            whole_rows: runtime
                .runtime()
                .whole_registry_rows_visited_for_test()
                .expect("bounded registry observation"),
        }
    }
    pub fn finish(self, runtime: &Runtime) -> Work {
        // Capture before inspecting the receiver's registry population.
        let whole_rows = runtime
            .runtime()
            .whole_registry_rows_visited_for_test()
            .expect("bounded registry observation")
            - self.whole_rows;
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
            whole_rows,
        }
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
