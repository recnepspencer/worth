//! A demand's newest exact settlement supersedes its earlier ones, and those
//! of every row of its occurrence it replaced. They leave the registry
//! together with their invalidation rows, never before them: a live dependent
//! row still resolves its upstream demand through this index.
//!
//! Retirement is an owner derived edit after the effect has committed. It runs
//! under the owner's own edit admission, so it can neither fail nor starve the
//! request that published; a denied retirement waits for the next one. A
//! superseded settlement waits in this index, which names it again. A
//! released one has left this index and waits with the invalidation owner.
//!
//! A row retired for custody drops its postings and keeps its lineage, so a
//! lineage head no row posts is an evicted one. Only a row holds the claims on
//! what its record consumed: an evicted head that consumed upstream outputs is
//! never replayed, and the next start of its producer succeeds it. One that
//! consumed none is replayed while its dependencies match, and its row posts
//! it again.
//!
//! Every publication also retires the settlement of the generation its record
//! displaced. A row of this occurrence usually posts it, and the supersession
//! names it; when none does, as after an eviction, it retires as a released
//! settlement does.

use std::sync::Arc;

use super::refreshed_rejoin::occurrence_rows;
use super::{DemandRegistryState, WorthQueryOutputDemandKey, WorthQueryOutputDemandRegistry};
use crate::domain_computation::primary_graph::output_lineage::{
    invalidation::{InvalidationEditAdmission, SourceInvalidationOwner},
    RecordedSettlementIdentity,
};

type Identity = Arc<RecordedSettlementIdentity>;

/// Returned by a settlement publication. Retiring needs the newest row
/// registered with the invalidation owner, so the publisher calls it after.
#[must_use = "superseded settlements stay retained until retired"]
pub(in crate::domain_computation::primary_graph) struct SupersededSettlements {
    registry: WorthQueryOutputDemandRegistry,
    key: Arc<WorthQueryOutputDemandKey>,
}

impl SupersededSettlements {
    pub(super) fn new(
        registry: WorthQueryOutputDemandRegistry,
        key: Arc<WorthQueryOutputDemandKey>,
    ) -> Self {
        Self { registry, key }
    }

    /// `displaced` is the settlement of the generation the publication's
    /// record displaced in the lineage.
    pub(in crate::domain_computation::primary_graph) fn retire(
        self,
        displaced: Option<Identity>,
        owner: &SourceInvalidationOwner,
    ) {
        let named = self
            .registry
            .retire_superseded_settlements(&self.key, owner);
        // A named settlement is retried by the next supersession. One no row
        // of this occurrence posts retires as a released settlement does.
        if let Some(displaced) = displaced.filter(|displaced| !named.contains(displaced)) {
            self.registry
                .retire_released_settlements(vec![(displaced, 0)], owner);
        }
    }
}

impl WorthQueryOutputDemandRegistry {
    /// Whether a row posts `identity`.
    pub(in crate::domain_computation::primary_graph) fn posts_settlement(
        &self,
        identity: &RecordedSettlementIdentity,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Ok(state
            .settlement_keys
            .get_exact_admitted(identity, admission)?
            .is_some())
    }

    /// Called after the newest settlement of `key` is registered. Rows still
    /// read by a live dependent stay, and a later supersession retries them:
    /// a replaced row hands its postings to the newest row of its occurrence,
    /// so every publication of the occurrence names them again. Returns the
    /// settlements it named.
    fn retire_superseded_settlements(
        &self,
        key: &WorthQueryOutputDemandKey,
        owner: &SourceInvalidationOwner,
    ) -> Vec<Identity> {
        let mut admission = owner.edit_admission();
        let candidates = {
            let state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if state.charge_record_lookup(key, &mut admission).is_err()
                || !state.records.contains_key(key)
            {
                return Vec::new();
            }
            let mut candidates = Vec::new();
            for (other, record) in occurrence_rows(&state.records, key) {
                let superseded = if other == key {
                    record.settlements.len().saturating_sub(1)
                } else if key.replacement_order(other) == Some(std::cmp::Ordering::Greater) {
                    record.settlements.len()
                } else {
                    0
                };
                if admission.charge_external_work(1).is_err()
                    || admit_slots(superseded, &mut admission).is_err()
                {
                    return Vec::new();
                }
                candidates.extend(
                    record.settlements[..superseded]
                        .iter()
                        .map(|(identity, _)| Arc::clone(identity)),
                );
            }
            candidates
        };
        if !candidates.is_empty() {
            drop(self.retire_settlements(&candidates, owner, &mut admission));
        }
        candidates
    }

