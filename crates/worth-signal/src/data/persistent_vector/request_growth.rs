//! Physical request growth for selected edits to a forked page vector.
//! Retained publication keeps its separate full logical root custody.
use std::sync::Arc;

use super::{ForkPage, PersistentVector, PersistentVectorStorage};
use crate::data::retained_storage::{
    arc_allocation_charge, ordered_edit_growth_charge, ordered_index_charge,
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};

impl<T: Clone + RetainedStorageMeasurement, const PAGE_LEN: usize> PersistentVector<T, PAGE_LEN> {
    /// New allocation retained by the final selected pages plus one in-flight
    /// path/page/cell while a sequential edit replaces its predecessor.
    /// The shared base and unchanged installed pages remain owned by their
    /// existing roots; they are covered by retained custody, not allocated by
    /// this request. `indices` are sorted unique and may include append slots.
    pub(crate) fn selected_batch_request_growth_bound(
        &self,
        indices: &[usize],
        may_stage_push: bool,
        work: &mut Work,
    ) -> Result<Charge, Denial> {
        self.selected_batch_request_growth_bound_with_payload(
            indices,
            may_stage_push,
            work,
            |value, work| value.retained_heap_charge(work),
        )
    }

    /// The vector owns page/index geometry; the caller owns the physical
    /// clone cost of its selected payload type. Shared subfields can price
    /// their new Arc pointers here without discounting retained custody.
    pub(crate) fn selected_batch_request_growth_bound_with_payload(
        &self,
        indices: &[usize],
        may_stage_push: bool,
        work: &mut Work,
        mut payload_growth: impl FnMut(&T, &mut Work) -> Result<Charge, Denial>,
    ) -> Result<Charge, Denial> {
        if indices.is_empty() {
            return Ok(Charge::ZERO);
        }
        // The distinct-page pass itself traverses every selected index.
        work.reserve_visits(indices.len())?;
        let distinct_pages = indices
            .iter()
            .map(|index| index / PAGE_LEN)
            .scan(None, |previous, page| {
                let distinct = *previous != Some(page);
                *previous = Some(page);
                Some(usize::from(distinct))
            })
            .sum::<usize>();
        let installed_pages = match &self.storage {
            PersistentVectorStorage::Exclusive(_) => 0,
            PersistentVectorStorage::ForkShared { changed_pages, .. } => changed_pages.len(),
        };
        let final_pages = installed_pages
            .checked_add(distinct_pages)
            .ok_or(Denial::ChargeOverflow)?;
        let page_capacity = PAGE_LEN
            .max(4)
            .checked_next_power_of_two()
            .ok_or(Denial::ChargeOverflow)?;
        let full_page = Charge::capacity::<(usize, Arc<T>)>(page_capacity)?
            .checked_add(Charge::capacity::<Arc<T>>(page_capacity)?)?;
        let page_arc = arc_allocation_charge::<ForkPage<T>>()?;
        let cell_arc = arc_allocation_charge::<T>()?;
        let path = ordered_edit_growth_charge::<usize, Arc<ForkPage<T>>>(final_pages)?;
        // im 15.1 stores 64 keys per node and rebalances nonroot nodes when
        // below its median of 32. A completed index of n keys has at most
        // ceil(n/31) nonroot nodes plus a root. This bounds the union of all
        // selected paths retained by the final changed-page index. One
        // transient path below covers the edit while its predecessor lives.
        let final_index = final_pages
            .div_ceil(31)
            .checked_add(1)
            .ok_or(Denial::ChargeOverflow)
            .and_then(|nodes| {
                ordered_index_charge::<usize, Arc<ForkPage<T>>>(0)?.checked_mul(nodes)
            });
        let selected_paths = path.checked_mul(distinct_pages);
        let retained_index = match (final_index, selected_paths) {
            (Ok(index), Ok(paths)) => index.min(paths),
            (Ok(index), Err(_)) => index,
            (Err(_), Ok(paths)) => paths,
            (Err(error), Err(_)) => return Err(error),
        };
        let lookup = self.lookup_steps();
        let visits = indices
            .len()
            .checked_mul(
                lookup
                    .checked_mul(2)
                    .and_then(|n| n.checked_add(1))
                    .ok_or(Denial::ChargeOverflow)?,
            )
            .ok_or(Denial::ChargeOverflow)?;
        work.reserve_visits(visits)?;

        let mut final_growth = Charge::ZERO;
        let mut largest_page = full_page;
        let mut largest_payload = Charge::ZERO;
        let mut previous_page = None;
        for &index in indices {
            let page = index / PAGE_LEN;
            if previous_page != Some(page) {
                let installed = match &self.storage {
                    PersistentVectorStorage::Exclusive(_) => Charge::ZERO,
                    PersistentVectorStorage::ForkShared { changed_pages, .. } => changed_pages
                        .get(&page)
                        .map_or(Ok(Charge::ZERO), |page| page.mutation_structure_charge())?,
                };
                let page_shape = full_page.max(installed);
                largest_page = largest_page.max(page_shape);
                final_growth = final_growth
                    .checked_add(page_arc)?
                    .checked_add(page_shape)?;
                previous_page = Some(page);
            }
            let payload = self
                .get(index)
                .map_or(Ok(Charge::ZERO), |value| payload_growth(value, work))?;
            largest_payload = largest_payload.max(payload);
            final_growth = final_growth.checked_add(cell_arc)?.checked_add(payload)?;
        }
        final_growth = final_growth.checked_add(retained_index)?;
        let in_flight = path
            .checked_add(page_arc)?
            .checked_add(largest_page)?
            .checked_add(cell_arc)?
            .checked_add(largest_payload)?;
        let staging = if may_stage_push {
            path.checked_add(page_arc)?
                .checked_add(full_page)?
                .checked_add(cell_arc)?
        } else {
            Charge::ZERO
        };
        final_growth.checked_add(in_flight)?.checked_add(staging)
    }

    /// Ordinary direct replacement mutates an exclusive slot without
    /// cloning a root. Retained staging must use the full helper above.
    pub(crate) fn selected_direct_replacement_request_growth_bound(
        &self,
        indices: &[usize],
        work: &mut Work,
        payload_growth: impl FnMut(&T, &mut Work) -> Result<Charge, Denial>,
    ) -> Result<Charge, Denial> {
        if matches!(self.storage, PersistentVectorStorage::Exclusive(_)) {
            return Ok(Charge::ZERO);
        }
        self.selected_batch_request_growth_bound_with_payload(indices, false, work, payload_growth)
    }
}
