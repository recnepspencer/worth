//! A cached Ready one starting demand joined is still cache.
//!
//! A closed row keeps its Ready for an equivalent later demand, and required
//! work refused custody retires such rows. A demand that starts on one holds
//! it open, so no reservation can retire it, and its refresh keeps the stale
//! Ready beside the one that replaces it. Until that demand has settled, the
//! Ready is nothing it was given: a refresh custody cannot keep releases the
//! Ready with its claims, and the row computes again as a new one would. The
//! lineage settlement the Ready posted leaves with it, so the execution
//! decides Fresh and never reuses the output it evicted.

use super::super::refreshed_rejoin::{charge_whole_map, WholeMapPasses};
use super::super::{
    DemandRecord, DemandState, WorthQueryOutputAdvancement, WorthQueryOutputCheckpoint,
    WorthQueryOutputDemandInterest, WorthQueryOutputDemandRegistry,
};
use crate::domain_computation::primary_graph::{
    InvalidationEditAdmission, WorthQueryOutputDemandDenial,
};

impl WorthQueryOutputDemandRegistry {
    /// Release the idle Ready of `interest`'s row when that interest alone
    /// holds the row. Whether it did; the stop of a walk the request's work
    /// cannot pay for releases nothing.
    pub(in crate::domain_computation::primary_graph) fn release_joined_ready(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        self.drain_closed_cleanup_admitted(admission)?;
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.drain_completed_custody(admission)?;
        let mut prepaid = charge_whole_map(&state.records, admission, WholeMapPasses::One, 4)
            .map_err(|_| super::super::refreshed_rejoin::WalkKind::JoinedReadyRelease.denial())?;
        let key = &interest.key;
        if !state.records.get(key).is_some_and(only_joined) || prepaid.stale_owner(key)? {
            return Ok(false);
        }
        let walks: Vec<_> = state.records[key]
            .prerequisites
            .iter()
            .map(|upstream| (upstream.as_ref().clone(), 3))
            .collect();
        let cascade: Vec<_> = state.records[key]
            .prerequisites
            .iter()
            .map(|key| key.as_ref().clone())
            .collect();
        let (ready, readmission, claims, retired) = super::cleanup_price::price_cascade_mutation(
            &mut state,
            admission,
            walks,
            &cascade,
            super::super::refreshed_rejoin::WalkKind::JoinedReadyRelease,
        )?
        .complete(|priced| {
            let claims = priced.release_prerequisites(key);
            let state = priced.state_mut();
            let mut record = state.records.get_mut(key).expect("the row was checked");
            let ready = std::mem::replace(&mut record.state, DemandState::Admitted);
            let readmission = record.readmission_source.take();
            record.required_stop = None;
            record.wake.notify();
            record.finish();
            let upstream = claims.iter().map(|claim| claim.as_ref().clone()).collect();
            let retired = priced.retire_closed_rows(upstream);
            Ok((ready, readmission, claims, retired))
        });
        drop(state);
        drop((ready, readmission, claims, retired));
        Ok(true)
    }
}

/// An idle Ready one interest holds and nothing else does.
fn only_joined(record: &DemandRecord) -> bool {
    let idle_ready = matches!(&record.state, DemandState::Output(output)
        if matches!(output.advancement, WorthQueryOutputAdvancement::Idle)
            && matches!(output.checkpoint, Some(WorthQueryOutputCheckpoint::Ready(_))));
    idle_ready
        && record.interests == 1
        && record.framework_required_count == 0
        && record.prepared_prerequisite_claims == 0
        && !record.pending_cleanup_queued
        && record.performed_obligations.is_empty()
        && record.performed_source.is_none()
        && record.successor_of.is_none()
        && record.held_successor.is_none()
}
