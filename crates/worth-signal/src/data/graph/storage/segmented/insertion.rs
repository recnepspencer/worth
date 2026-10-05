//! Select a segment handle before publishing an insertion.
use super::{
    checked_segment_component, hash_slice, FlatSegments, Segment, SegmentedStorage, SegmentedStore,
    SetHandle,
};
use crate::data::error::SignalError;
use crate::data::request_preparation::SignalPreparationBudget;
use crate::data::retained_storage::{
    arc_allocation_charge, RetainedStorageBacking, RetainedStorageCharge as Charge,
    RetainedStorageMeasurement, RetainedStoragePreparation as Work,
};
use std::hash::Hash;

mod capacity_bound;

/// Ordered insertions selected against one unchanged segment store. The batch
/// carries owned values so every handle is fixed before any graph publication.
pub(crate) struct PreparedSegmentBatchInsertion<T: Clone, Id: Clone> {
    base_count: usize,
    ids: Vec<Id>,
    store: Option<SegmentedStore<T, Id>>,
}

impl<T: Clone, Id: Clone> PreparedSegmentBatchInsertion<T, Id> {
    pub(crate) fn ids(&self) -> &[Id] {
        &self.ids
    }
}

impl<
        T: Clone + Hash + PartialEq + RetainedStorageMeasurement,
        Id: SetHandle + RetainedStorageMeasurement,
    > SegmentedStore<T, Id>
{
    /// Make a cold restored interner ready before checked callbacks. This is
    /// representation preparation: it changes no segment or public handle.
    pub(crate) fn ensure_interner_ready(
        &mut self,
        work: &mut Work,
        mut budget: Option<&mut SignalPreparationBudget>,
    ) -> Result<(), SignalError> {
        if !self.retained_interner_requires_reconstruction() {
            return Ok(());
        }
        let count = self.live_segment_count();
        let growth = fork_growth_charge::<T, Id>()?;
        work.reserve_visits(
            usize::try_from(growth.bytes())
                .map_err(|_| SignalError::invalid_input("segment promotion growth overflow"))?,
        )
        .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
        if let Some(budget) = budget.as_deref_mut() {
            budget.claim(growth.bytes())?;
        }
        let mut draft = self.fork_persistent();
        for index in 1..=count {
            work.reserve_visits(draft.lookup_steps())
                .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
            let segment = draft.get(Id::from_index(index));
            let bytes = segment_work(segment, work)?;
            work.reserve_visits(bytes)
                .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
            let hash = hash_slice(segment);
            let insertion = draft
                .interner
                .insertion_structure_growth_bound(&hash, work)
                .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
            draft
                .interner
                .reserve_lookup_work(&hash, work)
                .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
            let bucket = draft.interner.get(&hash).map_or(0, Vec::len);
            if let Some(budget) = budget.as_deref_mut() {
                budget.claim(insertion.bytes())?;
                budget.claim_vec::<Id>(bucket.saturating_add(1).saturating_mul(2).max(4))?;
            }
            draft
                .interner
                .reserve_entry_or_default_work(&hash, work)
                .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
            draft
                .interner
                .entry(hash)
                .or_default()
                .push(Id::from_index(index));
        }
        draft
            .interner
            .prepare_retained_charge(work)
            .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
        *self = draft;
        Ok(())
    }

    pub(crate) fn prepare_batch_insertion(
        &mut self,
        batches: &[Vec<T>],
        work: &mut Work,
        mut budget: Option<&mut SignalPreparationBudget>,
    ) -> Result<PreparedSegmentBatchInsertion<T, Id>, SignalError> {
        let base_count = self.live_segment_count();
        if let Some(budget) = budget.as_deref_mut() {
            budget.claim_vec::<Id>(batches.len())?;
        }
        work.reserve_visits(batches.len())
            .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
        if batches.iter().all(Vec::is_empty) {
            return Ok(PreparedSegmentBatchInsertion {
                base_count,
                ids: vec![Id::EMPTY; batches.len()],
                store: None,
            });
        }
        self.ensure_interner_ready(work, budget.as_deref_mut())?;
        // Forking moves Exclusive payloads into backing Arcs. The collection
        // owners supply complete empty overlay charges, including their roots,
        // hasher, and collision bookkeeping; existing payload is not copied.
        let growth = fork_growth_charge::<T, Id>()?;
        work.reserve_visits(
            usize::try_from(growth.bytes())
                .map_err(|_| SignalError::invalid_input("segment fork growth overflow"))?,
        )
        .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
        if let Some(budget) = budget.as_deref_mut() {
            budget.claim(growth.bytes())?;
        }
        // Promotion moves an Exclusive flat base into shared custody. A failed
        // draft leaves that representation ready for later epochs without
        // changing any handle or published segment contents.
        let mut draft = self.fork_persistent();
        let mut ids = Vec::with_capacity(batches.len());
        for values in batches {
            if values.is_empty() {
                ids.push(Id::EMPTY);
                continue;
            }
            checked_segment_component(values.len(), "segment length");
            // Hashing and candidate comparisons are complete before the draft
            // mutates. The installed interner confines comparison to this key's
            // bucket, so steady epochs never traverse unrelated segments.
            let bytes = segment_work(values, work)?;
            work.reserve_visits(bytes)
                .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
            let hash = hash_slice(values);
            draft
                .interner
                .reserve_lookup_work(&hash, work)
                .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
            let mut found = None;
            if let Some(candidates) = draft.interner.get(&hash) {
                for &candidate in candidates {
                    work.reserve_visits(draft.lookup_steps())
                        .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
                    let prior = draft.get(candidate);
                    let prior_bytes = segment_work(prior, work)?;
                    work.reserve_visits(bytes.saturating_add(prior_bytes))
                        .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
                    if prior == values {
                        found = Some(candidate);
                        break;
                    }
                }
            }
            if let Some(id) = found {
                ids.push(id);
                continue;
            }
            let next = draft
                .live_segment_count()
                .checked_add(1)
                .ok_or_else(|| SignalError::invalid_input("segment handle overflow"))?;
            checked_segment_component(next, "segment handle");
            let growth = draft
                .interner
                .insertion_structure_growth_bound(&hash, work)
                .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
            draft
                .interner
                .reserve_lookup_work(&hash, work)
                .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
            let bucket = draft.interner.get(&hash).map_or(0, Vec::len);
            if let Some(budget) = budget.as_deref_mut() {
                budget.claim_vec::<T>(values.len())?;
                budget.claim(
                    u64::try_from(
                        bytes.saturating_sub(values.len().saturating_mul(std::mem::size_of::<T>())),
                    )
                    .map_err(|_| SignalError::invalid_input("segment payload overflow"))?,
                )?;
                claim_appended_page::<T>(budget, draft.live_segment_count())?;
                budget.claim(growth.bytes())?;
                budget.claim_vec::<Id>(bucket.saturating_add(1).saturating_mul(2).max(4))?;
            }
            work.reserve_visits(bytes.saturating_add(bucket).saturating_add(7))
                .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
            draft
                .interner
                .reserve_entry_or_default_work(&hash, work)
                .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
            let id = Id::from_index(next);
            let SegmentedStorage::ForkShared { appended, .. } = &mut draft.storage else {
                unreachable!("batch draft must share the original segment base")
            };
            appended.push_back(values.to_vec());
            draft.interner.entry(hash).or_default().push(id);
            ids.push(id);
        }
        Ok(PreparedSegmentBatchInsertion {
            base_count,
            ids,
            store: Some(draft),
        })
    }

    pub(crate) fn publish_batch_insertion(
        &mut self,
        prepared: PreparedSegmentBatchInsertion<T, Id>,
    ) {
        assert_eq!(
            self.live_segment_count(),
            prepared.base_count,
            "prepared segment batch drifted"
        );
        if let Some(store) = prepared.store {
            *self = store;
        }
    }
}

