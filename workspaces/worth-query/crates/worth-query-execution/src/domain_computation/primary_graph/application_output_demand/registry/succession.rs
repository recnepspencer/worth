//! A row reopened to refresh its exact Ready, until that refresh publishes.
//!
//! A refresh that stops without publishing (its request stopped, or its stage
//! deferred) leaves the row unclaimed. A row that was reopened from Ready goes
//! back to that Ready, so the next caller's wave certifies it and claims the
//! refresh again with its consumed upstreams matched, instead of rerunning a
//! scheduled successor that no longer knows which upstreams already settled.
//!
//! A row admitted under a new key that ends before it publishes, superseded
//! by the World or let go by its last owner, gives its occurrence back to the
//! newest Ready it replaced, for the same reason. That holds for a refresh and
//! for a demand that named a newer source alike: the owners and dependents of
//! the replaced Ready refresh it from the current source instead of stopping.

use super::{
    DemandRecord, DemandState, WorthQueryOutputAdvancement, WorthQueryOutputCheckpoint,
    WorthQueryOutputProgress,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};

pub(super) struct Succession {
    predecessor: [u8; 32],
    reopened: Option<WorthQueryOutputProgress>,
}

impl Succession {
    /// A new row admitted as the successor of `predecessor`'s key.
    pub(super) fn new(predecessor: [u8; 32]) -> Self {
        Self {
            predecessor,
            reopened: None,
        }
    }

    /// An existing row whose exact Ready `reopened` this refresh replaces.
    pub(super) fn reopening(predecessor: [u8; 32], reopened: WorthQueryOutputProgress) -> Self {
        Self {
            predecessor,
            reopened: Some(reopened),
        }
    }

    pub(super) fn predecessor(&self) -> [u8; 32] {
        self.predecessor
    }

    /// The key identity a successor row replaces, if the row is one.
    pub(super) fn predecessor_of(succession: &Option<Self>) -> Option<[u8; 32]> {
        succession.as_ref().map(Self::predecessor)
    }
}

impl DemandRecord {
    /// The row's unfinished refresh gives up its claim. A reopened Ready
    /// comes back as the row's output; any other row takes `unclaimed`. The
    /// reopen cleared the row's performed source, so one it holds now was
    /// attached during the refresh, is newer than the reopen, and stays in
    /// the row's custody.
    pub(super) fn leave_refresh_unclaimed(&mut self, unclaimed: DemandState) {
        match self
            .successor_of
            .as_mut()
            .and_then(|succession| succession.reopened.take())
        {
            Some(ready) => {
                self.state = DemandState::Output(ready);
                self.successor_of = None;
            }
            None => self.state = unclaimed,
        }
    }

    /// The reopened Ready exists only to return the row to the demands that
    /// await it. Once none does, the caller drops it after the lock.
    pub(super) fn take_unawaited_reopened(&mut self) -> Option<WorthQueryOutputProgress> {
        if self.interests != 0 {
            return None;
        }
        self.successor_of.as_mut()?.reopened.take()
    }
}

impl super::WorthQueryOutputDemandRegistry {
    /// Whether `key` is a new-key row the World superseded before it
    /// published and the newest Ready it replaced answers for the occurrence
    /// again.
    pub(in crate::domain_computation::primary_graph) fn give_back_replaced_ready(
        &self,
        key: &super::WorthQueryOutputDemandKey,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.charge_record_lookup(key, admission)?;
        let Some(record) = state.records.get(key) else {
            return Ok(false);
        };
        if !record.unpublished_new_key() || !super::refreshed_rejoin::superseded(record) {
            return Ok(false);
        }
        // The walk visits at most every row once.
        admission
            .charge_external_work(u64::try_from(state.records.len()).map_err(|_| work_denial())?)
            .map_err(|_| work_denial())?;
        Ok(state.revive_replaced_ready(key))
    }
}

impl DemandRecord {
    /// A row admitted under a new key that has not published: it holds no
    /// output of its own and reopened none.
    pub(super) fn unpublished_new_key(&self) -> bool {
        !matches!(self.state, DemandState::Output(_))
            && self
                .successor_of
                .as_ref()
                .is_none_or(|succession| succession.reopened.is_none())
    }
}

impl super::DemandRegistryState {
    /// The unpublished new-key row at `key` ended: the World superseded it,
    /// or its last owner let it go. Unless a newer row answers for the
    /// occurrence, the newest Ready it replaced answers again, so the next
    /// wave refreshes that Ready from the current source instead of failing
    /// the owners and dependents that read it. Returns whether that Ready
    /// answers, revived here or earlier. Rows of one source order by their
    /// refresh, so a refreshed alias is newer than the row it refreshed. The
    /// walk stays within `key`'s occurrence and charges nothing: a caller
    /// that meters it pays before it runs.
    pub(super) fn revive_replaced_ready(&mut self, key: &super::WorthQueryOutputDemandKey) -> bool {
        use super::refreshed_rejoin::refresh_order;
        let mut replaced: Option<&super::WorthQueryOutputDemandKey> = None;
        for (other, record) in super::refreshed_rejoin::occurrence_rows(&self.records, key) {
            match refresh_order(other, key) {
                // A newer row answers for the occurrence.
                Some(std::cmp::Ordering::Greater) => return false,
                Some(std::cmp::Ordering::Less)
                    if replaced_ready(record)
                        && replaced.is_none_or(|newest| {
                            refresh_order(other, newest) == Some(std::cmp::Ordering::Greater)
                        }) =>
                {
                    replaced = Some(other);
                }
                _ => {}
            }
        }
        let Some(replaced) = replaced.cloned() else {
            return false;
        };
        let record = self
            .records
            .get_mut(&replaced)
            .expect("the replaced Ready row is retained");
        // The revived row's claims on its upstreams were released when it
        // stopped, unless claims on the row itself held it then; those stay
        // with it. Either way its next refresh certifies its upstreams again
        // and its publication replaces whatever claims it kept.
        if let DemandState::Output(output) = &mut record.state {
            if matches!(
                output.advancement,
                WorthQueryOutputAdvancement::Stopped { .. }
            ) {
                output.advancement = WorthQueryOutputAdvancement::Idle;
                record.wake.notify();
            }
        }
        true
    }
}

/// A row that holds a Ready a newer source replaced, or that already answers
/// for its occurrence again.
fn replaced_ready(record: &DemandRecord) -> bool {
    matches!(&record.state, DemandState::Output(output)
        if matches!(output.checkpoint, Some(WorthQueryOutputCheckpoint::Ready(_)))
            && (!matches!(output.advancement, WorthQueryOutputAdvancement::Stopped { .. })
                || super::refreshed_rejoin::superseded(record)))
}

fn work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        "replaced Ready lookup exceeds request work",
    )
}
