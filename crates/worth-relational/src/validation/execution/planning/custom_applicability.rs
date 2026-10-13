//! Candidate-local kind narrowing before a custom rule spends its own work budget.

use std::ops::ControlFlow;

use crate::storage::substrate::SharedMap;
use crate::validation::custom_rule::CustomPreparationBudget;

mod touched_journal;

use crate::identity::data::{EntityId, KindId, RelationId};
use crate::transactions::data::{
    CreateIntent, EntityMutationIntent, EntityReference, MutationIntent, RecordRef,
    RelationMutationIntent,
};
use crate::validation::data::CustomInvariantAccessContract;
use crate::validation::engine::state_view::InvariantStateView;
use crate::validation::engine::InvariantExecutionRequest;

pub(super) struct CandidateTouchedKinds {
    entities: SharedMap<KindId, ()>,
    relation_endpoints: SharedMap<KindId, ()>,
    relations: SharedMap<KindId, ()>,
}

impl CandidateTouchedKinds {
    /// `None` means a complete footprint could not be proven; all custom rules
    /// then retain the ordinary scope-preparation path.
    pub(super) fn from_request(
        request: &InvariantExecutionRequest<'_>,
        budget: Option<&CustomPreparationBudget>,
    ) -> Option<Self> {
        let plan = request.merged_plan()?;
        let observation = request.observation();
        let proposed = InvariantStateView::new(
            observation.enforcement_partition_access(),
            observation.enforcement_version_id(request.version_id()),
        );
        // A plan-only observation may have no mutation journal. The ordinary
        // scope collector likewise gets no touched slots in that case and
        // derives its applicability from the complete merged intents.
        let before = observation.before_image_partition_access().map(|access| {
            InvariantStateView::new(
                access,
                observation.before_image_version_id(request.current_version_id()),
            )
        });
        let mut kinds = Self {
            entities: SharedMap::new(),
            relation_endpoints: SharedMap::new(),
            relations: SharedMap::new(),
        };
        let mut complete = true;
        proposed
            .state()
            .visit_touched_partition_ids(&mut |partition| {
                if kinds
                    .partition(partition, &proposed, before.as_ref(), budget)
                    .is_none()
                {
                    complete = false;
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            });
        if !complete {
            return None;
        }
        for intent in &plan.merged_intents {
            pay_visit(budget)?;
            kinds.intent(intent, &proposed, before.as_ref(), budget)?;
        }
        Some(kinds)
    }

    pub(super) fn may_affect(&self, access: &CustomInvariantAccessContract) -> bool {
        access
            .affected_entity_kinds
            .iter()
            .any(|kind| self.entities.contains_key(kind))
            || (access.include_relation_endpoint_entity_touches
                && access
                    .affected_entity_kinds
                    .iter()
                    .any(|kind| self.relation_endpoints.contains_key(kind)))
            || access
                .affected_relation_kinds
                .iter()
                .any(|kind| self.relations.contains_key(kind))
    }

    fn entity(
        &mut self,
        id: EntityId,
        proposed: &InvariantStateView<'_>,
        before: Option<&InvariantStateView<'_>>,
        budget: Option<&CustomPreparationBudget>,
    ) -> Option<()> {
        pay_visit(budget)?;
        let current = proposed.entity_metadata(id);
        let previous = before.and_then(|view| view.entity_metadata(id));
        if current.is_none() && previous.is_none() {
            return None;
        }
        for metadata in [current, previous].into_iter().flatten() {
            insert_kind(&mut self.entities, metadata.kind_id, budget)?;
        }
        Some(())
    }

    fn relation(
        &mut self,
        id: RelationId,
        proposed: &InvariantStateView<'_>,
        before: Option<&InvariantStateView<'_>>,
        budget: Option<&CustomPreparationBudget>,
    ) -> Option<()> {
        pay_visit(budget)?;
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
                budget,
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
        budget: Option<&CustomPreparationBudget>,
    ) -> Option<()> {
        insert_kind(&mut self.relations, kind, budget)?;
        self.relation_endpoint(source, proposed, before, budget)?;
        self.relation_endpoint(target, proposed, before, budget)
    }

    fn relation_endpoint(
        &mut self,
        id: EntityId,
        proposed: &InvariantStateView<'_>,
        before: Option<&InvariantStateView<'_>>,
        budget: Option<&CustomPreparationBudget>,
    ) -> Option<()> {
        pay_visit(budget)?;
        let current = proposed.entity_metadata(id);
        let previous = before.and_then(|view| view.entity_metadata(id));
        if current.is_none() && previous.is_none() {
            return None;
        }
        for metadata in [current, previous].into_iter().flatten() {
            insert_kind(&mut self.relation_endpoints, metadata.kind_id, budget)?;
        }
        Some(())
    }

    fn reference(
        &mut self,
        reference: &EntityReference,
        proposed: &InvariantStateView<'_>,
        before: Option<&InvariantStateView<'_>>,
        budget: Option<&CustomPreparationBudget>,
    ) -> Option<()> {
        match reference {
            EntityReference::Existing(id) => self.relation_endpoint(*id, proposed, before, budget),
            EntityReference::Created(created) => {
                insert_kind(&mut self.relation_endpoints, created.kind_id, budget)
            }
        }
    }

    fn intent(
        &mut self,
        intent: &MutationIntent,
        proposed: &InvariantStateView<'_>,
        before: Option<&InvariantStateView<'_>>,
        budget: Option<&CustomPreparationBudget>,
    ) -> Option<()> {
        match intent {
            MutationIntent::Create(create) => match create {
                CreateIntent::Entity(spec) => insert_kind(&mut self.entities, spec.kind_id, budget),
                CreateIntent::EntityAspects(spec) => {
                    insert_kind(&mut self.entities, spec.kind_id, budget)
                }
                CreateIntent::BulkEntities(spec) => {
                    insert_kind(&mut self.entities, spec.kind_id, budget)
                }
                CreateIntent::Relation(spec) => {
                    insert_kind(&mut self.relations, spec.kind_id, budget)?;
                    self.reference(&spec.source, proposed, before, budget)?;
                    self.reference(&spec.target, proposed, before, budget)
                }
                CreateIntent::RelationAspects(spec) => {
                    insert_kind(&mut self.relations, spec.kind_id, budget)?;
                    self.reference(&spec.source, proposed, before, budget)?;
                    self.reference(&spec.target, proposed, before, budget)
                }
                CreateIntent::BulkRelations(spec) => {
                    insert_kind(&mut self.relations, spec.kind_id, budget)?;
                    for (source, target) in &spec.endpoints {
                        pay_visit(budget)?;
                        self.reference(source, proposed, before, budget)?;
                        self.reference(target, proposed, before, budget)?;
                    }
                    Some(())
                }
            },
            MutationIntent::Entity(entity) => match entity {
                EntityMutationIntent::UpdateFields(spec) => {
                    self.entity(spec.entity_id, proposed, before, budget)
                }
                EntityMutationIntent::ApplyAspectPatch(spec) => {
                    self.entity(spec.entity_id, proposed, before, budget)
                }
                // Replacement or deletion can affect implicit old adjacency.
                // Keep the ordinary planner authoritative for these shapes.
                EntityMutationIntent::Replace(_) | EntityMutationIntent::Delete(_) => None,
                EntityMutationIntent::Revalidate(spec) => {
                    self.entity(spec.entity_id, proposed, before, budget)
                }
            },
            MutationIntent::Relation(relation) => match relation {
                // Endpoint moves and deletion need before/after adjacency
                // semantics beyond a direct kind footprint.
                RelationMutationIntent::UpdateEndpoints(_) => None,
                RelationMutationIntent::ApplyAspectPatch(spec) => {
                    self.relation(spec.relation_id, proposed, before, budget)
                }
                RelationMutationIntent::Delete(_) => None,
            },
            MutationIntent::Materialization(materialization) => match materialization.record() {
                RecordRef::Entity(id) => self.entity(id, proposed, before, budget),
                RecordRef::Relation(id) => self.relation(id, proposed, before, budget),
            },
        }
    }
}

// These maps are private to this footprint and never cloned, so each new
// key allocates exactly the storage owner's unique-node/value entry quote.
fn insert_kind(
    kinds: &mut SharedMap<KindId, ()>,
    kind: KindId,
    budget: Option<&CustomPreparationBudget>,
) -> Option<()> {
    if let Some(budget) = budget {
        if !budget.try_visit(kinds.navigation_work_bound()) {
            return None;
        }
    }
    if kinds.contains_key(&kind) {
        return Some(());
    }
    if let Some(budget) = budget {
        if !budget.try_visit(kinds.navigation_work_bound())
            || !budget.try_claim_bytes(SharedMap::<KindId, ()>::entry_allocation_bytes())
        {
            return None;
        }
    }
    kinds.insert(kind, ());
    Some(())
}

// Metadata visits, merged intents and bulk endpoint pairs each pay before
// inspection. Ordered kind storage additionally pays its owner navigation bound.
const CANDIDATE_VISIT_WORK: usize = 1;

fn pay_visit(budget: Option<&CustomPreparationBudget>) -> Option<()> {
    budget
        .is_none_or(|budget| budget.try_visit(CANDIDATE_VISIT_WORK))
        .then_some(())
}

#[cfg(test)]
#[path = "custom_applicability_tests.rs"]
mod tests;
