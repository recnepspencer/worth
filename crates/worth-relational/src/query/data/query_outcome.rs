//! The query result carries its original pinned-plan custody.
use super::fragment_reduction::{CanonicalQueryResult, QueryComplexitySummary};
use super::snapshot_pinned_plan::SnapshotPinnedQueryPlan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryExecutionOutcome {
    pub plan: SnapshotPinnedQueryPlan,
    pub result: CanonicalQueryResult,
    pub complexity: QueryComplexitySummary,
}
