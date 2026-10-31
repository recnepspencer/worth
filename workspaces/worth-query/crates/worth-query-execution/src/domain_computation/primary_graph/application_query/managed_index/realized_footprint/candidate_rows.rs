//! All actual candidate bodies remain influence, including exact rejected rows.
use super::super::{
    source::{occurrences::PersistentSourceRows, projection::Entry},
    WorthQueryManagedIndexControl as Control, WorthQueryManagedIndexSourceSlot as Slot,
    WorthQueryManagedIndexStop as Stop, WorthQueryManagedIndexWorkDenial as Denial,
};
use crate::domain_computation::primary_graph::access_strategy::coverage::ReceivedSelection;
use crate::domain_computation::primary_graph::access_strategy::ordered_storage;
use crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity;
use std::sync::Arc;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};
use worth_relational::facade::identity::EntityId;

pub(super) struct CapturedCandidateRows {
    owner: WorthQueryGraphWorkSessionIdentity,
    entries: ExecutionOrderedMap<EntityId, RowInfluence>,
}
#[derive(Clone)]
struct RowInfluence {
    body: Arc<Entry>,
    accepted: bool,
}
impl CapturedCandidateRows {
    pub(super) fn capture(
        execution: &ExecutionRequest<'_, '_>,
        rows: &PersistentSourceRows,
        selected: &ReceivedSelection,
        control: &Control<'_>,
    ) -> Result<Self, Denial> {
        let mut entries = ExecutionOrderedMap::default();
        let candidates = selected.candidate_slots();
        for (ordinal, slot) in candidates.iter().enumerate() {
            let entity = entity(rows, *slot, control)?;
            ordered_storage::lookup(control, rows.len(), size_of::<EntityId>())?;
            let body = rows
                .entry(entity)
                .ok_or_else(|| control.refuse(Stop::SourceSlotUnavailable))?;
            let accepted = selected
                .candidate_accepted(ordinal)
                .ok_or_else(|| control.refuse(Stop::IncompleteRefinement))?;
            let influence = RowInfluence {
                body: Arc::clone(body),
                accepted,
            };
            ordered_storage::insert(&mut entries, &entity, &influence, execution, control)?;
        }
        if entries.len() != candidates.len() {
            return Err(control.refuse(Stop::IncompleteCoverage).into());
        }
        Ok(Self {
            owner: rows.owner(),
            entries,
        })
    }
    pub(super) fn matches(
        &self,
        rows: &PersistentSourceRows,
        selected: &ReceivedSelection,
        control: &Control<'_>,
    ) -> Result<bool, Denial> {
        control.checkpoint(1)?;
        let candidates = selected.candidate_slots();
        if self.owner != rows.owner() || candidates.len() != self.entries.len() {
            return Ok(false);
        }
        for (ordinal, slot) in candidates.iter().enumerate() {
            let entity = entity(rows, *slot, control)?;
            ordered_storage::lookup(control, self.entries.len(), size_of::<EntityId>())?;
            let Some(original) = self.entries.get(&entity) else {
                return Ok(false);
            };
            let accepted = selected
                .candidate_accepted(ordinal)
                .ok_or_else(|| control.refuse(Stop::IncompleteRefinement))?;
            if original.accepted != accepted {
                return Ok(false);
            }
            ordered_storage::lookup(control, rows.len(), size_of::<EntityId>())?;
            let current = rows
                .entry(entity)
                .ok_or_else(|| control.refuse(Stop::SourceSlotUnavailable))?;
            // Sharing this immutable occurrence/projection/dependency body is a
            // sufficient proof. Non-sharing is not a claim of unequal values.
            if !Arc::ptr_eq(&original.body, current) {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
fn entity(
    rows: &PersistentSourceRows,
    slot: Slot,
    control: &Control<'_>,
) -> Result<EntityId, Stop> {
    control.checkpoint(1)?;
    if slot.owner != rows.owner() || slot.ordinal != 0 {
        return Err(control.refuse(Stop::ForeignSourceSlot));
    }
    slot.entity
        .ok_or_else(|| control.refuse(Stop::ForeignSourceSlot))
}
