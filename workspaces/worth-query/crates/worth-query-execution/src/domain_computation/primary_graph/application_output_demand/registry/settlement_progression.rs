use std::sync::Arc;

use super::DemandState;
use super::SelectedReadyReadmission;
use super::WorthQueryAcceptedOutputAuthority;
use super::WorthQueryOutputCheckpoint;
use super::WorthQueryOutputDemandInterest;
use super::WorthQueryOutputDemandRegistry;

use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind;

impl WorthQueryOutputDemandRegistry {
    /// Discharge the caller's exact selected Ready without rediscovering its
    /// receipt. All ordered work is admitted before altering registry custody.
    pub(in crate::domain_computation::primary_graph) fn finish_settlement_admitted(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        selected: &SelectedReadyReadmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        if !Arc::ptr_eq(&self.state, &interest.owner.state)
            || !selected.matches_interest(interest)?
        {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignDemand,
                "",
            ));
        }
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // The selected pin and still-live interest keep the required member.
        // Pay the immutable Ready lookup and mutable obligation lookup.

        let record = state.records.get(&interest.key).ok_or_else(|| {
            WorthQueryOutputDemandDenial::new(WorthQueryOutputDemandDenialKind::Closed, "")
        })?;
        if record.interests == 0
            || (interest.requires_output && record.required_interests == 0)
            || !selected.matches_ready_record(record)
        {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::Superseded,
                "",
            ));
        }

        let mut record = state
            .records
            .get_mut(&interest.key)
            .expect("selected Ready row retained");
        let obligations = std::mem::take(&mut record.performed_obligations);
        record.finish();
        drop(state);
        drop(obligations);
        Ok(())
    }

    /// Observe an exact Ready checkpoint without advancing any other demand
    /// state. A pre-disclosure currentness probe must leave a non-Ready record
    /// untouched for the normal source-query progression.
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn peek_ready(
        &self,
        interest: &WorthQueryOutputDemandInterest,
    ) -> Result<Option<super::ReadyCompletion>, WorthQueryOutputDemandDenial> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let Some(record) = state.records.get(&interest.key) else {
            return Ok(None);
        };
        let DemandState::Output(output) = &record.state else {
            return Ok(None);
        };
        if !matches!(
            output.advancement(),
            super::WorthQueryOutputAdvancement::Idle
        ) {
            return Ok(None);
        }
        let Some(WorthQueryOutputCheckpoint::Ready(completion)) = &output.checkpoint else {
            return Ok(None);
        };

        Ok(Some(completion.clone()))
    }

    /// Discharge only the performed obligation whose accepted publication is
    /// still the current Ready checkpoint of this exact demand record.
    pub(in crate::domain_computation::primary_graph) fn finish_settlement(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        accepted: &WorthQueryAcceptedOutputAuthority,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut record = state.records.get_mut(&interest.key).ok_or_else(|| {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::Closed,
                "settled output lost its demand record",
            )
        })?;
        let same_ready = matches!(
            &record.state,
            DemandState::Output(output)
                if matches!(
                    &output.checkpoint,
                    Some(WorthQueryOutputCheckpoint::Ready(completion))
                        if completion.authority.is_same_settlement_authority(accepted)
                )
        );
        if !same_ready {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::Superseded,
                "output changed before its settlement was accepted",
            ));
        }
        record.release_obligations();
        record.finish();
        super::post_commit::release_settled_member(&mut state, &interest.key);
        Ok(())
    }
}
