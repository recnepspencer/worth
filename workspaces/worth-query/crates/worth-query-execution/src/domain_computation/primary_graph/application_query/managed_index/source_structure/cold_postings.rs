//! Complete cold postings are sorted once and allocate each persistent cell once.
use super::*;
use crate::domain_computation::primary_graph::application_query::read_execution::fallible_heap::{
    self, Heap,
};
use std::{cell::Cell, cmp::Ordering};

struct Pairs<'a, 'control> {
    values: &'a [Cell<Option<(Slot, usize)>>],
    control: &'a Control<'control>,
}
impl Heap for Pairs<'_, '_> {
    type Error = Stop;
    fn len(&self) -> usize {
        self.values.len()
    }
    fn compare(&mut self, left: usize, right: usize) -> Result<Ordering, Stop> {
        self.control.checkpoint((2 * size_of::<Slot>()) as u64)?;
        Ok(self.values[left]
            .get()
            .expect("captured pair")
            .0
            .cmp(&self.values[right].get().expect("captured pair").0))
    }
    fn swap(&mut self, left: usize, right: usize) -> Result<(), Stop> {
        self.control
            .checkpoint((2 * size_of::<(Slot, usize)>()) as u64)?;
        self.values[left].swap(&self.values[right]);
        Ok(())
    }
}

pub(super) fn capture(
    edits: &[Edit],
    execution: &ExecutionRequest<'_, '_>,
    control: &Control<'_>,
) -> Result<Map<Slot, usize>, Stop> {
    let mut count = 0;
    for edit in edits {
        control.checkpoint(1)?;
        count += usize::from(edit.region.and_then(|region| region.source).is_some());
    }
    let values = super::cold_columns::column(count, execution, control)?;
    let mut next = 0;
    for edit in edits {
        control.checkpoint(1)?;
        if let Some(slot) = edit.region.and_then(|region| region.source) {
            values.elements()[next].set(Some((slot, edit.ordinal)));
            next += 1;
        }
    }
    fallible_heap::sort(&mut Pairs {
        values: values.elements(),
        control,
    })?;
    // One adjacent full-key comparison, payload move and Entry/Node construction
    // per pair. No persistent search path is rebuilt during cold construction.
    let width = 2 * size_of::<Slot>() + size_of::<usize>() + 16 * size_of::<usize>();
    let work = count
        .checked_mul(width)
        .and_then(|work| u64::try_from(work).ok())
        .ok_or_else(|| control.refuse(Stop::CapacityOverflow))?;
    control.checkpoint(work)?;
    let sources = Map::from_sorted_native_for_request(
        count,
        values
            .elements()
            .iter()
            .map(|pair| pair.get().expect("captured pair")),
        *execution,
    )
    .map_err(|cause| control.refuse(cause.into()))?
    .ok_or_else(|| control.refuse(Stop::DuplicateSourceSlot))?;
    control.poll()?;
    Ok(sources)
}
