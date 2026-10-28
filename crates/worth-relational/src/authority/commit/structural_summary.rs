use std::collections::BTreeSet;

use crate::authority::commit::touched_scope::{
    touched_partitions_for_flat_plan_set, touched_partitions_for_plan_set,
};
use crate::identity::data::PartitionId;
use crate::mvcc::bulk_reservations_for_plan;
use crate::storage::overlay::CurrentPartitionAccess;
use crate::transactions::data::{CommitTopology, MergedCommitPlan};
use crate::validation::data::InvariantPlanContract;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CommitStructuralSummary {
    pub(crate) invariant_contract: InvariantPlanContract,
    pub(crate) commit_topology: CommitTopology,
    pub(crate) touched_partitions: BTreeSet<PartitionId>,
    pub(crate) bulk_entity_slots_reserved: usize,
    pub(crate) bulk_relation_slots_reserved: usize,
}

impl CommitStructuralSummary {
    pub(crate) fn derive(
        current_state: &impl CurrentPartitionAccess,
        planning_state: &impl CurrentPartitionAccess,
        merged_plan: &MergedCommitPlan,
        merge_parent_count: usize,
    ) -> Self {
        let invariant_contract = merged_plan.invariant_contract();
        let commit_topology = merged_plan.inferred_topology(merge_parent_count);
        let touched_partitions = match commit_topology {
            CommitTopology::FlatEntityBatch if !merged_plan.revalidates_relations() => {
                touched_partitions_for_flat_plan_set(merged_plan)
            }
            CommitTopology::FlatEntityBatch => {
                touched_partitions_for_plan_set(current_state, merged_plan)
            }
            CommitTopology::GraphMutation | CommitTopology::BranchMerge => {
                touched_partitions_for_plan_set(current_state, merged_plan)
            }
        };
        let (bulk_entity_slots_reserved, bulk_relation_slots_reserved) =
            bulk_reservations_for_plan(planning_state, merged_plan);

        Self {
            invariant_contract,
            commit_topology,
            touched_partitions,
            bulk_entity_slots_reserved,
            bulk_relation_slots_reserved,
        }
    }

    pub(crate) fn public_summary(
        &self,
        descriptor_semantics_version: crate::schema::data::DescriptorSemanticsVersion,
    ) -> crate::transactions::data::CommitStructuralSummary {
        crate::transactions::data::CommitStructuralSummary {
            invariant_groups: self.invariant_contract.may_invalidate_groups(),
            commit_topology: self.commit_topology,
            touched_partitions: self.touched_partitions.iter().copied().collect(),
            bulk_entity_slots_reserved: self.bulk_entity_slots_reserved,
            bulk_relation_slots_reserved: self.bulk_relation_slots_reserved,
            descriptor_semantics_version,
        }
    }
}
