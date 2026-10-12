//! staged with fresh read set.
use super::*;

pub(in super::super) fn staged_with_fresh_read_set<'run>(
    execution: &'run crate::domain_computation::primary_graph::TestHostRequest<'_>,

    running: &'run mut WorthQueryRunningDirectRun,
    graph: &WorthQueryInstalledGraphParticipationAuthority,
) -> (
    crate::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run>,
    WorthQueryFreshDecisionReadSet,
) {
    let staged = {
        let installed_fixture_phase = running.fixture_phase(execution);
        running
            .admit_provider_execution_plan(graph)
            .unwrap()
            .readmit(&installed_fixture_phase)
    }
    .unwrap()
    .prepare()
    .unwrap()
    .bind_reads_and_effects();
    let fresh = {
        let reads = staged.read_authority();
        let receipt = reads
            .capture_decision_read_set(
                [
                    "base",
                    "old",
                    "source-1",
                    "search-1",
                    "candidate-a",
                    "transform-1",
                    "policy-1",
                    "correspondence-1",
                    "identity-map-1",
                ]
                .into_iter()
                .map(|identity| {
                    WorthQueryDecisionFactRequest::new(
                        "planning-basis",
                        WorthQueryDecisionFactLocator::observed(identity).unwrap(),
                    )
                    .unwrap()
                }),
                crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                None,
            )
            .unwrap();
        match reads
            .compare_decision_read_set(&execution.borrowed_request_for_test(), receipt)
            .unwrap()
        {
            WorthQueryDecisionReadSetFreshnessOutcome::Fresh(fresh) => fresh,
            WorthQueryDecisionReadSetFreshnessOutcome::Stale(_) => {
                panic!("fixture read set must remain fresh")
            }
        }
    };
    (staged, fresh)
}