    /// Released settlements already left this index; their rows may still
    /// hold upstream rows of other demands, which leave with them. The owner
    /// keeps the ones that cannot leave yet and retries them at the next
    /// release.
    pub(super) fn retire_released_settlements(
        &self,
        released: Vec<(Identity, usize)>,
        owner: &SourceInvalidationOwner,
    ) {
        if released.is_empty() {
            return;
        }
        let mut admission = owner.edit_admission();
        let released = released.into_iter().map(|(identity, _)| identity);
        let retired = owner.retire_released(released, &mut admission);
        self.forget_retired(&retired, &mut admission);
    }

    /// The candidates the invalidation owner retired.
    fn retire_settlements(
        &self,
        candidates: &[Identity],
        owner: &SourceInvalidationOwner,
        admission: &mut InvalidationEditAdmission,
    ) -> Vec<Identity> {
        let retired = owner.retire_settlements(candidates, admission);
        self.forget_retired(&retired, admission);
        retired
    }

    /// Rows the owner retired leave this index with them.
    fn forget_retired(&self, retired: &[Identity], admission: &mut InvalidationEditAdmission) {
        if retired.is_empty() {
            return;
        }
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.forget_retired_settlements(retired, admission);
    }
}

impl DemandRegistryState {
    fn forget_retired_settlements(
        &mut self,
        retired: &[Identity],
        admission: &mut InvalidationEditAdmission,
    ) {
        for identity in retired {
            let Ok(Some(key)) = self.settlement_keys.get_exact_admitted(identity, admission) else {
                continue;
            };
            if self
                .settlement_keys
                .admit_selected_removal_work(std::iter::once(identity), admission)
                .is_err()
                || self.charge_record_lookup(&key, admission).is_err()
            {
                return;
            }
            let settled = self
                .records
                .get(key.as_ref())
                .map_or(0, |record| record.settlements.len());
            if admission.charge_external_work(settled as u64).is_err() {
                return;
            }
            let index_bytes = self.settlement_keys.remove(identity);
            let posting_bytes = self
                .records
                .get_mut(key.as_ref())
                .and_then(|record| {
                    let at = record
                        .settlements
                        .iter()
                        .position(|(settled, _)| settled == identity)?;
                    let key_bytes = record.settlements.remove(at).1;
                    // The slot leaves with its posting, unless a prepared
                    // posting of this row is about to fill it.
                    let slots = record.settlements.capacity();
                    if record.prepared_prerequisite_claims == 0 {
                        record.settlements.shrink_to_fit();
                    }
                    let freed = slots - record.settlements.capacity();
                    Some(key_bytes + freed * std::mem::size_of::<(Identity, usize)>())
                })
                .unwrap_or(0);
            self.required_reserved_bytes = self
                .required_reserved_bytes
                .saturating_sub(index_bytes.saturating_add(posting_bytes));
        }
    }
}

fn admit_slots(
    count: usize,
    admission: &mut InvalidationEditAdmission,
) -> Result<(), worth_relational::facade::mvcc::CompanionPreflightStop> {
    let bytes = count
        .checked_mul(std::mem::size_of::<Identity>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(worth_relational::facade::mvcc::CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
    admission.admit_read_scratch(bytes)?;
    admission.charge_external_work(count as u64)
}
