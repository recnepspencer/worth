use std::sync::Arc;

use super::{
    DemandState, SelectedReadyReadmission, WorthQueryAcceptedOutputAuthority,
    WorthQueryOutputAdvancement, WorthQueryOutputCheckpoint, WorthQueryOutputDemandInterest,
    WorthQueryOutputDemandRegistry,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};

impl WorthQueryOutputDemandRegistry {
    /// Discharge the caller's exact selected Ready without rediscovering its
    /// receipt. All ordered work is admitted before altering registry custody.
    pub(in crate::domain_computation::primary_graph) fn finish_settlement_admitted(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        selected: &SelectedReadyReadmission,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        fn work_denial() -> WorthQueryOutputDemandDenial {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                "",
            )
        }
        admission
            .charge_external_work(4)
            .map_err(|_| work_denial())?;
        if !Arc::ptr_eq(&self.state, &interest.owner.state)
            || !selected.matches_interest(interest, admission)?
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
        for _ in 0..2 {
            state.charge_record_lookup(&interest.key, admission)?;
        }
        admission
            .charge_external_work(16)
            .map_err(|_| work_denial())?;
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
        let slot = std::mem::size_of::<super::PerformedOutputObligation>();
        let destruction = record
            .performed_obligations
            .len()
            .checked_mul(slot)
            .and_then(|work| {
                work.checked_add(std::mem::size_of::<Vec<super::PerformedOutputObligation>>() + 8)
            })
            .ok_or_else(work_denial)?;
        admission
            .charge_external_work(u64::try_from(destruction).map_err(|_| work_denial())?)
            .map_err(|_| work_denial())?;
        let record = state
            .records
            .get_mut(&interest.key)
            .expect("selected Ready row retained");
        let obligation_bytes = record.obligation_reserved_bytes();
        let obligations = std::mem::take(&mut record.performed_obligations);
        drop(state);
        // Keep the retained charge until the Vec and its owned sources die.
        drop(obligations);
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.obligation_reserved_bytes = state
            .obligation_reserved_bytes
            .saturating_sub(obligation_bytes);
        Ok(())
    }

    /// Observe an exact Ready checkpoint without advancing any other demand
    /// state. A pre-disclosure currentness probe must leave a non-Ready record
    /// untouched for the normal source-query progression.
    pub(in crate::domain_computation::primary_graph) fn peek_ready(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<super::ReadyCompletion>, WorthQueryOutputDemandDenial> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.charge_record_lookup(&interest.key, admission)?;
        let Some(record) = state.records.get(&interest.key) else {
            return Ok(None);
        };
        let DemandState::Output(output) = &record.state else {
            return Ok(None);
        };
        if !matches!(output.advancement, WorthQueryOutputAdvancement::Idle) {
            return Ok(None);
        }
        let Some(WorthQueryOutputCheckpoint::Ready(completion)) = &output.checkpoint else {
            return Ok(None);
        };
        admission.charge_external_work(1).map_err(|_| {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                "ready completion pin exhausted demand work",
            )
        })?;
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
        let record = state.records.get_mut(&interest.key).ok_or_else(|| {
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
        let released = record.release_obligations();
        state.obligation_reserved_bytes = state.obligation_reserved_bytes.saturating_sub(released);
        state.remove_required_member_if_released(&interest.key);
        Ok(())
    }
}
