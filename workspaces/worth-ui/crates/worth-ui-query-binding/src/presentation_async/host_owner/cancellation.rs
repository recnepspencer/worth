use super::*;

impl WorthUiPresentationAsyncOwner {
    pub fn cancel_after_effects_may_have_begun(
        &mut self,
        receipt: &WorthUiPresentationPendingReceipt,
        observation: WorthUiPresentationCancellationEffectsObservation,
    ) -> Result<WorthUiPresentationUnresolvedReceipt, WorthUiPresentationSettlementDenial> {
        self.admit_effects_indeterminate(receipt, observation.into_indeterminate())
    }

    pub fn cancel_after_effects_may_have_begun_requiring_reconstruction(
        &mut self,
        receipt: &WorthUiPresentationPendingReceipt,
        observation: WorthUiPresentationCancellationEffectsObservation,
    ) -> Result<WorthUiPresentationRecoveryRequiredReceipt, WorthUiPresentationSettlementDenial>
    {
        let unresolved = self.cancel_after_effects_may_have_begun(receipt, observation)?;
        Ok(self.mark_reconstruction_required(&unresolved))
    }

    pub fn cancel_before_effects(
        &mut self,
        receipt: &WorthUiPresentationPendingReceipt,
    ) -> Result<(), WorthUiPresentationSettlementDenial> {
        if !correspondence::is_correspondence_authority(
            &self.correspondence_authority,
            &receipt.authority,
        ) {
            return Err(WorthUiPresentationSettlementDenial::ForeignPendingReceiptAuthority);
        }
        let key = PresentationAdmissionKey {
            attempt: receipt.attempt,
            binding: receipt.binding,
        };
        if self.settling.contains_key(&key) {
            return Err(WorthUiPresentationSettlementDenial::SettlementAlreadyBegan);
        }
        let mut pending = self
            .pending
            .remove(&key)
            .filter(|pending| pending.nonce == receipt.nonce)
            .ok_or(WorthUiPresentationSettlementDenial::InvalidPendingReceipt)?;
        if let Err(denial) = self.advance_cancellation(&mut pending) {
            self.pending.insert(key, pending);
            return Err(denial);
        }
        self.discard_pending_transition(key);
        self.record_transition(WorthUiPresentationTransitionKind::Cancelled, key);
        self.active_keys.remove(&key);
        Ok(())
    }

    fn advance_cancellation(
        &mut self,
        pending: &mut PendingPresentationAdmission,
    ) -> Result<(), WorthUiPresentationSettlementDenial> {
        if !pending.rejection.query_denied {
            pending
                .admission
                .admit_cancellation_before_effects(&mut self.workspace)
                .map_err(|_| {
                    WorthUiPresentationSettlementDenial::Progress(
                        WorthUiPresentationSettlementStop::QueryCompletion,
                    )
                })?;
            pending.rejection.query_denied = true;
        }
        if !pending.rejection.query_denial_observed {
            let observation = pending
                .admission
                .observation(&self.workspace)
                .map_err(|_| {
                    WorthUiPresentationSettlementDenial::Progress(
                        WorthUiPresentationSettlementStop::QueryObservation,
                    )
                })?;
            if observation.posture() != WorthUiPresentationAsyncPosture::Cancelled {
                return Err(WorthUiPresentationSettlementDenial::Progress(
                    WorthUiPresentationSettlementStop::UnexpectedQueryPosture,
                ));
            }
            pending.rejection.query_denial_observed = true;
        }
        if !pending.rejection.query_closed {
            pending
                .admission
                .close_query_live_view(&mut self.workspace)
                .map_err(|_| {
                    WorthUiPresentationSettlementDenial::Progress(
                        WorthUiPresentationSettlementStop::QueryClose,
                    )
                })?;
            pending.rejection.query_closed = true;
        }
        Ok(())
    }
}
