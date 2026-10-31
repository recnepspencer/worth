//! Actual publication changes join authored commands, including owner-induced edits.
use super::*;
use crate::publication::patch::data::RecordStructuralChange as Change;
impl AuthoredSourceInputs {
    pub(super) fn physical(
        &mut self,
        execution: ExecutionRequest<'_, '_>,
        budget: &CustomPreparationBudget,
        draft: &AppliedMutationDraft,
        before: &InvariantStateView<'_>,
        after: &InvariantStateView<'_>,
    ) -> Result<bool, Denial> {
        // effect_publication emits one canonical delta for each changed record
        // fragment. Use that actual structural classification, not a command guess.
        for delta in &draft.outcome.effect.publication.canonical_deltas {
            admission::work(budget, 1)?;
            let usage = match delta.structural_change {
                Change::Updated => Use::FIELDS,
                Change::Created | Change::Deleted | Change::RetainedForAudit => Use::LIFECYCLE,
                Change::MaterializationSuspended | Change::Rematerialized => return Ok(false),
            };
            match delta.target {
                RecordRef::Entity(_) => self.record(execution, budget, &delta.target, usage)?,
                // Audit retention has no adjacency delta. Its before/after
                // incidence must still enter the existing complete pullback.
                RecordRef::Relation(id) => self.relation(execution, budget, id, before, after)?,
            }
        }
        for delta in &draft.outcome.effect.adjacency.deltas {
            admission::work(budget, 1)?;
            self.relation(execution, budget, delta.relation_id, before, after)?;
        }
        Ok(true)
    }
}
