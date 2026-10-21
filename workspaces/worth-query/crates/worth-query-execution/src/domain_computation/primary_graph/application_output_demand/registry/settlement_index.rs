//! One exact settlement posting index. Its vacant nodes are invisible to reads.

mod reclaim;
mod reservation;

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;

use super::DemandRegistryState;
use super::WorthQueryOutputDemandKey;
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;
use crate::domain_computation::primary_graph::output_lineage::SemanticSource;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind as Kind;

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
    ) -> Result<Option<Arc<WorthQueryOutputDemandKey>>, WorthQueryOutputDemandDenial> {
        let Some(source) = self.sources.get(identity.source()) else {
            return Ok(None);
        };

        let Some(posting) = source.get(&identity.address()) else {
            return Ok(None);
        };

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

    pub(super) fn remove(&mut self, identity: &RecordedSettlementIdentity) {
        let source = self
            .sources
            .get_mut(identity.source())
            .expect("retained settlement source");
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
                "compact address does not substitute for exact identity"
            );
        }
        if source.is_empty() {
            self.sources.remove(identity.source());
        }
    }

    /// Point `identity`'s posting at `key`, the row that now answers for it.
    pub(super) fn repoint(
        &self,
        identity: &RecordedSettlementIdentity,
        key: Arc<WorthQueryOutputDemandKey>,
    ) {
        let posting = self
            .sources
            .get(identity.source())
            .and_then(|source| source.get(&identity.address()))
            .expect("retained exact settlement address");
        let mut held = posting
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some((recorded, answering)) = held.as_mut()
            && recorded.as_ref() == identity
        {
            *answering = key;
        }
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

    pub(super) fn drain_cancelled_settlement_vacancies(&mut self) {
        while let Some(mut cancelled) = self.settlement_keys.cancelled_head.take() {
            self.settlement_keys.cancelled_head = cancelled.next.take();
            self.settlement_keys.remove(cancelled.identity.as_ref());
        }
    }

    pub(super) fn reserve_settlement_vacancy(
        &mut self,
        identity: &Arc<RecordedSettlementIdentity>,
    ) -> Result<(Posting, Box<PendingVacancyCleanup>), WorthQueryOutputDemandDenial> {
        reservation::reserve(self, identity)
    }
}
