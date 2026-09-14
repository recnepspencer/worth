use serde::{Deserialize, Serialize};

use crate::transactions::data::{
    CreateIntent, EntityMutationIntent, MergedCommitPlan, MutationIntent, RelationMutationIntent,
};

use super::groups::{InvariantGroup, InvariantGroupSet};
use super::rules::InvariantRule;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct InvariantPlanContract {
    may_invalidate: InvariantGroupSet,
}

impl InvariantPlanContract {
    pub fn from_merged_plan(plan: &MergedCommitPlan) -> Self {
        let mut contract = Self::default();
        for intent in &plan.merged_intents {
            contract.observe_intent(intent);
        }
        contract
    }

    pub fn is_empty(self) -> bool {
        self.may_invalidate.is_empty()
    }

    pub fn may_invalidate_groups(self) -> InvariantGroupSet {
        self.may_invalidate
    }

    pub fn intersects_consumed_groups(self, consumed_groups: InvariantGroupSet) -> bool {
        self.is_empty() || self.may_invalidate.intersects(consumed_groups)
    }

    pub(crate) fn applies_to_rule(self, rule: &InvariantRule) -> bool {
        if self.is_empty() {
            return true;
        }
        self.may_invalidate.intersects(rule.groups())
    }

    fn observe_intent(&mut self, intent: &MutationIntent) {
        let groups = match intent {
            MutationIntent::Create(CreateIntent::Entity(_))
            | MutationIntent::Create(CreateIntent::EntityAspects(_))
            | MutationIntent::Create(CreateIntent::BulkEntities(_)) => {
                InvariantGroupSet::of(InvariantGroup::StorageCoherence)
                    .union(InvariantGroupSet::of(InvariantGroup::IdentityCoherence))
                    .union(InvariantGroupSet::of(InvariantGroup::SchemaCompliance))
                    .union(InvariantGroupSet::of(InvariantGroup::PublicationCoherence))
                    .union(InvariantGroupSet::of(InvariantGroup::VersionVisibility))
            }
            MutationIntent::Entity(EntityMutationIntent::UpdateFields(_))
            | MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(_)) => {
                InvariantGroupSet::of(InvariantGroup::IdentityCoherence)
                    .union(InvariantGroupSet::of(InvariantGroup::SchemaCompliance))
            }
            MutationIntent::Entity(EntityMutationIntent::Replace(_)) => {
                InvariantGroupSet::of(InvariantGroup::AdjacencyIntegrity)
                    .union(InvariantGroupSet::of(InvariantGroup::IdentityCoherence))
                    .union(InvariantGroupSet::of(InvariantGroup::SchemaCompliance))
                    .union(InvariantGroupSet::of(InvariantGroup::RelationIntegrity))
                    .union(InvariantGroupSet::of(InvariantGroup::PublicationCoherence))
                    .union(InvariantGroupSet::of(InvariantGroup::VersionVisibility))
            }
            MutationIntent::Entity(EntityMutationIntent::Delete(_)) => {
                InvariantGroupSet::of(InvariantGroup::AdjacencyIntegrity)
                    .union(InvariantGroupSet::of(InvariantGroup::StorageCoherence))
                    .union(InvariantGroupSet::of(InvariantGroup::LineageIntegrity))
                    .union(InvariantGroupSet::of(InvariantGroup::RelationIntegrity))
                    .union(InvariantGroupSet::of(InvariantGroup::PublicationCoherence))
                    .union(InvariantGroupSet::of(InvariantGroup::VersionVisibility))
            }
            MutationIntent::Create(CreateIntent::Relation(_))
            | MutationIntent::Create(CreateIntent::RelationAspects(_))
            | MutationIntent::Create(CreateIntent::BulkRelations(_)) => {
                InvariantGroupSet::of(InvariantGroup::AdjacencyIntegrity)
                    .union(InvariantGroupSet::of(InvariantGroup::StorageCoherence))
                    .union(InvariantGroupSet::of(InvariantGroup::SchemaCompliance))
                    .union(InvariantGroupSet::of(InvariantGroup::RelationIntegrity))
                    .union(InvariantGroupSet::of(InvariantGroup::PublicationCoherence))
                    .union(InvariantGroupSet::of(InvariantGroup::VersionVisibility))
            }
            MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(_)) => {
                InvariantGroupSet::of(InvariantGroup::AdjacencyIntegrity)
                    .union(InvariantGroupSet::of(InvariantGroup::StorageCoherence))
                    .union(InvariantGroupSet::of(InvariantGroup::SchemaCompliance))
                    .union(InvariantGroupSet::of(InvariantGroup::RelationIntegrity))
                    .union(InvariantGroupSet::of(InvariantGroup::PublicationCoherence))
                    .union(InvariantGroupSet::of(InvariantGroup::VersionVisibility))
            }
            MutationIntent::Relation(RelationMutationIntent::ApplyAspectPatch(_)) => {
                InvariantGroupSet::of(InvariantGroup::StorageCoherence)
                    .union(InvariantGroupSet::of(InvariantGroup::IdentityCoherence))
                    .union(InvariantGroupSet::of(InvariantGroup::SchemaCompliance))
                    .union(InvariantGroupSet::of(InvariantGroup::RelationIntegrity))
                    .union(InvariantGroupSet::of(InvariantGroup::PublicationCoherence))
                    .union(InvariantGroupSet::of(InvariantGroup::VersionVisibility))
            }
            MutationIntent::Relation(RelationMutationIntent::Delete(_)) => {
                InvariantGroupSet::of(InvariantGroup::AdjacencyIntegrity)
                    .union(InvariantGroupSet::of(InvariantGroup::StorageCoherence))
                    .union(InvariantGroupSet::of(InvariantGroup::RelationIntegrity))
                    .union(InvariantGroupSet::of(InvariantGroup::PublicationCoherence))
                    .union(InvariantGroupSet::of(InvariantGroup::VersionVisibility))
            }
            MutationIntent::Materialization(_) => InvariantGroupSet::all(),
        };
        self.may_invalidate = self.may_invalidate.union(groups);
    }
}

