use std::collections::BTreeSet;

use crate::lineage::access::LineageAccess;
use crate::lineage::data::{
    LineageDivergenceMetrics, LineageDivergenceRequest, LineageDivergenceSummary,
    LineageGraphRequest,
};

impl<'runtime> LineageAccess<'runtime> {
    pub fn divergence_between_branches(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: LineageDivergenceRequest,
    ) -> Result<LineageDivergenceSummary, crate::execution::RelationalExecutionDenialCause> {
        let left_graph = self.graph(execution, LineageGraphRequest {
            branch_id: request.left_branch.clone(),
            traversal_basis:
                crate::lineage::data::LineageGraphTraversalBasis::FullBranchGraphMaterialization,
        })?;
        let right_graph = self.graph(execution, LineageGraphRequest {
            branch_id: request.right_branch.clone(),
            traversal_basis:
                crate::lineage::data::LineageGraphTraversalBasis::FullBranchGraphMaterialization,
        })?;
        self.runtime
            .performance_access()
            .count_lineage_branch_divergence(
                left_graph.metrics.event_count,
                right_graph.metrics.event_count,
                left_graph.metrics.node_count,
                right_graph.metrics.node_count,
            );
        let left_event_ids = left_graph
            .events
            .iter()
            .map(|event| event.event_id)
            .collect::<BTreeSet<_>>();
        let right_event_ids = right_graph
            .events
            .iter()
            .map(|event| event.event_id)
            .collect::<BTreeSet<_>>();
        let left_lineage_ids = left_graph
            .nodes
            .iter()
            .map(|node| node.lineage_id)
            .collect::<BTreeSet<_>>();
        let right_lineage_ids = right_graph
            .nodes
            .iter()
            .map(|node| node.lineage_id)
            .collect::<BTreeSet<_>>();
        let shared_lineage_ids = left_lineage_ids
            .intersection(&right_lineage_ids)
            .copied()
            .collect::<Vec<_>>();
        let shared_lineage_count = shared_lineage_ids.len();
        Ok(LineageDivergenceSummary {
            left_branch: request.left_branch,
            right_branch: request.right_branch,
            traversal_basis: request.traversal_basis,
            left_only_event_ids: left_event_ids
                .difference(&right_event_ids)
                .copied()
                .collect(),
            right_only_event_ids: right_event_ids
                .difference(&left_event_ids)
                .copied()
                .collect(),
            shared_lineage_ids,
            metrics: LineageDivergenceMetrics {
                left_event_count: left_graph.metrics.event_count,
                right_event_count: right_graph.metrics.event_count,
                left_node_count: left_graph.metrics.node_count,
                right_node_count: right_graph.metrics.node_count,
                shared_lineage_count,
            },
        })
    }
}

// Fixture adaptation (C2.4): restores the cfg(test)-only `Divergence` door
// from ea2ffee250:cold_lineage_event_matrix and 81d41f5575:cold_lineage_matrix
// across all six failure classes (sticky and transient).
#[test]
fn divergence_cold_lineage_doors_preserve_every_failure_class() {
    use crate::execution::RelationalExecutionDenialCause as Execution;
    use crate::identity::data::LineageId;
    use crate::storage::data::{
        RelationalStorageSubject as Subject, RelationalStorageUnavailable as Unavailable,
        RelationalStorageUnavailableCause as Cause,
    };
    use std::sync::atomic::Ordering;
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let failures = [
        (Cause::Damaged, true),
        (Cause::IntegrityMismatch, true),
        (Cause::MissingMember, true),
        (Cause::LimitExceeded { length: 9, limit: 8 }, false),
        (Cause::MemoryExhausted, false),
        (Cause::SourceUnavailable, false),
    ];
    let req = || LineageDivergenceRequest {
        left_branch: crate::history::data::BranchId("main".into()),
        right_branch: crate::history::data::BranchId("peer".into()),
        traversal_basis:
            crate::lineage::data::LineageDivergenceTraversalBasis::FullBranchGraphComparison,
    };
    for (event_fault, (cause, sticky)) in [false, true]
        .into_iter()
        .flat_map(|ef| failures.into_iter().map(move |f| (ef, f)))
    {
        super::graph::with_cold_lineage_reopened(
            execution,
            |lineage, access, history, source, (n_mem, n_first, n_last), (e_mem, e_first, e_last)| {
                let frontier = history.latest_patch_stream_position();
                let (member, subject) = if event_fault {
                    (
                        e_mem,
                        Subject::Lineage {
                            first: LineageId(e_first),
                            last: LineageId(e_last),
                        },
                    )
                } else {
                    (
                        n_mem,
                        Subject::Lineage {
                            first: LineageId(n_first),
                            last: LineageId(n_last),
                        },
                    )
                };
                source.failures.lock().unwrap().insert(member, cause);
                let expected = Execution::StorageUnavailable(Unavailable { subject, cause });
                assert_eq!(
                    access.divergence_between_branches(execution, req()),
                    Err(expected.clone())
                );
                source.failures.lock().unwrap().clear();
                let reads = source.reads.load(Ordering::Relaxed);
                if sticky {
                    assert_eq!(
                        access.divergence_between_branches(execution, req()),
                        Err(expected)
                    );
                    assert_eq!(source.reads.load(Ordering::Relaxed), reads);
                } else {
                    let summary = access.divergence_between_branches(execution, req()).unwrap();
                    assert!(!summary.shared_lineage_ids.is_empty());
                    assert!(source.reads.load(Ordering::Relaxed) > reads);
                    assert!(lineage.node(summary.shared_lineage_ids[0]).is_some());
                }
                assert_eq!(history.latest_patch_stream_position(), frontier);
            },
        );
    }
}
