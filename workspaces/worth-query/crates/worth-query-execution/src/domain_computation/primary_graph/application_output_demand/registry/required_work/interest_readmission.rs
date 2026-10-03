//! The caller's issued interest joins its exact Ready and readmission row.

use super::super::{
    DemandState, WorthQueryOutputAdvancement, WorthQueryOutputCheckpoint,
    WorthQueryOutputDemandInterest,
};
use super::selection::{charge_required_key_lookup, SelectedReadyReadmission};
use super::*;

impl WorthQueryOutputDemandRegistry {
    /// Keep the issued interest, Ready cell, source and producer in one registry
    /// pin. A Ready observation alone cannot be paired with another row's
    /// source or installed executor during a required wave.
    pub(in crate::domain_computation::primary_graph) fn interest_ready_readmission(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<SelectedReadyReadmission>, WorthQueryOutputDemandDenial> {
        // Owner identity, the interest counters, Ready state and three Arc
        // retains precede the lookups. Denial subjects are borrowed statics.
        admission
            .charge_external_work(17)
            .map_err(|_| empty_work_denial())?;
        if !Arc::ptr_eq(&self.state, &interest.owner.state) {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignDemand,
                "required output interest belongs to another registry",
            ));
        }
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.charge_record_lookup(&interest.key, admission)?;
        charge_required_key_lookup(&state, &interest.key, admission)?;
        if !state.required_keys.contains(&interest.key) {
            return Ok(None);
        }
        let Some(record) = state.records.get(&interest.key) else {
            return Ok(None);
        };
        if record.interests == 0
            || (interest.requires_output && record.required_interests == 0)
            || !record.is_required()
        {
            return Ok(None);
        }
        let DemandState::Output(output) = &record.state else {
            return Ok(None);
        };
        if !matches!(output.advancement, WorthQueryOutputAdvancement::Idle) {
            return Ok(None);
        }
        let (
            Some(WorthQueryOutputCheckpoint::Ready(completion)),
            Some(readmission),
            Some(membership),
        ) = (
            &output.checkpoint,
            record.readmission_source.as_ref(),
            record.work_membership.as_ref(),
        )
        else {
            return Ok(None);
        };
        Ok(Some(SelectedReadyReadmission {
            completion: completion.clone(),
            readmission: Arc::clone(readmission),
            membership: Arc::clone(membership),
        }))
    }
}

fn empty_work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, "")
}
