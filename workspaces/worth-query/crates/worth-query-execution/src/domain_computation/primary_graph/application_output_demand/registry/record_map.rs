//! Demand rows and observation of whole-map traversal, at the map owner.
use super::{DemandRecord, WorthQueryOutputDemandKey};
use std::collections::BTreeMap;
use std::ops::Bound::{Excluded, Included, Unbounded};
type Map = BTreeMap<WorthQueryOutputDemandKey, DemandRecord>;
#[derive(Default)]
pub(super) struct DemandRecords(Map, BTreeMap<
    crate::domain_computation::primary_graph::application_query::observed_source::source_identity::WorthQueryObservedSourceCoordinate,
    std::collections::BTreeSet<WorthQueryOutputDemandKey>>, super::publication_head::PublicationHeads, #[cfg(test)] std::cell::Cell<u64>);
#[cfg(feature = "test-query-execution-observer")]
thread_local! { static WHOLE_ROWS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) }; }
fn observe_row() {
    #[cfg(feature = "test-query-execution-observer")]
    WHOLE_ROWS.with(|count| count.set(count.get().checked_add(1).expect("row visit counter")));
}
#[cfg(feature = "test-query-execution-observer")]
pub(super) fn whole_rows() -> u64 {
    WHOLE_ROWS.with(std::cell::Cell::get)
}
impl DemandRecords {
    #[cfg(test)]
    pub(super) fn occurrence_visits_for_test(&self) -> u64 {
        self.3.get()
    }

    pub(super) fn retain(
        &mut self,
        mut predicate: impl FnMut(&WorthQueryOutputDemandKey, &mut DemandRecord) -> bool,
    ) {
        self.0.retain(|key, row| {
            observe_row();
            let keep = predicate(key, row);
            if !keep {
                self.2.remove(row.product_occurrence);
                let coordinate = key.source.occurrence_coordinate();
                let keys = self.1.get_mut(&coordinate).expect("indexed row exists");
                keys.remove(key);
                if keys.is_empty() {
                    self.1.remove(&coordinate);
                }
            }
            keep
        });
    }
    #[cfg(test)]
    pub(super) fn extend(
        &mut self,
        rows: impl IntoIterator<Item = (WorthQueryOutputDemandKey, DemandRecord)>,
    ) {
        for (key, row) in rows {
            self.insert(key, row);
        }
    }
    pub(super) fn new() -> Self {
        Self::default()
    }
    pub(super) fn iter(
        &self,
    ) -> impl DoubleEndedIterator<Item = (&WorthQueryOutputDemandKey, &DemandRecord)> {
        ObservedRows(self.0.iter())
    }
    pub(super) fn values(&self) -> impl DoubleEndedIterator<Item = &DemandRecord> {
        ObservedRows(self.0.values())
    }
    pub(super) fn values_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut DemandRecord> {
        ObservedRows(self.0.values_mut())
    }
    pub(super) fn get(&self, key: &WorthQueryOutputDemandKey) -> Option<&DemandRecord> {
        self.0.get(key)
    }
    pub(super) fn get_mut(&mut self, key: &WorthQueryOutputDemandKey) -> Option<&mut DemandRecord> {
        self.0.get_mut(key)
    }
    pub(super) fn contains_key(&self, key: &WorthQueryOutputDemandKey) -> bool {
        self.0.contains_key(key)
    }
    pub(super) fn insert(
        &mut self,
        key: WorthQueryOutputDemandKey,
        mut row: DemandRecord,
    ) -> Option<DemandRecord> {
        if let Some(previous) = self.0.get(&key) {
            self.2.remove(previous.product_occurrence);
        }
        let counter = self.2.insert(row.product_occurrence);
        if let super::DemandState::Output(output) = &mut row.state {
            output.bind_publication_head(counter);
        }
        self.1
            .entry(key.source.occurrence_coordinate())
            .or_default()
            .insert(key.clone());
        self.0.insert(key, row)
    }
    pub(super) fn remove(&mut self, key: &WorthQueryOutputDemandKey) -> Option<DemandRecord> {
        let row = self.0.remove(key)?;
        self.2.remove(row.product_occurrence);
        let coordinate = key.source.occurrence_coordinate();
        let keys = self.1.get_mut(&coordinate).expect("indexed row exists");
        keys.remove(key);
        if keys.is_empty() {
            self.1.remove(&coordinate);
        }
        Some(row)
    }
    #[cfg(test)]
    pub(super) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub(super) fn len(&self) -> usize {
        self.0.len()
    }
    /// Two keyed seeks, bounded by the contiguous occurrence on either side.
    /// No caller can choose bounds that include another occurrence.
    pub(super) fn occurrence_rows<'a>(
        &'a self,
        key: &'a WorthQueryOutputDemandKey,
    ) -> impl Iterator<Item = (&'a WorthQueryOutputDemandKey, &'a DemandRecord)> {
        let before = self
            .0
            .range((Unbounded, Excluded(key)))
            .rev()
            .take_while(move |(other, _)| other.same_occurrence(key));
        let from = self
            .0
            .range((Included(key), Unbounded))
            .take_while(move |(other, _)| other.same_occurrence(key));
        let rows = before.chain(from);
        #[cfg(test)]
        let rows = rows.inspect(|_| self.3.set(self.3.get() + 1));
        rows
    }
}
impl std::ops::Index<&WorthQueryOutputDemandKey> for DemandRecords {
    type Output = DemandRecord;
    fn index(&self, key: &WorthQueryOutputDemandKey) -> &DemandRecord {
        &self.0[key]
    }
}
impl DemandRecords {
    /// Keyed source-coordinate lookup includes its producer occurrences only.
    pub(super) fn source_occurrence_rows<'a>(
        &'a self,
        source: &super::SourceEpoch,
    ) -> impl Iterator<Item = (&'a WorthQueryOutputDemandKey, &'a DemandRecord)> {
        self.1
            .get(&source.occurrence_coordinate())
            .into_iter()
            .flat_map(|keys| keys.iter())
            .map(|key| (key, self.0.get(key).expect("source index names a live row")))
    }
    /// One worst-case ordered node for each of the source index's two levels,
    /// funded by each row even when its coordinate is shared with other rows.
    pub(super) fn source_index_bytes(key: &WorthQueryOutputDemandKey) -> Option<usize> {
        let coordinate=std::mem::size_of::<crate::domain_computation::primary_graph::application_query::observed_source::source_identity::WorthQueryObservedSourceCoordinate>();
        let outer = 12
            * (coordinate
                + std::mem::size_of::<std::collections::BTreeSet<WorthQueryOutputDemandKey>>())
            + 14 * std::mem::size_of::<usize>();
        let inner = 12 * std::mem::size_of::<WorthQueryOutputDemandKey>()
            + 14 * std::mem::size_of::<usize>();
        outer
            .checked_add(inner)?
            .checked_add(super::publication_head::PublicationHeads::node_bytes())?
            .checked_add(key.producer.len())
    }
}
impl From<Map> for DemandRecords {
    fn from(map: Map) -> Self {
        let mut records = Self::default();
        for (key, row) in map {
            records.insert(key, row);
        }
        records
    }
}
#[cfg(feature = "test-query-execution-observer")]
mod observation {
    use super::*;
    impl DemandRecords {
        pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn publication_count_for_test(
            &self,
        ) -> usize {
            self.2
                 .0
                .values()
                .map(|head| head.undelivered.load(std::sync::atomic::Ordering::Acquire))
                .sum()
        }
    }
    impl<Schema>
        crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
    {
        #[doc(hidden)]
        pub fn registry_row_count_for_test(&self) -> usize {
            self.output_demands
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .records
                .len()
        }
        #[doc(hidden)]
        pub fn whole_registry_rows_visited_for_test(&self) -> u64 {
            whole_rows()
        }
    }
}

