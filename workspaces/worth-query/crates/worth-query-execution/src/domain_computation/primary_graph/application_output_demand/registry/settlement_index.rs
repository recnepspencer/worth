//! One exact settlement posting index. Its vacant nodes are invisible to reads.

#[cfg(test)]
mod navigation_tests;
mod removal;
mod reservation;

use std::collections::BTreeMap;
use std::mem::size_of;
use std::sync::{Arc, Mutex};

use super::{DemandRegistryState, WorthQueryOutputDemandKey};
use crate::domain_computation::primary_graph::output_lineage::{
    invalidation::InvalidationEditAdmission, RecordedSettlementIdentity, SemanticSource,
};
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind as Kind,
};

type Address = (
    worth_runtime_world::facade::ProductBranchIncarnation,
    u64,
    usize,
);
pub(super) type Posting = Arc<
    Mutex<
        Option<(
            Arc<RecordedSettlementIdentity>,
            Arc<WorthQueryOutputDemandKey>,
        )>,
    >,
>;
type SourcePostings = BTreeMap<Address, Posting>;

/// Prepared before the product effect. Cancellation only links this node;
/// ordered-index removal runs at the next admitted registry entry.
pub(super) struct PendingVacancyCleanup {
    identity: Arc<RecordedSettlementIdentity>,
    next: Option<Box<Self>>,
}

#[derive(Default)]
pub(super) struct SettlementIndex {
    sources: BTreeMap<SemanticSource, SourcePostings>,
    retained_bytes: usize,
    cancelled_head: Option<Box<PendingVacancyCleanup>>,
}

impl Drop for SettlementIndex {
    fn drop(&mut self) {
        let mut current = self.cancelled_head.take();
        while let Some(mut cue) = current {
            current = cue.next.take();
        }
    }
}

impl SettlementIndex {
    pub(super) fn get_exact_admitted(
        &self,
        identity: &RecordedSettlementIdentity,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<Arc<WorthQueryOutputDemandKey>>, WorthQueryOutputDemandDenial> {
        admission
            .charge_external_work(
                tree_lookup_work::<SemanticSource>(self.sources.len()).ok_or_else(work_denial)?,
            )
            .map_err(|_| work_denial())?;
        let Some(source) = self.sources.get(identity.source()) else {
            return Ok(None);
        };
        admission
            .charge_external_work(
                tree_lookup_work::<Address>(source.len()).ok_or_else(work_denial)?,
            )
            .map_err(|_| work_denial())?;
        let Some(posting) = source.get(&identity.address()) else {
            return Ok(None);
        };
        admission
            .charge_external_work(1)
            .map_err(|_| work_denial())?;
        let held = posting
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some((recorded, key)) = held.as_ref() else {
            return Ok(None);
        };
        Ok((recorded.as_ref() == identity).then(|| Arc::clone(key)))
    }

    pub(super) fn fill_prepared(
        posting: &Posting,
        expected: &RecordedSettlementIdentity,
        identity: Arc<RecordedSettlementIdentity>,
        key: Arc<WorthQueryOutputDemandKey>,
    ) {
        assert_eq!(
            expected,
            identity.as_ref(),
            "World performed the prepared exact settlement"
        );
        assert!(
            posting
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .replace((identity, key))
                .is_none(),
            "one prepared settlement fills once"
        );
    }

    pub(super) fn remove(&mut self, identity: &RecordedSettlementIdentity) -> usize {
        let outer_before =
            tree_retained_bytes::<SemanticSource, SourcePostings>(self.sources.len())
                .expect("admitted settlement tree size");
        let source = self
            .sources
            .get_mut(identity.source())
            .expect("retained settlement source");
        let inner_before = tree_retained_bytes::<Address, Posting>(source.len())
            .expect("admitted settlement address tree size");
        let removed = source
            .remove(&identity.address())
            .expect("retained exact settlement address");
        if let Some((recorded, _)) = removed
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
        {
            assert_eq!(
                recorded.as_ref(),
                identity,
                "compact posting address never replaces exact identity"
            );
        }
        let inner_after = tree_retained_bytes::<Address, Posting>(source.len())
            .expect("admitted settlement address tree size");
        if source.is_empty() {
            self.sources.remove(identity.source());
        }
        let outer_after = tree_retained_bytes::<SemanticSource, SourcePostings>(self.sources.len())
            .expect("admitted settlement tree size");
        let released = inner_before
            .checked_sub(inner_after)
            .unwrap()
            .checked_add(outer_before.checked_sub(outer_after).unwrap())
            .and_then(|bytes| bytes.checked_add(posting_cell_bytes()))
            .and_then(|bytes| bytes.checked_add(size_of::<PendingVacancyCleanup>()))
            .unwrap();
        self.retained_bytes -= released;
        released
    }
}

impl DemandRegistryState {
    pub(super) fn defer_cancelled_settlement_vacancy(
        &mut self,
        mut cleanup: Box<PendingVacancyCleanup>,
    ) {
        cleanup.next = self.settlement_keys.cancelled_head.take();
        self.settlement_keys.cancelled_head = Some(cleanup);
    }

