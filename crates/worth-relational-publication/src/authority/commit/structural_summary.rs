use std::collections::BTreeSet;

use crate::authority::commit::touched_scope::{
    touched_partitions_for_flat_plan_set, touched_partitions_for_plan_set,
};
use worth_relational_contracts::internal_port::identity::data::primitives::PartitionId;
use worth_relational_contracts::internal_port::transactions::data::mutation_planning::CommitTopology;
use worth_relational_contracts::internal_port::transactions::data::outcomes::plan_artifacts::MergedCommitPlan;
use worth_relational_contracts::internal_port::validation::data::contracts::InvariantPlanContract;
use worth_relational_mutation::internal_port::authority::commit::plan_reservation_summary::bulk_reservations_for_plan;
use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitStructuralSummary {
    pub(crate) invariant_contract: InvariantPlanContract,
    pub(crate) commit_topology: CommitTopology,
    pub(crate) touched_partitions: BTreeSet<PartitionId>,
    pub(crate) bulk_entity_slots_reserved: usize,
    pub(crate) bulk_relation_slots_reserved: usize,
}

impl CommitStructuralSummary {
    pub(crate) fn derive(
        current_state: &impl CurrentPartitionAccess,
        merged_plan: &MergedCommitPlan,
    ) -> Self {
        let invariant_contract = merged_plan.invariant_contract();
        let commit_topology = merged_plan.inferred_topology();
        let touched_partitions = match commit_topology {
            CommitTopology::FlatEntityBatch if !merged_plan.revalidates_relations() => {
                touched_partitions_for_flat_plan_set(merged_plan)
            }
            CommitTopology::FlatEntityBatch | CommitTopology::GraphMutation => {
                touched_partitions_for_plan_set(current_state, merged_plan)
            }
        };
        let (bulk_entity_slots_reserved, bulk_relation_slots_reserved) =
            bulk_reservations_for_plan(merged_plan);

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
        descriptor_semantics_version: worth_relational_contracts::internal_port::schema::data::continuity::descriptor_versions::DescriptorSemanticsVersion,
    ) -> worth_relational_contracts::internal_port::transactions::data::outcomes::commit_summaries::CommitStructuralSummary{
        worth_relational_contracts::internal_port::transactions::data::outcomes::commit_summaries::CommitStructuralSummary {
            invariant_groups: self.invariant_contract.may_invalidate_groups(),
            commit_topology: self.commit_topology,
            touched_partitions: self.touched_partitions.iter().copied().collect(),
            bulk_entity_slots_reserved: self.bulk_entity_slots_reserved,
            bulk_relation_slots_reserved: self.bulk_relation_slots_reserved,
            descriptor_semantics_version,
        }
    }
}

#[doc(hidden)]
pub trait CommitStructuralSummaryFieldsPort: Sized {
    fn bulk_entity_slots_reserved(&self) -> usize;
    fn bulk_relation_slots_reserved(&self) -> usize;

    fn touched_partitions(&self) -> &BTreeSet<PartitionId>;
}

impl CommitStructuralSummaryFieldsPort for CommitStructuralSummary {
    fn bulk_entity_slots_reserved(&self) -> usize {
        self.bulk_entity_slots_reserved
    }

    fn bulk_relation_slots_reserved(&self) -> usize {
        self.bulk_relation_slots_reserved
    }

    fn touched_partitions(&self) -> &BTreeSet<PartitionId> {
        &self.touched_partitions
    }
}
