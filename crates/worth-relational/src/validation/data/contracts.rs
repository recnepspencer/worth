use serde::{Deserialize, Serialize};

use crate::transactions::data::{
    CreateIntent, EntityMutationIntent, MergedCommitPlan, MutationIntent, RelationMutationIntent,
};

use super::groups::{InvariantGroup, InvariantGroupSet};
use super::rules::InvariantRule;

/// What one plan's intents ask of the invariant engine.
///
/// Two questions are kept apart because they have different consequences. What
/// a plan **may invalidate** is an effect claim: topology inference, the
/// touched-partition walk, the working-state clone and the public commit
/// summary all read it, so overstating it makes a commit pay for and report
/// damage it never did. What a plan **must have rejudged** is only a selection
/// statement: it asks that a group's rules look at this plan, without claiming
/// the plan breaks anything in that group. A revalidation demand is exactly the
/// second and none of the first, so the two cannot share one field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct InvariantPlanContract {
    may_invalidate: InvariantGroupSet,
    must_rejudge: InvariantGroupSet,
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
        self.may_invalidate.is_empty() && self.must_rejudge.is_empty()
    }

    /// The groups this plan may actually damage.
    ///
    /// This is the effect claim, so it never includes a group a plan merely
    /// asked to be rejudged under. Callers that size work from it — topology
    /// inference, the clone, the public summary — stay honest by construction.
    pub fn may_invalidate_groups(self) -> InvariantGroupSet {
        self.may_invalidate
    }

    /// The groups whose rules this plan selects: what it may damage, plus what
    /// it asked to be judged by regardless. Selection widens; effects do not.
    pub fn selected_groups(self) -> InvariantGroupSet {
        self.may_invalidate.union(self.must_rejudge)
    }

    pub fn intersects_consumed_groups(self, consumed_groups: InvariantGroupSet) -> bool {
        self.is_empty() || self.selected_groups().intersects(consumed_groups)
    }

    pub(crate) fn applies_to_rule(self, rule: &InvariantRule) -> bool {
        if self.is_empty() {
            return true;
        }
        self.selected_groups().intersects(rule.groups())
    }

    fn observe_intent(&mut self, intent: &MutationIntent) {
        // A revalidation demand asks every rule that governs this record to
        // judge it again and damages nothing. Anything narrower than `all()`
        // would silently drop exactly the rule the demand was raised for, and
        // anything on the invalidation channel would bill the commit for
        // damage it never did.
        if let MutationIntent::Entity(EntityMutationIntent::Revalidate(_)) = intent {
            self.must_rejudge = InvariantGroupSet::all();
        }
        // A new endpoint can owe a minimum relation without changing an edge.
        // Select the rule without reporting a graph-topology mutation.
        if matches!(
            intent,
            MutationIntent::Create(CreateIntent::Entity(_))
                | MutationIntent::Create(CreateIntent::EntityAspects(_))
                | MutationIntent::Create(CreateIntent::BulkEntities(_))
        ) {
            self.must_rejudge = self
                .must_rejudge
                .union(InvariantGroupSet::of(InvariantGroup::RelationIntegrity));
        }
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
            // The demand handled above. It damages nothing, so it contributes
            // nothing to the effect claim; its selection widening already went
            // to `must_rejudge`.
            MutationIntent::Entity(EntityMutationIntent::Revalidate(_)) => {
                InvariantGroupSet::empty()
            }
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
    fn entity_create_selects_relation_minimum_without_claiming_graph_mutation() {
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
        assert!(contract
            .selected_groups()
            .contains(crate::validation::data::InvariantGroup::RelationIntegrity));
        assert!(!contract
            .may_invalidate_groups()
            .contains(crate::validation::data::InvariantGroup::RelationIntegrity));
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
