//! Candidate-local kind narrowing before a custom rule spends its own work budget.

use std::collections::BTreeSet;

use crate::identity::data::{EntityId, KindId, RelationId};
use crate::transactions::data::{
    CreateIntent, EntityMutationIntent, EntityReference, MutationIntent, RecordRef,
    RelationMutationIntent,
};
use crate::validation::data::CustomInvariantAccessContract;
use crate::validation::engine::state_view::InvariantStateView;
use crate::validation::engine::InvariantExecutionRequest;

pub(super) struct CandidateTouchedKinds {
    entities: BTreeSet<KindId>,
    relations: BTreeSet<KindId>,
}

impl CandidateTouchedKinds {
    /// `None` means a complete footprint could not be proven; all custom rules
    /// then retain the ordinary scope-preparation path.
    pub(super) fn from_request(request: &InvariantExecutionRequest<'_>) -> Option<Self> {
        let plan = request.merged_plan()?;
        let observation = request.observation();
        let proposed = InvariantStateView::new(
            observation.enforcement_partition_access(),
            observation.enforcement_version_id(request.version_id()),
        );
        // A plan-only observation may have no mutation journal. The ordinary
        // scope collector likewise gets no touched slots in that case and
        // derives its applicability from the complete merged intents.
        let partitions = proposed.state().touched_partition_ids().unwrap_or_default();
        let before = observation.before_image_partition_access().map(|access| {
            InvariantStateView::new(
                access,
                observation.before_image_version_id(request.current_version_id()),
            )
        });
        let mut kinds = Self {
            entities: BTreeSet::new(),
            relations: BTreeSet::new(),
        };
        for partition in partitions {
            proposed.state().get_partition(partition)?;
            for slot in proposed.state().touched_entity_slots(partition)? {
                let current = proposed.entity_metadata_for_slot(partition, slot);
                let previous = before
                    .as_ref()
                    .and_then(|view| view.entity_metadata_for_slot(partition, slot));
                if current.is_none() && previous.is_none() {
                    return None;
                }
                for metadata in [current, previous].into_iter().flatten() {
                    kinds.entities.insert(metadata.kind_id);
                }
            }
            for slot in proposed.state().touched_relation_slots(partition)? {
                let current = proposed.relation_metadata_for_slot(partition, slot);
                let previous = before
                    .as_ref()
                    .and_then(|view| view.relation_metadata_for_slot(partition, slot));
                if current.is_none() && previous.is_none() {
                    return None;
                }
                for metadata in [current, previous].into_iter().flatten() {
                    kinds.relation_metadata(
                        metadata.kind_id,
                        metadata.source,
                        metadata.target,
                        &proposed,
                        before.as_ref(),
                    )?;
                }
            }
        }
        for intent in &plan.merged_intents {
            kinds.intent(intent, &proposed, before.as_ref())?;
        }
        Some(kinds)
    }

    pub(super) fn may_affect(&self, access: &CustomInvariantAccessContract) -> bool {
        access
            .affected_entity_kinds
            .iter()
            .any(|kind| self.entities.contains(kind))
            || access
                .affected_relation_kinds
                .iter()
                .any(|kind| self.relations.contains(kind))
    }

    fn entity(
        &mut self,
        id: EntityId,
        proposed: &InvariantStateView<'_>,
        before: Option<&InvariantStateView<'_>>,
    ) -> Option<()> {
        let current = proposed.entity_metadata(id);
        let previous = before.and_then(|view| view.entity_metadata(id));
        if current.is_none() && previous.is_none() {
            return None;
        }
        for metadata in [current, previous].into_iter().flatten() {
            self.entities.insert(metadata.kind_id);
        }
        Some(())
    }

