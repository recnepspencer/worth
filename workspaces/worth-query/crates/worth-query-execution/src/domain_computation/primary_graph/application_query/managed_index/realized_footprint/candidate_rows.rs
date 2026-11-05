//! All actual candidate bodies remain influence, including exact rejected rows.
use {super::super::source::occurrences::PersistentSourceRows, super::super::source::projection::Entry, crate::domain_computation::vocabulary::primary_graph::control::WorthQueryManagedIndexControl as Control, crate::domain_computation::primary_graph::vocabulary::application_query::source_rows::WorthQueryManagedIndexSourceSlot as Slot, crate::domain_computation::vocabulary::primary_graph::stop::WorthQueryManagedIndexStop as Stop, crate::domain_computation::vocabulary::primary_graph::invocation::WorthQueryManagedIndexWorkDenial as Denial};
use crate::domain_computation::primary_graph::access_strategy::coverage::ReceivedSelection;
use crate::domain_computation::primary_graph::access_strategy::ordered_storage;
use crate::domain_computation::vocabulary::provider_session::session_identity::WorthQueryGraphWorkSessionIdentity;
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
        scratch: &mut crate::domain_computation::primary_graph::vocabulary::application_query::content_storage::ContentScratch<
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
            if !checkpoint(32) || left_key != right_key || left.accepted != right.accepted {
                return false;
            }
            if !whole_source && !Arc::ptr_eq(&left.body, &right.body) {
                // Every Err has already latched its original cause in the
                // erased receiver; false cannot swallow it into a decision.
                let Ok(true) = left.body.same_content(
                    &right.body,
                    &mut |work| if checkpoint(work) { Ok(()) } else { Err(()) },
                    scratch.reserve,
                    &mut |cause| (scratch.fault)(cause),
                    scratch.frames,
                ) else {
                    return false;
                };
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
        execution: &ExecutionRequest<'_, '_>,
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
            if !Arc::ptr_eq(&original.body, current)
                && (original.body.content(control)? != current.content(control)?
                    || !original
                        .body
                        .same_content(current, &mut |work| control.checkpoint(work),
                            &mut |bytes| control.reserve_content_memory(bytes),
                            &mut |cause| {
                                use crate::domain_computation::primary_graph::vocabulary::application_query::content_comparison::ProjectionContentComparisonFault as Fault;
                                control.refuse(match cause {
                                    Fault::ContentInconsistency => Stop::Content(worth_relational::facade::content::RelationalContentDenial::EncodingInconsistency),
                                    Fault::CapacityOverflow => Stop::CapacityOverflow,
                                    Fault::AllocationFailed => Stop::MaterialStorage(worth_relational::facade::transactions::RelationalExecutionDenialCause::AllocationFailed),
                                })
                            },
                            &mut |left, right| left.same_content(execution, right).map_err(|cause| control.refuse(match cause {
                                worth_relational::facade::content::RelationalContentDenial::Execution(cause) => Stop::MaterialStorage(cause),
                                cause => Stop::Content(cause),
                            })))?)
            {
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