fn claim_appended_page<T: Clone>(
    budget: &mut SignalPreparationBudget,
    segment_count: usize,
) -> Result<(), SignalError> {
    budget.claim(appended_page_charge::<T>(segment_count)?.bytes())
}

fn fork_growth_charge<T: Clone, Id: Clone>() -> Result<Charge, SignalError> {
    arc_allocation_charge::<RetainedStorageBacking<FlatSegments<T>>>()
        .and_then(|charge| charge.checked_add(
            crate::data::persistent_hash_map::PersistentHashMap::<u64, Vec<Id>>::empty_persistent_overlay_charge()?,
        ))
        .and_then(|charge| charge.checked_add(
            crate::data::persistent_vector::PersistentVector::<Vec<T>>::empty_persistent_overlay_charge()?,
        ))
        .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)
}

fn appended_page_charge<T: Clone>(segment_count: usize) -> Result<Charge, SignalError> {
    // The persistent vector's selected 32-cell page can detach. Its header
    // contains a base length and two Vecs; one Arc<Vec<T>> owns the new segment.
    type Page<T> = (
        usize,
        Vec<(usize, std::sync::Arc<Vec<T>>)>,
        Vec<std::sync::Arc<Vec<T>>>,
    );
    let page = arc_allocation_charge::<Page<T>>()
        .and_then(|charge| charge.checked_add(arc_allocation_charge::<Vec<T>>()?))
        .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
    let page_levels =
        (usize::BITS as usize - (segment_count / 32 + 1).leading_zeros() as usize).max(1);
    page.checked_add(
        Charge::capacity::<std::sync::Arc<Vec<T>>>(32)
            .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?,
    )
    .and_then(|charge| {
        charge.checked_add(
            crate::data::retained_storage::ordered_index_charge::<usize, std::sync::Arc<()>>(1)?
                .checked_mul(page_levels)?,
        )
    })
    .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)
}

