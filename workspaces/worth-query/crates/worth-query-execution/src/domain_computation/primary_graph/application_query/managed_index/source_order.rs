//! Provider ordinals are canonical full-GEN roots, independent of Source ORDER BY.
use super::{
    invocation::controlled, WorthQueryManagedIndexControl as Control,
    WorthQueryManagedIndexStop as Stop, WorthQueryManagedIndexWorkDenial as Denial,
};
use crate::domain_computation::primary_graph::application_query::{
    projection::WorthQueryApplicationDisclosedProjectionTree as Tree,
    read_execution::fallible_heap::{self, Heap},
};
use std::{cell::Cell, cmp::Ordering};
use worth_execution::{ExecutionArray, ExecutionRequest, ExecutionRequestArrayBuilder};

struct Slots<'a> {
    rows: &'a Tree,
    ordinals: ExecutionArray<Cell<usize>>,
    control: &'a Control<'a>,
}
impl Heap for Slots<'_> {
    type Error = Stop;
    fn len(&self) -> usize {
        self.ordinals.len()
    }
    fn compare(&mut self, left: usize, right: usize) -> Result<Ordering, Stop> {
        self.control.checkpoint(2)?;
        let left = self.ordinals[left].get();
        let right = self.ordinals[right].get();
        Ok(self.rows.raw_rows()[left]
            .entity_id()
            .cmp(&self.rows.raw_rows()[right].entity_id())
            // Do not silently deduplicate or alter a genuine Source contract.
            // An equal root preserves its original Source occurrence order.
            .then_with(|| left.cmp(&right)))
    }
    fn swap(&mut self, left: usize, right: usize) -> Result<(), Stop> {
        self.control.checkpoint(2)?;
        let old = self.ordinals[left].get();
        self.ordinals[left].set(self.ordinals[right].get());
        self.ordinals[right].set(old);
        Ok(())
    }
}

pub(super) fn canonical(
    execution: &ExecutionRequest<'_, '_>,
    tree: &Tree,
) -> Result<ExecutionArray<usize>, Denial> {
    controlled(execution, |control| {
        let count = tree.raw_rows().len();
        let mut scratch = ExecutionRequestArrayBuilder::allocate(*execution, count)
            .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
        for ordinal in 0..count {
            control.checkpoint(1)?;
            scratch
                .push(Cell::new(ordinal))
                .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
        }
        let mut slots = Slots {
            rows: tree,
            ordinals: scratch
                .seal()
                .map_err(|cause| control.refuse(Stop::Storage(cause)))?,
            control,
        };
        fallible_heap::sort(&mut slots)?;
        // Scratch and retained backing coexist under separate genuine SDK
        // reservations until the final initialized array is sealed.
        let mut retained = ExecutionRequestArrayBuilder::allocate(*execution, count)
            .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
        for ordinal in 0..count {
            control.checkpoint(1)?;
            retained
                .push(slots.ordinals[ordinal].get())
                .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
        }
        let retained = retained
            .seal()
            .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
        control.poll()?;
        Ok(retained)
    })
}
