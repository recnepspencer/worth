//! Check that an issued required successor remains a live replacement.

use std::{cmp::Ordering, sync::Arc};

use super::super::{WorthQueryOutputDemandInterest, WorthQueryOutputDemandRegistry};
use super::selection::{charge_required_key_lookup, SelectedReadyReadmission};
use super::*;

const TERMINAL_SUBJECT_BYTES: u64 = 64;

impl WorthQueryOutputDemandRegistry {
    /// A scheduling and custody check only. The caller compares the retained
    /// predecessor Ready pin with its pre-effect pin before this join; the
    /// successor still needs its own currentness and producer authorization.
    pub(in crate::domain_computation::primary_graph) fn required_successor_is_live(
        &self,
        predecessor: &SelectedReadyReadmission,
        successor: &WorthQueryOutputDemandInterest,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        admission
            .admit_read_scratch(TERMINAL_SUBJECT_BYTES)
            .map_err(empty_preflight_denial)?;
        admission
            .charge_external_work(TERMINAL_SUBJECT_BYTES + 4)
            .map_err(|_| empty_work_denial())?;
        let prior_key = predecessor.membership.key.as_ref();
        let successor_key = &successor.key;
        admission
            .charge_external_work(replacement_comparison_work(
                prior_key.producer.len(),
                successor_key.producer.len(),
            )?)
            .map_err(|_| empty_work_denial())?;
        // Issuer/Required checks, the replacement result, and the selected
        // record's interest/required-state observations are separate from
        // reading the two keys and their semantic comparison.
        admission
            .charge_external_work(12)
            .map_err(|_| empty_work_denial())?;

        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !Arc::ptr_eq(&self.state, &successor.owner.state) || !successor.requires_output {
            return Ok(false);
        }
        if !matches!(
            prior_key.replacement_order(successor_key),
            Some(Ordering::Less | Ordering::Equal)
        ) {
            return Ok(false);
        }
        state.charge_record_lookup(successor_key, admission)?;
        charge_required_key_lookup(&state, successor_key, admission)?;
        let live = state.records.get(successor_key).is_some_and(|record| {
            record.interests != 0 && record.required_interests != 0 && record.is_required()
        });
        Ok(live && state.required_keys.contains(successor_key))
    }
}

fn replacement_comparison_work(
    prior_len: usize,
    successor_len: usize,
) -> Result<u64, WorthQueryOutputDemandDenial> {
    // replacement_order compares both producer texts, then same_occurrence
    // twice on the successful path. Each pass reads both operands' query,
    // parameters, root and occurrence. It finally compares two 32-byte
    // semantic identities and two u64 generations.
    let occurrence = 64usize
        .checked_add(std::mem::size_of::<
            worth_relational::facade::identity::EntityId,
        >())
        .and_then(|n| {
            n.checked_add(std::mem::size_of::<
                worth_runtime_world::facade::ProductBranchIncarnation,
            >())
        })
        .ok_or_else(empty_work_denial)?;
    let work = prior_len
        .checked_add(successor_len)
        .and_then(|n| n.checked_add(1))
        .and_then(|n| n.checked_add(occurrence.checked_mul(4)?))
        .and_then(|n| n.checked_add(64 + 2 * std::mem::size_of::<u64>() + 4))
        .ok_or_else(empty_work_denial)?;
    u64::try_from(work).map_err(|_| empty_work_denial())
}

fn empty_work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, "")
}

fn empty_preflight_denial(
    stop: worth_relational::facade::mvcc::CompanionPreflightStop,
) -> WorthQueryOutputDemandDenial {
    use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;
    let kind = match stop {
        Stop::WorkExhausted { .. } | Stop::WorkCounterOverflow => {
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
        }
        _ => WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
    };
    WorthQueryOutputDemandDenial::new(kind, "")
}