pub(super) struct ObservedRows<I>(I);
impl<I: Iterator> Iterator for ObservedRows<I> {
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().inspect(|_| observe_row())
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
impl<I: DoubleEndedIterator> DoubleEndedIterator for ObservedRows<I> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().inspect(|_| observe_row())
    }
}
impl<'a> IntoIterator for &'a DemandRecords {
    type Item = (&'a WorthQueryOutputDemandKey, &'a DemandRecord);
    type IntoIter = ObservedRows<
        std::collections::btree_map::Iter<'a, WorthQueryOutputDemandKey, DemandRecord>,
    >;
    fn into_iter(self) -> Self::IntoIter {
        ObservedRows(self.0.iter())
    }
}
impl<'a> IntoIterator for &'a mut DemandRecords {
    type Item = (&'a WorthQueryOutputDemandKey, &'a mut DemandRecord);
    type IntoIter = ObservedRows<
        std::collections::btree_map::IterMut<'a, WorthQueryOutputDemandKey, DemandRecord>,
    >;
    fn into_iter(self) -> Self::IntoIter {
        ObservedRows(self.0.iter_mut())
    }
}

impl DemandRecords {
    pub(super) fn admit_publication_head_insertion(
        &self,
        admission: &mut crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission,
    ) -> Result<(), crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial> {
        self.2.admit_insertion(admission)
    }
    pub(super) fn publication_counter(
        &self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        admission: &mut crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission,
    ) -> Result<
        std::sync::Arc<std::sync::atomic::AtomicUsize>,
        crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial,
    > {
        self.2.counter(occurrence, admission)
    }
    pub(super) fn publication_selection(
        &self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        admission: &mut crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission,
    ) -> Result<
        super::PublicationHeadSelection,
        crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial,
    > {
        self.2.selection(occurrence, admission)
    }
}
