//! Logical aggregate custody for an actual applied business revision.
use super::projection::AggregateProjection;
use std::sync::Arc;
use worth_relational_branch::internal_port::branch::root::RelationalBranchRoot;
use worth_relational_contracts::internal_port::identity::data::primitives::VersionId;

#[derive(Clone)]
pub struct AggregateCandidateRead {
    pub(super) projection: Arc<AggregateProjection>,
    pub(super) before: Arc<RelationalBranchRoot>,
    pub(super) before_version: VersionId,
}
impl AggregateCandidateRead {
    pub(super) fn new(
        projection: Arc<AggregateProjection>,
        before: Arc<RelationalBranchRoot>,
        before_version: VersionId,
    ) -> Self {
        Self {
            projection,
            before,
            before_version,
        }
    }
}
impl std::fmt::Debug for AggregateCandidateRead {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AggregateCandidateRead")
            .field("before", &self.before.id())
            .finish_non_exhaustive()
    }
}
