//! Registry custody for a required successor a queue frame left unfinished.
//!
//! A queue frame refreshes a row for owners outside the request. When the
//! successor's stage defers, or its run stops, that successor is the only
//! handle that can finish the row. It stays on its own row until the row is
//! Ready, a newer row of its occurrence supersedes it, or nothing awaits it.

use std::any::Any;

use super::refreshed_rejoin::awaited_by_stale_owner;
use super::refreshed_rejoin::superseded;
use super::DemandRecord;
use super::DemandRegistryState;
use super::DemandState;
use super::WorthQueryOutputDemandKey;
use super::WorthQueryOutputDemandRegistry;

use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;

/// The typed successor, erased so the registry stays independent of schema.
pub(in crate::domain_computation::primary_graph) type HeldRequiredSuccessor =
    Box<dyn Any + Send + Sync>;

impl WorthQueryOutputDemandRegistry {
    /// Keep `successor` on the row `key_of` names. Returns what the caller
    /// drops after this call: a replaced older successor of that row, or
    /// `successor` itself when its row no longer needs it.
    pub(in crate::domain_computation::primary_graph) fn hold_required_successor(
        &self,
        successor: HeldRequiredSuccessor,
        key_of: impl FnOnce(&HeldRequiredSuccessor) -> Option<&WorthQueryOutputDemandKey>,
    ) -> Option<HeldRequiredSuccessor> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(key) = key_of(&successor) else {
            return Some(successor);
        };
        let awaited = awaited_by_stale_owner(&state.records, key);

        match state.records.get_mut(key) {
            Some(mut record) if needs_successor(&record) && (record.interests > 1 || awaited) => {
                record.held_successor.replace(successor)
            }
            _ => Some(successor),
        }
    }

    /// Take the successor held on `key`'s row so a dependent can resume it.
    pub(in crate::domain_computation::primary_graph) fn take_published_held_successor(
        &self,
        key: &WorthQueryOutputDemandKey,
    ) -> Result<Option<HeldRequiredSuccessor>, WorthQueryOutputDemandDenial> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        Ok(state
            .records
            .get_mut(key)
            .and_then(|mut record| {
                let published = matches!(&record.state, DemandState::Output(output)
                    if matches!(output.advancement(), super::WorthQueryOutputAdvancement::Idle)
                    && matches!(output.checkpoint, Some(super::WorthQueryOutputCheckpoint::Published { .. } | super::WorthQueryOutputCheckpoint::Delivered { .. })));
                published.then(|| record.held_successor.take()).flatten()
            }))
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
        let orphaned = record.successor_is_orphaned(awaited_by_stale_owner(&self.records, key));
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

impl DemandRecord {
    pub(super) fn successor_is_orphaned(&self, awaited: bool) -> bool {
        self.held_successor.is_some() && self.interests <= 1 && !awaited
    }
}

/// A moved registry frame reached through the requesting member's actual
/// retained prerequisite claim. A declaration or row key cannot issue it.
pub(in crate::domain_computation::primary_graph) struct RequiredUpstreamSuccessor {
    held: HeldRequiredSuccessor,
}
impl RequiredUpstreamSuccessor {
    pub(super) fn issued(held: HeldRequiredSuccessor) -> Self {
        Self { held }
    }
    pub(in crate::domain_computation::primary_graph) fn into_held(self) -> HeldRequiredSuccessor {
        self.held
    }
}
