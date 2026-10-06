//! Registry custody for a required successor a queue frame left unfinished.
//!
//! A queue frame refreshes a row for owners outside the request. When the
//! successor's stage defers, or its run stops, that successor is the only
//! handle that can finish the row. It stays on its own row until the row is
//! Ready, a newer row of its occurrence supersedes it, or nothing awaits it.

use std::any::Any;

use super::refreshed_rejoin::{awaited_by_stale_owner, superseded};
use super::{
    DemandRecord, DemandRegistryState, DemandState, WorthQueryOutputDemandKey,
    WorthQueryOutputDemandRegistry,
};
use crate::domain_computation::primary_graph::application_output_demand::RequiredOutputCustodyCapacity;
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;

/// The typed successor, erased so the registry stays independent of schema.
pub(in crate::domain_computation::primary_graph) type HeldRequiredSuccessor =
    Box<dyn Any + Send + Sync>;

impl WorthQueryOutputDemandRegistry {
    /// Fund the registry's copy of one held successor under the same required
    /// custody limit as a caller's continuations.
    pub(in crate::domain_computation::primary_graph) fn reserve_held_successor_capacity(
        &self,
        bytes: usize,
    ) -> Result<RequiredOutputCustodyCapacity, WorthQueryOutputDemandDenial> {
        self.reserve_required_custody_capacity(bytes)
    }

    /// Keep `successor` on the row `key_of` names. Returns what the caller
    /// drops after this call: a replaced older successor of that row, or
    /// `successor` itself when its row no longer needs it.
    pub(in crate::domain_computation::primary_graph) fn hold_required_successor(
        &self,
        successor: HeldRequiredSuccessor,
        key_of: impl FnOnce(&HeldRequiredSuccessor) -> &WorthQueryOutputDemandKey,
    ) -> Option<HeldRequiredSuccessor> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match state.records.get_mut(key_of(&successor)) {
            Some(record) if needs_successor(record) => record.held_successor.replace(successor),
            _ => Some(successor),
        }
    }

    /// Take the successor held on `key`'s row so a dependent can resume it.
    pub(in crate::domain_computation::primary_graph) fn take_held_successor(
        &self,
        key: &WorthQueryOutputDemandKey,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<HeldRequiredSuccessor>, WorthQueryOutputDemandDenial> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.charge_record_lookup(key, admission)?;
        Ok(state
            .records
            .get_mut(key)
            .and_then(|record| record.held_successor.take()))
    }
}

impl DemandRegistryState {
    /// The held successor of `key`'s row once the row is finished, or once
    /// nothing but that successor awaits it. The caller drops it after
    /// releasing the lock.
    pub(super) fn take_finished_successor(
        &mut self,
        key: &WorthQueryOutputDemandKey,
    ) -> Option<HeldRequiredSuccessor> {
        let record = self.records.get(key)?;
        record.held_successor.as_ref()?;
        // Only demands own a row: a prerequisite claim of a cached dependent
        // row keeps the row, not the successor.
        let orphaned = record.interests <= 1 && !awaited_by_stale_owner(&self.records, key);
        if needs_successor(record) && !orphaned {
            return None;
        }
        self.records.get_mut(key)?.held_successor.take()
    }
}

/// Whether the row still has work only its held successor can finish.
fn needs_successor(record: &DemandRecord) -> bool {
    !record.has_cached_ready()
        && !superseded(record)
        && !matches!(record.state, DemandState::Failed(_))
}
