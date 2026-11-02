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
    content: worth_foundational::facade::ProjectedCandidateContentV1,
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
        let mut admit = |work| control.checkpoint(work);
        let overflow = || control.refuse(Stop::CapacityOverflow);
        let mut writer = crate::domain_computation::primary_graph::application_query::projection::ContentWriter::new(&mut admit, &overflow);
        writer.bytes(worth_foundational::facade::ProjectedCandidateContentV1::DOMAIN)?;
        writer.number(entries.len() as u64)?;
        for (entity, influence) in entries.iter() {
            writer.entity(*entity)?;
            writer.number(u64::from(influence.accepted))?;
            writer.bytes(&influence.body.content(control)?)?;
        }
        let content = worth_foundational::facade::ProjectedCandidateContentV1::new(writer.finish());
        control.poll()?;
        Ok(Self {
            owner: rows.owner(),
            entries,
            content,
        })
    }
    pub(super) fn content(&self) -> worth_foundational::facade::ProjectedCandidateContentV1 {
        self.content
    }
    pub(super) fn same_captured(
        &self,
        other: &Self,
        whole_source: bool,
        checkpoint: &mut dyn FnMut(u64) -> bool,
        scratch: &mut crate::domain_computation::primary_graph::application_query::ContentScratch<
            '_,
        >,
    ) -> bool {
        if self.owner != other.owner
            || self.content != other.content
            || self.entries.len() != other.entries.len()
        {
            return false;
        }
        for ((left_key, left), (right_key, right)) in self.entries.iter().zip(other.entries.iter())
        {
            if !checkpoint(32)
                || left_key != right_key
                || left.accepted != right.accepted
                || (!whole_source
                    && !Arc::ptr_eq(&left.body, &right.body)
                    && !left
                        .body
                        .same_content(
                            &right.body,
                            &mut |work| {
                                if checkpoint(work) {
                                    Ok(())
                                } else {
                                    Err(())
                                }
                            },
                            scratch.reserve,
                            &mut |cause| {
                                (scratch.fault)(cause);
                                ()
                            },
                        )
                        .unwrap_or(false))
            {
                return false;
            }
        }
        checkpoint(0)
    }
    pub(super) fn same_membership(
        &self,
        other: &Self,
        checkpoint: &mut dyn FnMut(u64) -> bool,
    ) -> bool {
        if self.owner != other.owner || self.entries.len() != other.entries.len() {
            return false;
        }
        for ((left_key, left), (right_key, right)) in self.entries.iter().zip(other.entries.iter())
        {
            if !checkpoint(32) || left_key != right_key || left.accepted != right.accepted {
                return false;
            }
        }
        checkpoint(0)
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
            if !Arc::ptr_eq(&original.body, current) {
                if original.body.content(control)? != current.content(control)?
                    || !original
                        .body
                        .same_content(current, &mut |work| control.checkpoint(work),
                            &mut |bytes| control.reserve_content_memory(bytes),
                            &mut |cause| {
                                use crate::domain_computation::primary_graph::application_query::ProjectionContentStorageFault as Fault;
                                control.refuse(match cause {
                                    Fault::CapacityOverflow => Stop::CapacityOverflow,
                                    Fault::AllocationFailed => Stop::MaterialStorage(worth_relational::facade::transactions::RelationalExecutionDenialCause::AllocationFailed),
                                })
                            })?
                {
                    return Ok(false);
                }
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
