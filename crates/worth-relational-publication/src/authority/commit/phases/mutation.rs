use std::collections::BTreeSet;
use worth_relational_commit_admission::internal_port::intent_admission;
use worth_relational_contracts::internal_port as contracts;

use contracts::transactions::data::intents::mutation_intent::MutationIntent;
use contracts::transactions::data::outcomes::plan_artifacts::MergedCommitPlan;
use worth_relational_mutation::internal_port::authority::mutation::workspace::BranchLocalDeleteAllowance;
use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;

pub(crate) fn branch_local_delete_allowance_for_plan(
    selected_state: &impl CurrentPartitionAccess,
    working_state: &impl CurrentPartitionAccess,
    merged_plan: &MergedCommitPlan,
) -> BranchLocalDeleteAllowance {
    let mut entity_ids = BTreeSet::new();
    let mut relation_ids = BTreeSet::new();

    for intent in &merged_plan.merged_intents {
        match intent {
            MutationIntent::Entity(contracts::transactions::data::intents::mutation_intent::EntityMutationIntent::Delete(
                spec,
            )) => {
                if !intent_admission::entity_exists_in_state(
                    working_state,
                    spec.entity_id,
                ) && intent_admission::entity_exists_in_state(
                    selected_state,
                    spec.entity_id,
                ) {
                    entity_ids.insert(spec.entity_id);
                }
            }
            MutationIntent::Relation(
                contracts::transactions::data::intents::mutation_intent::RelationMutationIntent::UpdateEndpoints(spec),
            ) => {
                let relation_id = spec.relation_id;
                if !intent_admission::relation_exists_in_state(
                    working_state,
                    relation_id,
                ) && intent_admission::relation_exists_in_state(
                    selected_state,
                    relation_id,
                ) {
                    relation_ids.insert(relation_id);
                }
            }
            MutationIntent::Relation(
                contracts::transactions::data::intents::mutation_intent::RelationMutationIntent::ApplyAspectPatch(_),
            ) => {}
            MutationIntent::Relation(
                contracts::transactions::data::intents::mutation_intent::RelationMutationIntent::Delete(spec),
            ) => {
                let relation_id = spec.relation_id;
                if !intent_admission::relation_exists_in_state(
                    working_state,
                    relation_id,
                ) && intent_admission::relation_exists_in_state(
                    selected_state,
                    relation_id,
                ) {
                    relation_ids.insert(relation_id);
                }
            }
            MutationIntent::Create(_)
            | MutationIntent::Entity(_)
            | MutationIntent::Materialization(_)
            | MutationIntent::Relation(
                contracts::transactions::data::intents::mutation_intent::RelationMutationIntent::Revalidate(_),
            ) => {}
        }
    }

    BranchLocalDeleteAllowance {
        entity_ids,
        relation_ids,
    }
}
