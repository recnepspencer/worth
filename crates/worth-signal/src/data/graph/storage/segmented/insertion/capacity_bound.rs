//! Owner-local pre-dispatch ceiling for a selected segment insertion batch.
use std::hash::Hash;

use super::super::{SegmentedStore, SetHandle};
use super::{appended_page_charge, fork_growth_charge};
use crate::data::error::SignalError;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparationDenial as Denial,
};

impl<
        T: Clone + Hash + PartialEq + RetainedStorageMeasurement,
        Id: SetHandle + RetainedStorageMeasurement,
    > SegmentedStore<T, Id>
{
    pub(crate) fn insertion_payload_capacity_bound(
        &self,
        values: usize,
        nested_heap: u64,
    ) -> Result<Charge, SignalError> {
        Charge::capacity::<T>(values)
            .and_then(|charge| charge.checked_add(Charge::from_bytes(nested_heap)))
            .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)
    }

    /// One fork plus every prospective page, hash path and collision bucket.
    /// All selected segments may share the same hash or full-hash group.
    pub(crate) fn batch_structure_capacity_bound(
        &self,
        maximum_insertions: usize,
    ) -> Result<Charge, SignalError> {
        if maximum_insertions == 0 {
            return Ok(Charge::ZERO);
        }
        let count = self
            .live_segment_count()
            .checked_add(maximum_insertions)
            .ok_or_else(|| SignalError::invalid_input("segment count overflow"))?;
        let map = self
            .interner
            .worst_insertion_structure_growth_bound(maximum_insertions)
            .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
        let bucket_slots = count
            .checked_mul(2)
            .ok_or(Denial::ChargeOverflow)
            .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
        let bucket = Charge::capacity::<Id>(bucket_slots)
            .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
        let per_segment = appended_page_charge::<T>(count)?
            .checked_add(map)
            .and_then(|charge| charge.checked_add(bucket))
            .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
        fork_growth_charge::<T, Id>()?
            .checked_add(
                per_segment
                    .checked_mul(maximum_insertions)
                    .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?,
            )
            .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)
    }
}
