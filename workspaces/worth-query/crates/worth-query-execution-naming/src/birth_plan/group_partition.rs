//! Already-lowered group partitions attached to an execution birth plan.
use worth_relational::facade::aggregate_identity::{
    AggregateGroupScheme, AggregateIdentity, AggregateMemberRole,
};
use worth_relational::facade::lineage::PersistentNameIdentity;

use super::ExecutionNameBirthPlan;

impl ExecutionNameBirthPlan {
    /// Adds one group scheme's declared partition, each member by its
    /// persistent name and continuity role. Relational names the groups at
    /// commit finalization.
    pub fn with_group_partition(
        self,
        scheme: AggregateGroupScheme,
        groups: Vec<Vec<(PersistentNameIdentity, AggregateMemberRole)>>,
    ) -> Self {
        self.with_group_claim(scheme, Vec::new(), groups)
    }

    /// Adds one group scheme's claim: `dissolved` held groups release their
    /// members from the scheme, and `groups` is the declared partition.
    pub fn with_group_claim(
        mut self,
        scheme: AggregateGroupScheme,
        dissolved: Vec<AggregateIdentity>,
        groups: Vec<Vec<(PersistentNameIdentity, AggregateMemberRole)>>,
    ) -> Self {
        self.commit_plan
            .declare_group_partition(scheme, dissolved, groups);
        self
    }
}
