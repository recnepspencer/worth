//! Already-lowered group partitions attached to an execution birth plan.
use worth_relational::facade::aggregate_identity::{AggregateGroupScheme, AggregateMemberRole};
use worth_relational::facade::lineage::PersistentNameIdentity;

use super::ExecutionNameBirthPlan;

impl ExecutionNameBirthPlan {
    /// Adds one group scheme's declared partition, each member by its
    /// persistent name and continuity role. Relational names the groups at
    /// commit finalization.
    pub fn with_group_partition(
        mut self,
        scheme: AggregateGroupScheme,
        groups: Vec<Vec<(PersistentNameIdentity, AggregateMemberRole)>>,
    ) -> Self {
        self.commit_plan
            .declare_group_partition(scheme, Vec::new(), groups);
        self
    }
}
