//! Project the original result into descriptive commit-log evidence.
use super::InvariantExecutionResult;
use worth_relational_contracts::internal_port::transactions::data::commit_log::CommitInvariantEvaluation;

impl From<&InvariantExecutionResult> for CommitInvariantEvaluation {
    fn from(result: &InvariantExecutionResult) -> Self {
        let metadata = result.metadata();
        let summary = result.summary();
        let result_count = summary.result_count();
        let advisory_count = summary.advisory_count();
        let violation_count = summary.violation_count();
        let blocking_violation = summary.has_blocking_violation();
        let publication_violation = summary.has_publication_violation();

        Self {
            execution_point: metadata.execution_point(),
            observation_kind: metadata.observation_kind(),
            disposition: metadata.disposition(),
            preparation_selected_mode: metadata
                .preparation_strategy()
                .map(|strategy| strategy.selected_mode),
            preparation_parallel_legality: metadata
                .preparation_strategy()
                .map(|strategy| strategy.parallel_legality),
            preparation_parallel_profitability: metadata
                .preparation_strategy()
                .map(|strategy| strategy.parallel_profitability),
            preparation_serial_selection_reason: metadata
                .preparation_strategy()
                .and_then(|strategy| strategy.serial_selection_reason),
            consumed_groups: metadata.consumed_groups(),
            applicable_groups: metadata.applicable_groups(),
            result_count,
            advisory_count,
            violation_count,
            blocking_violation,
            publication_violation,
        }
    }
}