    fn relation(
        &mut self,
        id: RelationId,
        proposed: &InvariantStateView<'_>,
        before: Option<&InvariantStateView<'_>>,
    ) -> Option<()> {
        let current = proposed.relation_metadata(id);
        let previous = before.and_then(|view| view.relation_metadata(id));
        if current.is_none() && previous.is_none() {
            return None;
        }
        for metadata in [current, previous].into_iter().flatten() {
            self.relation_metadata(
                metadata.kind_id,
                metadata.source,
                metadata.target,
                proposed,
                before,
            )?;
        }
        Some(())
    }

    fn relation_metadata(
        &mut self,
        kind: KindId,
        source: EntityId,
        target: EntityId,
        proposed: &InvariantStateView<'_>,
        before: Option<&InvariantStateView<'_>>,
    ) -> Option<()> {
        self.relations.insert(kind);
        self.entity(source, proposed, before)?;
        self.entity(target, proposed, before)
    }

    fn reference(
        &mut self,
        reference: &EntityReference,
        proposed: &InvariantStateView<'_>,
        before: Option<&InvariantStateView<'_>>,
    ) -> Option<()> {
        match reference {
            EntityReference::Existing(id) => self.entity(*id, proposed, before),
            EntityReference::Created(created) => {
                self.entities.insert(created.kind_id);
                Some(())
            }
        }
    }

    fn intent(
        &mut self,
        intent: &MutationIntent,
        proposed: &InvariantStateView<'_>,
        before: Option<&InvariantStateView<'_>>,
    ) -> Option<()> {
        match intent {
            MutationIntent::Create(create) => match create {
                CreateIntent::Entity(spec) => {
                    self.entities.insert(spec.kind_id);
                    Some(())
                }
                CreateIntent::EntityAspects(spec) => {
                    self.entities.insert(spec.kind_id);
                    Some(())
                }
                CreateIntent::BulkEntities(spec) => {
                    self.entities.insert(spec.kind_id);
                    Some(())
                }
                CreateIntent::Relation(spec) => {
                    self.relations.insert(spec.kind_id);
                    self.reference(&spec.source, proposed, before)?;
                    self.reference(&spec.target, proposed, before)
                }
                CreateIntent::RelationAspects(spec) => {
                    self.relations.insert(spec.kind_id);
                    self.reference(&spec.source, proposed, before)?;
                    self.reference(&spec.target, proposed, before)
                }
                CreateIntent::BulkRelations(spec) => {
                    self.relations.insert(spec.kind_id);
                    for (source, target) in &spec.endpoints {
                        self.reference(source, proposed, before)?;
                        self.reference(target, proposed, before)?;
                    }
                    Some(())
                }
            },
            MutationIntent::Entity(entity) => match entity {
                EntityMutationIntent::UpdateFields(spec) => {
                    self.entity(spec.entity_id, proposed, before)
                }
                EntityMutationIntent::ApplyAspectPatch(spec) => {
                    self.entity(spec.entity_id, proposed, before)
                }
                // Replacement or deletion can affect implicit old adjacency.
                // Keep the ordinary planner authoritative for these shapes.
                EntityMutationIntent::Replace(_) | EntityMutationIntent::Delete(_) => None,
                EntityMutationIntent::Revalidate(spec) => {
                    self.entity(spec.entity_id, proposed, before)
                }
            },
            MutationIntent::Relation(relation) => match relation {
                // Endpoint moves and deletion need before/after adjacency
                // semantics beyond a direct kind footprint.
                RelationMutationIntent::UpdateEndpoints(_) => None,
                RelationMutationIntent::ApplyAspectPatch(spec) => {
                    self.relation(spec.relation_id, proposed, before)
                }
                RelationMutationIntent::Delete(_) => None,
            },
            MutationIntent::Materialization(materialization) => match materialization.record() {
                RecordRef::Entity(id) => self.entity(id, proposed, before),
                RecordRef::Relation(id) => self.relation(id, proposed, before),
            },
        }
    }
}

#[cfg(test)]
#[path = "custom_applicability_tests.rs"]
mod tests;
