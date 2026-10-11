//! Issued undelivered publications, indexed by their exact Product occurrence.
use super::*;
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind;
use std::sync::atomic::{AtomicUsize, Ordering};
use worth_runtime_world::facade::ProductBranchIncarnation;

#[derive(Default)]
pub(super) struct PublicationHeads(pub(super) BTreeMap<ProductBranchIncarnation, Head>);
pub(super) struct Head {
    rows: usize,
    pub(super) undelivered: Arc<AtomicUsize>,
}

/// Only the registry's answer can issue a live-head witness.
pub(in crate::domain_computation::primary_graph) struct LivePublicationHead(());
pub(in crate::domain_computation::primary_graph) enum PublicationHeadSelection {
    Live(LivePublicationHead),
    Frozen,
}

/// Kept by the row even while its checkpoint is claimed for delivery. Dropping
/// the retired row releases this hold without a registry reentry or a scan.
pub(super) struct UndeliveredPublication(Arc<AtomicUsize>);
impl UndeliveredPublication {
    pub(super) fn new(counter: Arc<AtomicUsize>) -> Self {
        counter
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                count.checked_add(1)
            })
            .expect("one undelivered hold per admitted row");
        Self(counter)
    }
}
impl Drop for UndeliveredPublication {
    fn drop(&mut self) {
        let previous = self.0.fetch_sub(1, Ordering::AcqRel);
        assert_ne!(previous, 0, "the published row owns its hold");
    }
}

impl PublicationHeads {
    pub(super) fn insert(&mut self, occurrence: ProductBranchIncarnation) -> Arc<AtomicUsize> {
        let head = self.0.entry(occurrence).or_insert_with(|| Head {
            rows: 0,
            undelivered: Arc::new(AtomicUsize::new(0)),
        });
        head.rows += 1;
        Arc::clone(&head.undelivered)
    }
    pub(super) fn admit_insertion(
        &self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        self.pay_seek(admission)?;
        let height = usize::BITS as usize - self.0.len().max(1).leading_zeros() as usize;
        let peak = Self::node_bytes().checked_mul(height + 1).ok_or_else(|| {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
                "",
            )
        })?;
        admission.admit_read_scratch(peak as u64).map_err(|_| {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
                "",
            )
        })?;
        admission.charge_external_work(peak as u64).map_err(|_| {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                "",
            )
        })
    }
    pub(super) fn counter(
        &self,
        occurrence: ProductBranchIncarnation,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Arc<AtomicUsize>, WorthQueryOutputDemandDenial> {
        self.pay_seek(admission)?;
        Ok(Arc::clone(&self.0[&occurrence].undelivered))
    }
    pub(super) fn remove(&mut self, occurrence: ProductBranchIncarnation) {
        let head = self
            .0
            .get_mut(&occurrence)
            .expect("the row's head is indexed");
        head.rows -= 1;
        if head.rows == 0 {
            self.0.remove(&occurrence);
            if self.0.is_empty() {
                // No row funds the empty tree: release its last node as well.
                self.0 = BTreeMap::new();
            }
        }
    }
    pub(super) fn selection(
        &self,
        occurrence: ProductBranchIncarnation,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<PublicationHeadSelection, WorthQueryOutputDemandDenial> {
        self.pay_seek(admission)?;
        Ok(
            if self
                .0
                .get(&occurrence)
                .is_some_and(|head| head.undelivered.load(Ordering::Acquire) != 0)
            {
                PublicationHeadSelection::Frozen
            } else {
                PublicationHeadSelection::Live(LivePublicationHead(()))
            },
        )
    }
    fn pay_seek(
        &self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        // Pay the ordered seek before inspecting the per-head counter. Each
        // comparison has a fixed Product incarnation key; no row is traversed.
        let count = self.0.len();
        let height = usize::BITS as usize - count.max(1).leading_zeros() as usize;
        let work = count.min(11) * height * 3 + 1;
        admission.charge_ordered_operations(2, work as u64).map_err(|_| {
            WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                "the live-head lookup exhausted its request",
            )
        })?;
        Ok(())
    }
    pub(super) fn node_bytes() -> usize {
        // Each row funds one worst-case head node and the counter allocation,
        // including an insertion's extra node and the Arc headers.
        12 * (std::mem::size_of::<ProductBranchIncarnation>() + std::mem::size_of::<Head>())
            + 14 * std::mem::size_of::<usize>()
            + 3 * std::mem::size_of::<usize>()
    }
}

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn publication_head_selection(
        &self,
        occurrence: ProductBranchIncarnation,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<PublicationHeadSelection, WorthQueryOutputDemandDenial> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.records.publication_selection(occurrence, admission)
    }
    pub(in crate::domain_computation::primary_graph) fn record_selection_state(
        &self,
        frozen: bool,
    ) {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .last_selection_frozen = Some(frozen);
    }
}

#[cfg(test)]
impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn publication_counts_for_registry_test(
        &self,
    ) -> (usize, usize) {
        let state = self.state.lock().unwrap();
        let scan = state.records.values().filter(|record| matches!(&record.state,
            DemandState::Output(output) if !matches!(output.advancement(), WorthQueryOutputAdvancement::Stopped { .. })
            && matches!(output.checkpoint, Some(WorthQueryOutputCheckpoint::Published { .. })))).count();
        (state.records.publication_count_for_test(), scan)
    }
}
