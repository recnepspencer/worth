//! Check that an issued required successor remains a live replacement.

use std::{cmp::Ordering, sync::Arc};

use super::super::{
    WorthQueryOutputDemandInterest, WorthQueryOutputDemandKey, WorthQueryOutputDemandRegistry,
};
use super::selection::{charge_required_key_lookup, SelectedReadyReadmission};
use super::*;

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
            .charge_external_work(4)
            .map_err(|_| empty_work_denial())?;
        let prior_key = predecessor.membership.key.as_ref();
        let successor_key = &successor.key;
        admission
            .charge_external_work(replacement_comparison_work(prior_key, successor_key)?)
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
    prior: &WorthQueryOutputDemandKey,
    successor: &WorthQueryOutputDemandKey,
) -> Result<u64, WorthQueryOutputDemandDenial> {
    // The key formula covers declared family, profile and producer text.
    // replacement_order also reads same_occurrence
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
    let work = super::super::key_comparison::comparison_work([prior, successor])
        .ok_or_else(empty_work_denial)?
        .total
        .checked_add(1)
        .and_then(|n| n.checked_add(occurrence.checked_mul(4)?))
        .and_then(|n| n.checked_add(64 + 2 * std::mem::size_of::<u64>() + 4))
        .ok_or_else(empty_work_denial)?;
    u64::try_from(work).map_err(|_| empty_work_denial())
}

fn empty_work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, "")
}

#[cfg(test)]
mod comparison_tests {
    #[test]
    fn successor_join_prices_both_declared_keys_and_occurrence_reads() {
        let (short, long) = super::super::super::key_comparison::fixtures::keys();
        let occurrence = 64
            + std::mem::size_of::<worth_relational::facade::identity::EntityId>()
            + std::mem::size_of::<worth_runtime_world::facade::ProductBranchIncarnation>();
        let expected = (4 + 1 + 1 + 10)
            + (4 + 15 + 16 + 10)
            + 1
            + 4 * occurrence
            + 64
            + 2 * std::mem::size_of::<u64>()
            + 4;
        assert_eq!(
            super::replacement_comparison_work(&short, &long).unwrap(),
            expected as u64
        );
    }
}
