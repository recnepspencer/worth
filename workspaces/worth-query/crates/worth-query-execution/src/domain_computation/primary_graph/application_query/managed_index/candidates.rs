//! Fixed source-slot membership, without raw entity or mutation authority.
use super::{
    WorthQueryManagedIndexControl, WorthQueryManagedIndexSourceRows,
    WorthQueryManagedIndexSourceSlot, WorthQueryManagedIndexStop as Stop,
};
use crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity;
use std::{cell::Cell, sync::Arc};
use worth_execution::{ExecutionArray, ExecutionRequest, ExecutionRequestArrayBuilder};

/// Candidate source slots. Final ordinary projection and disclosure remain authoritative.
pub struct WorthQueryManagedIndexCandidateWriter<'a> {
    owner: WorthQueryGraphWorkSessionIdentity,
    marked: ExecutionArray<Cell<bool>>,
    sparse: Option<SparseCandidates<'a>>,
    count: usize,
    control: &'a WorthQueryManagedIndexControl<'a>,
}
struct SparseCandidates<'a> {
    image: super::source_rows::SourceImageView<'a>,
    selected: im::OrdSet<WorthQueryManagedIndexSourceSlot>,
    allocation: Arc<crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation>,
    execution: ExecutionRequest<'a, 'a>,
}
impl<'a> WorthQueryManagedIndexCandidateWriter<'a> {
    pub(super) fn allocate<S, I>(
        execution: &ExecutionRequest<'a, 'a>,
        source: &WorthQueryManagedIndexSourceRows<'a, S, I>,
        control: &'a WorthQueryManagedIndexControl<'a>,
    ) -> Result<Self, Stop>
    where
        S: worth_query_declaration::facade::application_schema::ApplicationSchema,
        I: worth_query_declaration::facade::application_query::ApplicationManagedIndexBinding<S>,
    {
        let sparse = if matches!(
            source.image().rows,
            super::source_rows::SourceImageRows::Maintained(_)
        ) {
            Some(SparseCandidates {
                image: source.image(),
                selected: im::OrdSet::new(),
                allocation: crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation::open(execution)
                    .map_err(|cause| control.refuse(Stop::MaterialStorage(cause)))?,
                execution: *execution,
            })
        } else {
            None
        };
        let dense_count = if sparse.is_some() { 0 } else { source.len() };
        let mut rows = ExecutionRequestArrayBuilder::allocate(*execution, dense_count)
            .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
        for _ in 0..dense_count {
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
            sparse,
            count: 0,
            control,
        })
    }
    pub fn emit(&mut self, slot: WorthQueryManagedIndexSourceSlot) -> Result<(), Stop> {
        self.control.checkpoint(1)?;
        if slot.owner != self.owner {
            return Err(self.control.refuse(Stop::ForeignSourceSlot));
        }
        if let Some(sparse) = &mut self.sparse {
            let super::source_rows::SourceImageRows::Maintained(rows) = sparse.image.rows else {
                unreachable!("sparse candidates borrow only maintained material");
            };
            if !slot.entity.is_some_and(|entity| rows.contains(entity)) || slot.ordinal != 0 {
                return Err(self.control.refuse(Stop::SourceSlotUnavailable));
            }
            if !sparse.selected.contains(&slot) {
                sparse
                    .allocation
                    .claim_tree::<WorthQueryManagedIndexSourceSlot>(
                        &sparse.execution,
                        sparse.selected.len(),
                    )
                    .map_err(|cause| self.control.refuse(Stop::MaterialStorage(cause)))?;
                sparse.selected.insert(slot);
                self.count = self
                    .count
                    .checked_add(1)
                    .ok_or_else(|| self.control.refuse(Stop::CapacityOverflow))?;
            }
            return self.control.poll();
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
        if let Some(sparse) = &self.sparse {
            for slot in sparse.selected.iter().copied() {
                self.control.checkpoint(1)?;
                selected
                    .push(slot)
                    .map_err(|cause| self.control.refuse(Stop::Storage(cause)))?;
            }
            return selected
                .seal()
                .map_err(|cause| self.control.refuse(Stop::Storage(cause)));
        }
        for ordinal in 0..self.marked.len() {
            if self.contains(ordinal)? {
                selected
                    .push(WorthQueryManagedIndexSourceSlot {
                        owner: self.owner,
                        ordinal,
                        entity: None,
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
