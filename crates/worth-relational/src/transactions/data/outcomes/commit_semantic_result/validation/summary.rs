use super::RelationalInvariantExecutionMeaning;
use crate::transactions::data::outcomes::commit_validation::ValidationSummaryInput;
use crate::transactions::data::CommitValidationSummary;
use crate::validation::engine::InvariantExecutionSummary;

pub(crate) fn summarize(
    executions: &[RelationalInvariantExecutionMeaning],
) -> CommitValidationSummary {
    let mut summary = CommitValidationSummary::default();
    for execution in executions {
        let mut results = InvariantExecutionSummary::empty();
        for check in &execution.checks {
            results.observe_meaning(check.execution_point, check.failure_effect, &check.verdict);
        }
        summary.include_execution(ValidationSummaryInput {
            execution_point: execution.metadata.execution_point,
            observation_kind: execution.metadata.observation_kind,
            consumed_groups: execution.metadata.consumed_groups,
            applicable_groups: execution.metadata.applicable_groups,
            disposition: execution.metadata.disposition,
            has_merged_plan: execution.metadata.has_merged_plan,
            results: &results,
        });
    }
    summary
}
