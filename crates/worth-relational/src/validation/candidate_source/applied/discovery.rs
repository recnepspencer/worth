//! An actual completed closure can cross only within its one live Source stage.
use super::*;
use crate::authority::mutation::AdjacencyDelta;
use crate::validation::candidate_source::scope_closure::PreparedManagedIndexScopeClosure;
use crate::validation::candidate_source::ManagedIndexSourceProgram;
use crate::validation::custom_rule::CustomPreparationBudget;

pub(in crate::validation::candidate_source) struct DiscoveredSource<'a> {
    context: &'a SourcePreparationContext<'a>,
    group: &'a SourcePreparationGroup,
    program: &'a ManagedIndexSourceProgram,
    closure: PreparedManagedIndexScopeClosure,
    work: CustomInvariantWorkMeter,
}
impl<'a> DiscoveredSource<'a> {
    pub(in crate::validation::candidate_source) fn capture(
        context: &'a SourcePreparationContext<'a>,
        group: &'a SourcePreparationGroup,
        program: &'a ManagedIndexSourceProgram,
        records: &[RecordRef],
        adjacency: &[AdjacencyDelta],
        closure: PreparedManagedIndexScopeClosure,
        work: CustomInvariantWorkMeter,
        budget: &CustomPreparationBudget,
    ) -> Result<Option<Self>, Denial> {
        let mut physical_count = 0usize;
        for _ in program.physical_sources() {
            admit(budget, 1)?;
            physical_count = physical_count.checked_add(1).ok_or_else(exhausted)?;
        }
        let record_width = physical_count
            .checked_mul(2 * size_of::<crate::identity::data::KindId>())
            .and_then(|n| {
                n.checked_add(
                    2 * size_of::<crate::identity::data::KindId>() + size_of::<RecordRef>(),
                )
            })
            .and_then(|n| n.checked_mul(2))
            .ok_or_else(exhausted)?;
        let units = context
            .records
            .len()
            .checked_mul(record_width)
            .and_then(|n| {
                adjacency
                    .len()
                    .checked_mul(size_of::<AdjacencyDelta>())
                    .and_then(|m| n.checked_add(m))
            })
            .ok_or_else(exhausted)?;
        admit(budget, units)?;
        let before = InvariantStateView::new(&context.selected, context.selected.version_id());
        let overlay = OverlayStateView::new(&context.selected, context.draft.state());
        let after = InvariantStateView::new(&overlay, context.proposal.proposed_version_id());
        let physical = context
            .records
            .iter()
            .filter(|record| source_record_is_relevant(program, record, &before, &after));
        let same_records = physical.eq(records.iter());
        let same_adjacency = adjacency == context.draft.outcome.effect.adjacency.deltas;
        let same = same_records && same_adjacency;
        context
            .execution
            .check_live()
            .map_err(|cause| DerivedRelationReadDenial::Request(cause.into()))?;
        if !same {
            return Ok(None);
        }
        if !work.try_claim_bytes(size_of::<Self>() as u64) {
            return Err(DerivedRelationReadDenial::Definition(
                crate::derived_relation::DerivedRelationDenial::WorkBudgetExceeded,
            )
            .into());
        }
        Ok(Some(Self {
            context,
            group,
            program,
            closure,
            work,
        }))
    }
    pub(super) fn take(
        discoveries: &mut Vec<Self>,
        context: &SourcePreparationContext<'_>,
        group: &SourcePreparationGroup,
        program: &ManagedIndexSourceProgram,
        budget: &CustomPreparationBudget,
    ) -> Result<Option<(PreparedManagedIndexScopeClosure, CustomInvariantWorkMeter)>, Denial> {
        admit(
            budget,
            discoveries.len().checked_mul(3).ok_or_else(exhausted)?,
        )?;
        let index = discoveries.iter().position(|entry| {
            std::ptr::eq(entry.context, context)
                && std::ptr::eq(entry.group, group)
                && std::ptr::eq(entry.program, program)
        });
        admit(budget, 0)?;
        let Some(index) = index else {
            return Ok(None);
        };
        let entry = discoveries.swap_remove(index);
        Ok(Some((entry.closure, entry.work)))
    }
}
fn admit(budget: &CustomPreparationBudget, units: usize) -> Result<(), Denial> {
    budget.try_visit(units).then_some(()).ok_or_else(exhausted)
}
fn exhausted() -> Denial {
    DerivedRelationReadDenial::Definition(
        crate::derived_relation::DerivedRelationDenial::WorkBudgetExceeded,
    )
    .into()
}
