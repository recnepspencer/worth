use crate::query::data::{
    reduce_query_fragments, CanonicalQueryResult, QueryComplexitySummary, QueryExecutionOutcome,
    QueryWorkerFragment, SnapshotPinnedQueryPlan,
};
use crate::runtime::RelationalRuntime;
use worth_execution::ExecutionResourceLease;
use worth_foundational::ExecutionReport;

use super::super::query_preparation::{combine_query_reports, remaining_work_lease};
use super::leased_completion::complete_leased_query;
use super::QueryReadExecutionStop;

pub(in crate::visibility::materialization::read_records::reader) fn query_execution_outcome_maybe_leased(
    runtime: &RelationalRuntime,
    plan: SnapshotPinnedQueryPlan,
    packet_count: usize,
    touched_partitions: usize,
    target_count: usize,
    fragments: Vec<QueryWorkerFragment>,
    lease: Option<&ExecutionResourceLease<'_>>,
    prior_report: Option<ExecutionReport>,
) -> Result<(QueryExecutionOutcome, Option<ExecutionReport>), QueryReadExecutionStop> {
    let Some(lease) = lease else {
        return Ok((
            query_execution_outcome(
                runtime,
                plan,
                packet_count,
                touched_partitions,
                target_count,
                fragments,
            ),
            None,
        ));
    };
    let prior = prior_report.expect("leased packet execution has a report");
    let child = remaining_work_lease(lease, prior)?;
    let entity_count = fragments
        .iter()
        .map(|fragment| fragment.counters.authoritative_entity_records_emitted)
        .sum();
    let relation_count = fragments
        .iter()
        .map(|fragment| fragment.counters.authoritative_relation_records_emitted)
        .sum();
    let (result, completion) = complete_leased_query(
        plan.packet.execution_shape,
        plan.packet.ordering,
        fragments,
        &child,
    )
    .map_err(|stop| match stop {
        QueryReadExecutionStop::CompletionStopped { reason, report } => {
            QueryReadExecutionStop::CompletionStopped {
                reason,
                report: combine_query_reports(prior, report),
            }
        }
        other => other,
    })?;
    let outcome = assemble_outcome(
        runtime,
        plan,
        packet_count,
        touched_partitions,
        target_count,
        entity_count,
        relation_count,
        result,
    );
    Ok((outcome, Some(combine_query_reports(prior, completion))))
}

pub(in crate::visibility::materialization::read_records::reader) fn query_execution_outcome(
    runtime: &RelationalRuntime,
    plan: SnapshotPinnedQueryPlan,
    packet_count: usize,
    touched_partitions: usize,
    target_count: usize,
    fragments: Vec<crate::query::data::QueryWorkerFragment>,
) -> QueryExecutionOutcome {
    let entity_count = fragments
        .iter()
        .map(|fragment| fragment.counters.authoritative_entity_records_emitted)
        .sum();
    let relation_count = fragments
        .iter()
        .map(|fragment| fragment.counters.authoritative_relation_records_emitted)
        .sum();
    let result =
        reduce_query_fragments(plan.packet.execution_shape, plan.packet.ordering, fragments);
    assemble_outcome(
        runtime,
        plan,
        packet_count,
        touched_partitions,
        target_count,
        entity_count,
        relation_count,
        result,
    )
}

fn assemble_outcome(
    runtime: &RelationalRuntime,
    plan: SnapshotPinnedQueryPlan,
    packet_count: usize,
    touched_partitions: usize,
    target_count: usize,
    authoritative_entity_records_emitted: usize,
    authoritative_relation_records_emitted: usize,
    result: CanonicalQueryResult,
) -> QueryExecutionOutcome {
    let complexity = QueryComplexitySummary {
        packet_count,
        fragment_count: packet_count,
        touched_partitions,
        target_count,
        authoritative_entity_records_emitted,
        authoritative_relation_records_emitted,
    };
    runtime
        .performance_access()
        .count_query_emissions(result.entities.len(), result.relations.len());

    QueryExecutionOutcome {
        plan,
        result,
        complexity,
    }
}