#[cfg(test)]
mod tests {
    use super::InvariantPlanContract;
    use crate::identity::data::{EntityId, Generation, KindId, LocalSlot, PartitionId};
    use crate::symbols::data::ClientKey;
    use crate::transactions::data::{
        BulkEntityCreateIntent, CreateIntent, DeleteEntityIntent, EntityMutationIntent, EntitySpec,
        MergedCommitPlan, MutationIntent, ReplaceEntityIntent, TransactionId,
    };

    #[test]
    fn contract_marks_entity_create_as_aspect_patch_and_uniqueness_sensitive() {
        let plan = MergedCommitPlan {
            transaction_id: TransactionId(1),
            merged_intents: vec![MutationIntent::Create(CreateIntent::BulkEntities(
                BulkEntityCreateIntent {
                    partition_id: PartitionId(7),
                    kind_id: KindId(9),
                    client_keys: vec![ClientKey::raw("a")],
                    field_patches: vec![crate::transactions::data::AspectFieldPatch::default()],
                },
            ))],
        };

        let contract = InvariantPlanContract::from_merged_plan(&plan);
        assert!(!contract.may_invalidate_groups().is_empty());
        assert!(contract
            .may_invalidate_groups()
            .contains(crate::validation::data::InvariantGroup::SchemaCompliance));
    }

    #[test]
    fn contract_marks_entity_delete_without_field_patch_or_uniqueness_surface() {
        let plan = MergedCommitPlan {
            transaction_id: TransactionId(2),
            merged_intents: vec![MutationIntent::Entity(EntityMutationIntent::Delete(
                DeleteEntityIntent {
                    entity_id: EntityId::new(PartitionId(1), LocalSlot(0).0, Generation(1).0),
                },
            ))],
        };

        let contract = InvariantPlanContract::from_merged_plan(&plan);
        assert!(contract
            .may_invalidate_groups()
            .contains(crate::validation::data::InvariantGroup::AdjacencyIntegrity));
        assert!(contract
            .may_invalidate_groups()
            .contains(crate::validation::data::InvariantGroup::LineageIntegrity));
    }

    #[test]
    fn contract_marks_entity_replace_as_relation_integrity_sensitive() {
        let plan = MergedCommitPlan {
            transaction_id: TransactionId(3),
            merged_intents: vec![MutationIntent::Entity(EntityMutationIntent::Replace(
                ReplaceEntityIntent {
                    entity_id: EntityId::new(PartitionId(1), LocalSlot(0).0, Generation(1).0),
                    replacement: EntitySpec {
                        partition_id: PartitionId(1),
                        kind_id: KindId(9),
                        client_key: ClientKey::raw("replacement"),
                        fields: crate::transactions::data::AspectFieldPatch::default(),
                    },
                },
            ))],
        };

        let contract = InvariantPlanContract::from_merged_plan(&plan);
        assert!(contract
            .may_invalidate_groups()
            .contains(crate::validation::data::InvariantGroup::RelationIntegrity));
        assert!(contract
            .may_invalidate_groups()
            .contains(crate::validation::data::InvariantGroup::AdjacencyIntegrity));
    }

    #[test]
    fn contract_keeps_entity_field_update_out_of_snapshot_publication_groups() {
        let plan = MergedCommitPlan {
            transaction_id: TransactionId(4),
            merged_intents: vec![MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                crate::transactions::data::UpdateEntityFieldsIntent {
                    entity_id: EntityId::new(PartitionId(1), LocalSlot(0).0, Generation(1).0),
                    fields: crate::transactions::data::AspectFieldPatch::default(),
                },
            ))],
        };

        let contract = InvariantPlanContract::from_merged_plan(&plan);
        assert!(contract
            .may_invalidate_groups()
            .contains(crate::validation::data::InvariantGroup::IdentityCoherence));
        assert!(contract
            .may_invalidate_groups()
            .contains(crate::validation::data::InvariantGroup::SchemaCompliance));
        assert!(!contract
            .may_invalidate_groups()
            .contains(crate::validation::data::InvariantGroup::VersionVisibility));
        assert!(!contract
            .may_invalidate_groups()
            .contains(crate::validation::data::InvariantGroup::PublicationCoherence));
    }
}
