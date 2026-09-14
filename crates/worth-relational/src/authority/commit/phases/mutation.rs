use std::collections::BTreeSet;

use crate::authority::mutation::BranchLocalDeleteAllowance;
use crate::branch::SelectedRelationalBranchState;
use crate::capabilities::StorageRead;
use crate::transactions::data::{MergedCommitPlan, MutationIntent};

pub(crate) fn branch_local_delete_allowance_for_plan(
    selected_state: &SelectedRelationalBranchState,
    working_state: &impl StorageRead,
    merged_plan: &MergedCommitPlan,
) -> BranchLocalDeleteAllowance {
    let selected_state = selected_state.state();
    let mut entity_ids = BTreeSet::new();
    let mut relation_ids = BTreeSet::new();

    for intent in &merged_plan.merged_intents {
        match intent {
            MutationIntent::Entity(crate::transactions::data::EntityMutationIntent::Delete(
                spec,
            )) => {
                if !crate::authority::intent_merge::entity_exists_in_state(
                    working_state,
                    spec.entity_id,
                ) && crate::authority::intent_merge::entity_exists_in_state(
                    selected_state,
                    spec.entity_id,
                ) {
                    entity_ids.insert(spec.entity_id);
                }
            }
            MutationIntent::Relation(
                crate::transactions::data::RelationMutationIntent::UpdateEndpoints(spec),
            ) => {
                let relation_id = spec.relation_id;
                if !crate::authority::intent_merge::relation_exists_in_state(
                    working_state,
                    relation_id,
                ) && crate::authority::intent_merge::relation_exists_in_state(
                    selected_state,
                    relation_id,
                ) {
                    relation_ids.insert(relation_id);
                }
            }
            MutationIntent::Relation(
                crate::transactions::data::RelationMutationIntent::ApplyAspectPatch(_),
            ) => {}
            MutationIntent::Relation(
                crate::transactions::data::RelationMutationIntent::Delete(spec),
            ) => {
                let relation_id = spec.relation_id;
                if !crate::authority::intent_merge::relation_exists_in_state(
                    working_state,
                    relation_id,
                ) && crate::authority::intent_merge::relation_exists_in_state(
                    selected_state,
                    relation_id,
                ) {
                    relation_ids.insert(relation_id);
                }
            }
            MutationIntent::Create(_)
            | MutationIntent::Entity(_)
            | MutationIntent::Materialization(_) => {}
        }
    }

    BranchLocalDeleteAllowance {
        entity_ids,
        relation_ids,
    }
}