    pub(super) fn drain_cancelled_settlement_vacancies(
        &mut self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        while let Some(head) = self.settlement_keys.cancelled_head.as_ref() {
            let source_count = self.settlement_keys.sources.len();
            let outer_work =
                tree_lookup_work::<SemanticSource>(source_count).ok_or_else(work_denial)?;
            admission
                .charge_external_work(outer_work)
                .map_err(|_| work_denial())?;
            let address_count = self
                .settlement_keys
                .sources
                .get(head.identity.source())
                .expect("cancelled posting source remains retained")
                .len();
            // Removing a final address also removes the outer source. Admit
            // both paths before unlinking the cancellation cue.
            let work = tree_lookup_work::<Address>(address_count)
                .and_then(|part| part.checked_mul(2))
                .and_then(|part| part.checked_add(outer_work))
                .ok_or_else(work_denial)?;
            admission
                .charge_external_work(work)
                .map_err(|_| work_denial())?;
            let mut cancelled = self.settlement_keys.cancelled_head.take().unwrap();
            self.settlement_keys.cancelled_head = cancelled.next.take();
            let released = self.settlement_keys.remove(cancelled.identity.as_ref());
            self.required_reserved_bytes -= released;
        }
        Ok(())
    }

    pub(super) fn reserve_settlement_vacancy(
        &mut self,
        identity: &Arc<RecordedSettlementIdentity>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(Posting, Box<PendingVacancyCleanup>), WorthQueryOutputDemandDenial> {
        reservation::reserve(self, identity, admission)
    }
}

fn work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        Kind::WorkBudgetExceeded,
        "exact settlement index exceeds request work",
    )
}

fn capacity_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        Kind::RetentionBudgetExceeded,
        "exact settlement index exceeds registry capacity",
    )
}

/// Stable std B-tree nodes have at least five occupied entries below the root.
/// One spare root covers the first insertion and split boundaries.
fn tree_retained_bytes<K, V>(entries: usize) -> Option<usize> {
    if entries == 0 {
        return Some(0);
    }
    let nodes = 1usize.checked_add(entries.checked_add(4)? / 5)?;
    node_bytes::<K, V>()?.checked_mul(nodes)
}

fn node_bytes<K, V>() -> Option<usize> {
    size_of::<(K, V)>()
        .checked_mul(11)?
        .checked_add(size_of::<usize>().checked_mul(16)?)?
        .checked_add(64)
}

fn posting_cell_bytes() -> usize {
    let alignment = std::mem::align_of::<
        Mutex<
            Option<(
                Arc<RecordedSettlementIdentity>,
                Arc<WorthQueryOutputDemandKey>,
            )>,
        >,
    >()
    .max(std::mem::align_of::<usize>());
    let raw = size_of::<usize>() * 2
        + size_of::<
            Mutex<
                Option<(
                    Arc<RecordedSettlementIdentity>,
                    Arc<WorthQueryOutputDemandKey>,
                )>,
            >,
        >();
    raw.div_ceil(alignment) * alignment
}

fn tree_insert_bytes<K, V>(entries: usize) -> Option<u64> {
    let levels = usize::BITS as usize - entries.max(1).leading_zeros() as usize;
    u64::try_from(node_bytes::<K, V>()?.checked_mul(levels.checked_add(2)?)?).ok()
}

fn tree_lookup_work<K>(entries: usize) -> Option<u64> {
    if entries == 0 {
        return Some(1);
    }
    let levels = tree_level_bound(entries)?;
    let _key = std::marker::PhantomData::<K>;
    let comparisons = entries.min(11_usize.checked_mul(levels)?);
    u64::try_from(comparisons).ok()
}

fn tree_level_bound(entries: usize) -> Option<usize> {
    // std's B-tree has at most 11 keys per node and at least 5 keys in each
    // non-root node. A tree with another level needs at least 11, 71, 431...
    // entries. A selected search visits no key twice, even after deletion.
    let mut levels = 1_usize;
    let mut minimum_for_next_level = 11_u128;
    while minimum_for_next_level <= entries as u128 {
        levels = levels.checked_add(1)?;
        minimum_for_next_level = minimum_for_next_level.checked_mul(6)?.checked_add(5)?;
    }
    Some(levels)
}
