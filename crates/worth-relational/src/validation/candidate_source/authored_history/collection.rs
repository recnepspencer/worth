//! Authored lifecycle and endpoint incidence, including equal-value endpoint writes.
use super::{admission, AuthoredSourceIncidence};
use crate::authority::commit::phases::applied_mutation::AppliedMutationDraft;
use crate::authority::mutation::{AdjacencyDelta, AdjacencyDeltaKind};
use crate::derived_relation::DerivedRelationReadDenial;
use crate::identity::data::{KindId, RelationId};
use crate::transactions::data::{
    EntityMutationIntent as Entity, MergedCommitPlan, MutationIntent, RecordRef,
    RelationMutationIntent as Relation,
};
use crate::validation::candidate_source::ManagedIndexSourcePreparationDenial as Denial;
use crate::validation::custom_rule::CustomPreparationBudget;
use crate::validation::engine::state_view::InvariantStateView;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};

pub(super) struct AuthoredMembership {
    pub(super) records: ExecutionOrderedMap<RecordRef, ()>,
    pub(super) incidence: ExecutionOrderedMap<RelationId, AuthoredSourceIncidence>,
}
impl AuthoredMembership {
    pub(super) fn collect(
        execution: ExecutionRequest<'_, '_>,
        budget: &CustomPreparationBudget,
        draft: &AppliedMutationDraft,
        plan: &MergedCommitPlan,
        before: &InvariantStateView<'_>,
        after: &InvariantStateView<'_>,
    ) -> Result<Option<Self>, Denial> {
        let mut result = Self {
            records: ExecutionOrderedMap::default(),
            incidence: ExecutionOrderedMap::default(),
        };
        for allocation in draft.identities.records.canonical() {
            admission::work(budget, 1)?;
            match allocation.record() {
                record @ RecordRef::Entity(_) => result.record(execution, budget, record)?,
                RecordRef::Relation(id) => {
                    result.relation(execution, budget, *id, before, after)?
                }
            }
        }
        for intent in &plan.merged_intents {
            admission::work(budget, 1)?;
            match intent {
                MutationIntent::Entity(Entity::Replace(value)) => {
                    result.record(execution, budget, &RecordRef::Entity(value.entity_id))?
                }
                MutationIntent::Entity(Entity::Delete(value)) => {
                    result.record(execution, budget, &RecordRef::Entity(value.entity_id))?
                }
                MutationIntent::Relation(Relation::UpdateEndpoints(value)) => {
                    result.relation(execution, budget, value.relation_id, before, after)?
                }
                MutationIntent::Relation(Relation::Delete(value)) => {
                    result.relation(execution, budget, value.relation_id, before, after)?
                }
                // No first-slice materialization/history correspondence is claimed.
                MutationIntent::Materialization(_) => return Ok(None),
                MutationIntent::Create(_)
                | MutationIntent::Entity(_)
                | MutationIntent::Relation(_) => {}
            }
        }
        Ok(Some(result))
    }
    fn record(
        &mut self,
        execution: ExecutionRequest<'_, '_>,
        budget: &CustomPreparationBudget,
        record: &RecordRef,
    ) -> Result<(), Denial> {
        admission::ordered(budget, self.records.len(), size_of::<RecordRef>())?;
        self.records
            .insert_cloned_native_for_request(record, &(), execution)
            .map_err(admission::map)
    }
    fn relation(
        &mut self,
        execution: ExecutionRequest<'_, '_>,
        budget: &CustomPreparationBudget,
        id: RelationId,
        before: &InvariantStateView<'_>,
        after: &InvariantStateView<'_>,
    ) -> Result<(), Denial> {
        admission::work(budget, 2)?;
        let old = before.relation_metadata(id);
        let new = after.relation_metadata(id);
        let kind = old
            .as_ref()
            .or(new.as_ref())
            .ok_or(Denial::ForeignSource)?
            .kind_id;
        if old.as_ref().is_some_and(|value| value.kind_id != kind)
            || new.as_ref().is_some_and(|value| value.kind_id != kind)
        {
            return Err(Denial::ForeignSource);
        }
        let incidence = AuthoredSourceIncidence {
            relation: id,
            kind,
            before: old.map(|value| (value.source, value.target)),
            after: new.map(|value| (value.source, value.target)),
        };
        admission::ordered(
            budget,
            self.incidence.len(),
            size_of::<AuthoredSourceIncidence>(),
        )?;
        self.incidence
            .insert_cloned_native_for_request(&id, &incidence, execution)
            .map_err(admission::map)
    }
    pub(super) fn deltas(
        &self,
        budget: &CustomPreparationBudget,
    ) -> Result<Vec<AdjacencyDelta>, Denial> {
        let count = self.incidence.len().checked_mul(2).ok_or_else(|| {
            admission::storage(
                crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow,
            )
        })?;
        // Scratch-only authored incidence interpreted by the existing pullback;
        // these are never installed into MutationEffect or canonical semantics.
        if !budget.try_plan_item(
            (count as u64)
                .checked_mul(size_of::<AdjacencyDelta>() as u64)
                .ok_or_else(|| {
                    admission::storage(
                        crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow,
                    )
                })?,
        ) {
            return Err(DerivedRelationReadDenial::Definition(
                crate::derived_relation::DerivedRelationDenial::WorkBudgetExceeded,
            )
            .into());
        }
        let mut deltas = Vec::new();
        deltas.try_reserve_exact(count).map_err(|_| {
            admission::storage(crate::execution::RelationalExecutionDenialCause::AllocationFailed)
        })?;
        for (_, value) in self.incidence.iter() {
            admission::work(budget, 2)?;
            if let Some((source, target)) = value.before {
                deltas.push(AdjacencyDelta {
                    relation_id: value.relation,
                    kind_id: value.kind,
                    kind: AdjacencyDeltaKind::Deleted { source, target },
                });
            }
            if let Some((source, target)) = value.after {
                deltas.push(AdjacencyDelta {
                    relation_id: value.relation,
                    kind_id: value.kind,
                    kind: AdjacencyDeltaKind::Created { source, target },
                });
            }
        }
        Ok(deltas)
    }
    pub(super) fn entity_kinds<'a>(
        &'a self,
        before: &'a InvariantStateView<'a>,
        after: &'a InvariantStateView<'a>,
    ) -> impl Iterator<Item = KindId> + 'a {
        self.records.iter().flat_map(move |(record, _)| {
            let RecordRef::Entity(id) = record else {
                return [None, None].into_iter().flatten();
            };
            [
                before.entity_metadata(*id).map(|v| v.kind_id),
                after.entity_metadata(*id).map(|v| v.kind_id),
            ]
            .into_iter()
            .flatten()
        })
    }
    pub(super) fn relation_kinds(&self) -> impl Iterator<Item = KindId> + '_ {
        self.incidence.iter().map(|(_, value)| value.kind)
    }
}
