//! Stable managed-footprint order, controlled at each actual comparison and move.
use super::super::fallible_heap::{self, Heap};
use super::*;
use material_storage::MaterialFootprintStorage as Storage;
use std::{cell::Cell, cmp::Ordering};
use worth_execution::{
    ExecutionAllocationDenialClass as Allocation, ExecutionArray,
    ExecutionPhysicalAllocationDenialKind as Physical, ExecutionRequestArrayBuilder,
    ExecutionRequestArrayDenial as ArrayStop,
};
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
type Denial = WorthQueryApplicationReadExecutionDenial;
fn refused(cause: Cause) -> Denial {
    super::super::super::denial::relational_read_denial(cause, "managed footprint order")
}
fn array_stop(cause: ArrayStop) -> Denial {
    let cause = match cause {
        ArrayStop::Request(cause) => cause.into(),
        ArrayStop::Allocation(cause) => match cause.classify() {
            Allocation::Request(cause) => cause.into(),
            Allocation::Physical(Physical::Layout) => Cause::MemoryOverflow,
            Allocation::Physical(Physical::Allocator) => Cause::AllocationFailed,
            Allocation::Physical(
                Physical::CapacityMismatch
                | Physical::WriteBeyondReserved
                | Physical::IncompleteSeal,
            ) => Cause::ResultCapacityExceeded,
        },
    };
    refused(cause)
}
fn span(
    storage: &Storage<'_, '_, '_>,
    count: usize,
    ordinals: bool,
) -> Result<ExecutionArray<Cell<usize>>, Denial> {
    let mut result =
        ExecutionRequestArrayBuilder::allocate(*storage.execution(), count).map_err(array_stop)?;
    for index in 0..count {
        storage.work(1)?;
        result
            .push(Cell::new(if ordinals { index } else { 0 }))
            .map_err(array_stop)?;
    }
    result.seal().map_err(array_stop)
}
struct Ordinals<'a, 'request, 'authority, T, F> {
    positions: ExecutionArray<Cell<usize>>,
    values: &'a [T],
    storage: &'a Storage<'a, 'request, 'authority>,
    compare: F,
}
impl<T, F: FnMut(&T, &T) -> Result<Ordering, Denial>> Heap for Ordinals<'_, '_, '_, T, F> {
    type Error = Denial;
    fn len(&self) -> usize {
        self.positions.len()
    }
    fn compare(&mut self, left: usize, right: usize) -> Result<Ordering, Denial> {
        let left = self.positions[left].get();
        let right = self.positions[right].get();
        Ok((self.compare)(&self.values[left], &self.values[right])?.then_with(|| left.cmp(&right)))
    }
    fn swap(&mut self, left: usize, right: usize) -> Result<(), Denial> {
        self.storage.work(4)?;
        let value = self.positions[left].get();
        self.positions[left].set(self.positions[right].get());
        self.positions[right].set(value);
        Ok(())
    }
}
fn stable<T>(
    storage: &Storage<'_, '_, '_>,
    values: &mut [T],
    compare: impl FnMut(&T, &T) -> Result<Ordering, Denial>,
) -> Result<(), Denial> {
    if values.len() < 2 {
        return Ok(());
    }
    let mut heap = Ordinals {
        positions: span(storage, values.len(), true)?,
        values,
        storage,
        compare,
    };
    fallible_heap::sort(&mut heap)?;
    let inverse = span(storage, values.len(), false)?;
    for rank in 0..heap.positions.len() {
        storage.work(2)?;
        inverse[heap.positions[rank].get()].set(rank);
    }
    let Ordinals {
        positions, compare, ..
    } = heap;
    drop(compare);
    drop(positions); // End the immutable source borrow before permutation.
    for index in 0..values.len() {
        loop {
            storage.work(1)?;
            let target = inverse[index].get();
            if target == index {
                break;
            }
            storage.work(4)?;
            values.swap(index, target);
            let next = inverse[target].get();
            inverse[target].set(target);
            inverse[index].set(next);
        }
    }
    Ok(())
}
fn dedup<T>(
    storage: &Storage<'_, '_, '_>,
    values: &mut Vec<T>,
    mut equal: impl FnMut(&T, &T) -> Result<bool, Denial>,
) -> Result<(), Denial> {
    let mut kept = 0;
    for index in 0..values.len() {
        storage.work(1)?;
        if kept > 0 && equal(&values[kept - 1], &values[index])? {
            continue;
        }
        if kept != index {
            storage.work(1)?;
            values.swap(kept, index);
        }
        kept += 1;
    }
    values.truncate(kept);
    Ok(())
}
fn field_work(
    storage: &Storage<'_, '_, '_>,
    left: &WorthQueryObservedFieldRevision,
    right: &WorthQueryObservedFieldRevision,
) -> Result<(), Denial> {
    let bytes = [left, right]
        .iter()
        .try_fold(
            2 * size_of::<WorthQueryObservedFieldRevision>(),
            |n, field| {
                n.checked_add(field.entity_name.len())?
                    .checked_add(field.aspect.as_str().len())?
                    .checked_add(field.field.as_str().len())
            },
        )
        .ok_or_else(|| refused(Cause::WorkCounterOverflow))?;
    storage.work(bytes)
}
pub(super) fn normalize(
    storage: &Storage<'_, '_, '_>,
    footprint: &mut WorthQueryObservedSourceFootprint,
) -> Result<(), Denial> {
    // Lengths are O(1); admission precedes every variable traversal below.
    let visits = footprint
        .entities
        .len()
        .checked_add(footprint.aspects.len())
        .and_then(|n| n.checked_add(footprint.adjacencies.len()))
        .ok_or_else(|| refused(Cause::WorkCounterOverflow))?;
    storage.work(visits)?;
    stable(storage, &mut footprint.aspects, |l, r| {
        field_work(storage, l, r)?;
        Ok((l.entity, &l.aspect, &l.field).cmp(&(r.entity, &r.aspect, &r.field)))
    })?;
    dedup(storage, &mut footprint.aspects, |l, r| {
        field_work(storage, l, r)?;
        Ok(l == r)
    })?;
    stable(storage, &mut footprint.entities, |l, r| {
        storage.work(2 * size_of_val(l))?;
        Ok(l.cmp(r))
    })?;
    dedup(storage, &mut footprint.entities, |l, r| {
        storage.work(2 * size_of_val(l))?;
        Ok(l == r)
    })?;
    stable(storage, &mut footprint.adjacencies, |l, r| {
        storage.work(2 * size_of_val(l))?;
        Ok((l.anchor, l.relation_kind, l.direction as u8).cmp(&(
            r.anchor,
            r.relation_kind,
            r.direction as u8,
        )))
    })?;
    dedup(storage, &mut footprint.adjacencies, |l, r| {
        let bytes = l
            .endpoints
            .len()
            .checked_add(r.endpoints.len())
            .and_then(|n| n.checked_mul(size_of::<worth_relational::facade::identity::EntityId>()))
            .and_then(|n| n.checked_add(2 * size_of_val(l)))
            .ok_or_else(|| refused(Cause::WorkCounterOverflow))?;
        storage.work(bytes)?;
        Ok(l == r)
    })
}