fn segment_work<T: RetainedStorageMeasurement>(
    values: &[T],
    work: &mut Work,
) -> Result<usize, SignalError> {
    let mut bytes = values
        .len()
        .checked_mul(std::mem::size_of::<T>())
        .ok_or_else(|| SignalError::invalid_input("segment work overflow"))?;
    for value in values {
        let payload = value
            .retained_heap_charge(work)
            .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
        bytes = bytes
            .checked_add(
                usize::try_from(payload.bytes())
                    .map_err(|_| SignalError::invalid_input("segment payload overflow"))?,
            )
            .ok_or_else(|| SignalError::invalid_input("segment work overflow"))?;
    }
    Ok(bytes)
}

pub(crate) struct PreparedSegmentInsertion<'store, 'items, T: Clone, Id: Clone> {
    store: &'store mut SegmentedStore<T, Id>,
    items: &'items [T],
    id: Id,
    new_hash: Option<u64>,
}

impl<T: Clone + Hash + PartialEq, Id: SetHandle> SegmentedStore<T, Id> {
    /// Keeps the store exclusively borrowed so the selected handle cannot drift.
    /// Publication owns backing writes. Existing cold callers may rebuild the
    /// interner here; retained callers must first require prepared indexes.
    pub(crate) fn prepare_insertion<'store, 'items>(
        &'store mut self,
        items: &'items [T],
    ) -> PreparedSegmentInsertion<'store, 'items, T, Id> {
        if items.is_empty() {
            return PreparedSegmentInsertion {
                store: self,
                items,
                id: Id::EMPTY,
                new_hash: None,
            };
        }
        self.rebuild_interner_if_needed();
        let hash = hash_slice(items);
        if let Some(candidates) = self.interner.get(&hash) {
            for &candidate in candidates {
                if self.get(candidate) == items {
                    return PreparedSegmentInsertion {
                        store: self,
                        items,
                        id: candidate,
                        new_hash: None,
                    };
                }
            }
        }
        checked_segment_component(items.len(), "segment length");
        if let SegmentedStorage::Exclusive(flat) = &self.storage {
            checked_segment_component(flat.items.len(), "segment start");
        }
        let next = self
            .live_segment_count()
            .checked_add(1)
            .expect("segment handle overflow");
        checked_segment_component(next, "segment handle");
        PreparedSegmentInsertion {
            id: Id::from_index(next),
            store: self,
            items,
            new_hash: Some(hash),
        }
    }
}

impl<T: Clone + Hash + PartialEq, Id: SetHandle> PreparedSegmentInsertion<'_, '_, T, Id> {
    pub(crate) fn id(&self) -> Id {
        self.id
    }

    pub(crate) fn publish(self) -> Id {
        if let Some(hash) = self.new_hash {
            match &mut self.store.storage {
                SegmentedStorage::Exclusive(flat) => {
                    let start = checked_segment_component(flat.items.len(), "segment start");
                    flat.items.extend_from_slice(self.items);
                    flat.segments.push(Segment {
                        start,
                        len: checked_segment_component(self.items.len(), "segment length"),
                    });
                }
                SegmentedStorage::ForkShared { appended, .. } => {
                    appended.push_back(self.items.to_vec())
                }
            }
            self.store.interner.entry(hash).or_default().push(self.id);
        }
        self.id
    }
}
