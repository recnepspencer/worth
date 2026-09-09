use super::*;

impl WorthUiPresentationAsyncOwner {
    pub fn reject_before_effects(
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
        if self
            .settling
            .get(&key)
            .is_some_and(|pending| pending.nonce == receipt.nonce)
        {
            return Err(WorthUiPresentationSettlementDenial::SettlementAlreadyBegan);
        }
        if self
            .superseded_pending
            .get(&key)
            .is_some_and(|pending| pending.nonce == receipt.nonce)
        {
            return self.reject_superseded_before_effects(key);
        }
        let mut pending = self
            .pending
            .remove(&key)
            .filter(|pending| pending.nonce == receipt.nonce)
            .ok_or(WorthUiPresentationSettlementDenial::InvalidPendingReceipt)?;
        if let Err(denial) = self.advance_rejection(&mut pending) {
            self.pending.insert(key, pending);
            return Err(denial);
        }
        self.discard_pending_transition(key);
        self.active_keys.remove(&key);
        Ok(())
    }

    fn reject_superseded_before_effects(
        &mut self,
        key: PresentationAdmissionKey,
    ) -> Result<(), WorthUiPresentationSettlementDenial> {
        let pending = self
            .superseded_pending
            .remove(&key)
            .expect("validated superseded receipt remains retained");
        if pending
            .admission
            .close_query_live_view(&mut self.workspace)
            .is_err()
        {
            self.superseded_pending.insert(key, pending);
            return Err(WorthUiPresentationSettlementDenial::Progress(
                WorthUiPresentationSettlementStop::QueryClose,
            ));
        }
        self.active_keys.remove(&key);
        Ok(())
    }

    pub(super) fn advance_rejection(
        &mut self,
        pending: &mut PendingPresentationAdmission,
    ) -> Result<(), WorthUiPresentationSettlementDenial> {
        if !pending.rejection.query_denied {
            pending
                .admission
                .admit_denial_before_effects(&mut self.workspace)
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
            if observation.posture() != WorthUiPresentationAsyncPosture::Failed {
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
