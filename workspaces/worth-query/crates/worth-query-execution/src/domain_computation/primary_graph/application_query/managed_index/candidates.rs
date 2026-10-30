//! Fixed source-slot membership, without raw entity or mutation authority.
use super::{
    WorthQueryManagedIndexControl, WorthQueryManagedIndexSourceRows,
    WorthQueryManagedIndexSourceSlot, WorthQueryManagedIndexStop as Stop,
};
use crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity;
use std::cell::Cell;
use worth_execution::{ExecutionArray, ExecutionRequest, ExecutionRequestArrayBuilder};

/// Candidate source slots. Final ordinary projection and disclosure remain authoritative.
pub struct WorthQueryManagedIndexCandidateWriter<'a> {
    owner: WorthQueryGraphWorkSessionIdentity,
    marked: ExecutionArray<Cell<bool>>,
    count: usize,
    control: &'a WorthQueryManagedIndexControl<'a>,
}
impl<'a> WorthQueryManagedIndexCandidateWriter<'a> {
    pub(super) fn allocate<S, I>(
        execution: &ExecutionRequest<'_, '_>,
        source: &WorthQueryManagedIndexSourceRows<'_, S, I>,
        control: &'a WorthQueryManagedIndexControl<'a>,
    ) -> Result<Self, Stop>
    where
        S: worth_query_declaration::facade::application_schema::ApplicationSchema,
        I: worth_query_declaration::facade::application_query::ApplicationManagedIndexBinding<S>,
    {
        let mut rows = ExecutionRequestArrayBuilder::allocate(*execution, source.len())
            .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
        for _ in 0..source.len() {
            control.checkpoint(1)?;
            rows.push(Cell::new(false))
                .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
        }
        let marked = rows
            .seal()
            .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
        Ok(Self {
            owner: source.owner(),
            marked,
            count: 0,
            control,
        })
    }
    pub fn emit(&mut self, slot: WorthQueryManagedIndexSourceSlot) -> Result<(), Stop> {
        self.control.checkpoint(1)?;
        if slot.owner != self.owner {
            return Err(self.control.refuse(Stop::ForeignSourceSlot));
        }
        let cell = self
            .marked
            .get(slot.ordinal)
            .ok_or_else(|| self.control.refuse(Stop::SourceSlotUnavailable))?;
        if !cell.replace(true) {
            self.count = self
                .count
                .checked_add(1)
                .ok_or_else(|| self.control.refuse(Stop::CapacityOverflow))?;
        }
        self.control.poll()
    }
    pub(super) fn contains(&self, ordinal: usize) -> Result<bool, Stop> {
        self.control.checkpoint(1)?;
        self.marked
            .get(ordinal)
            .map(Cell::get)
            .ok_or_else(|| self.control.refuse(Stop::SourceSlotUnavailable))
    }
    pub(super) fn finish(
        self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ExecutionArray<WorthQueryManagedIndexSourceSlot>, Stop> {
        let mut selected = ExecutionRequestArrayBuilder::allocate(*execution, self.count)
            .map_err(|cause| self.control.refuse(Stop::Storage(cause)))?;
        for ordinal in 0..self.marked.len() {
            if self.contains(ordinal)? {
                selected
                    .push(WorthQueryManagedIndexSourceSlot {
                        owner: self.owner,
                        ordinal,
                    })
                    .map_err(|cause| self.control.refuse(Stop::Storage(cause)))?;
            }
        }
        let selected = selected
            .seal()
            .map_err(|cause| self.control.refuse(Stop::Storage(cause)))?;
        self.control.poll()?;
        Ok(selected)
    }
    pub fn len(&self) -> usize {
        self.count
    }
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}
