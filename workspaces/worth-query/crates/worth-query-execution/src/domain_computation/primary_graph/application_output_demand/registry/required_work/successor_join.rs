//! Check that an issued required successor remains a live replacement.

use std::cmp::Ordering;
use std::sync::Arc;

use super::super::WorthQueryOutputDemandInterest;
use super::super::WorthQueryOutputDemandRegistry;
use super::selection::SelectedReadyReadmission;
use super::*;

impl WorthQueryOutputDemandRegistry {
    /// A scheduling and custody check only. The caller compares the retained
    /// predecessor Ready pin with its pre-effect pin before this join; the
    /// successor still needs its own currentness and producer authorization.
    pub(in crate::domain_computation::primary_graph) fn required_successor_is_live(
        &self,
        predecessor: &SelectedReadyReadmission,
        successor: &WorthQueryOutputDemandInterest,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        let prior_key = predecessor.membership.key.as_ref();
        let successor_key = &successor.key;

        // Issuer/Required checks, the replacement result, and the selected
        // record's interest/required-state observations are separate from
        // reading the two keys and their semantic comparison.

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

        let live = state.records.get(successor_key).is_some_and(|record| {
            record.interests != 0 && record.required_interests != 0 && record.is_required()
        });
        Ok(live && state.required_keys.contains(successor_key))
    }
}
