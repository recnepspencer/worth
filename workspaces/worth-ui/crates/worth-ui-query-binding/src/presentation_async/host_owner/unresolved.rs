use super::*;

impl WorthUiPresentationAsyncOwner {
    pub fn admit_effects_indeterminate(
        &mut self,
        receipt: &WorthUiPresentationPendingReceipt,
        observation: WorthUiPresentationEffectsIndeterminateObservation,
    ) -> Result<WorthUiPresentationUnresolvedReceipt, WorthUiPresentationSettlementDenial> {
        let (authority, receipt_authority, pending_nonce, payload_byte_len) =
            observation.into_parts();
        if !correspondence::is_correspondence_authority(
            &self.correspondence_authority,
            &receipt.authority,
        ) {
            return Err(WorthUiPresentationSettlementDenial::ForeignPendingReceiptAuthority);
        }
        if !correspondence::is_correspondence_authority(&self.correspondence_authority, &authority)
        {
            return Err(WorthUiPresentationSettlementDenial::ForeignCompletionAuthority);
        }
        if !correspondence::is_correspondence_authority(&receipt.authority, &receipt_authority) {
            return Err(WorthUiPresentationSettlementDenial::CompletionReceiptMismatch);
        }
        if pending_nonce != receipt.nonce {
            return Err(WorthUiPresentationSettlementDenial::CompletionReceiptMismatch);
        }
        let key = PresentationAdmissionKey {
            attempt: receipt.attempt,
            binding: receipt.binding,
        };
        let (mut pending, already_committed) = match self
            .unresolved
            .remove(&key)
            .filter(|pending| pending.nonce == receipt.nonce)
        { Some(pending) => {
            (pending, true)
        } _ => {
            (
                self.pending
                    .remove(&key)
                    .filter(|pending| pending.nonce == receipt.nonce)
                    .ok_or(WorthUiPresentationSettlementDenial::InvalidPendingReceipt)?,
                false,
            )
        }};
        if pending.settlement.completion_progress.is_none() {
            match pending
                .admission
                .begin_owner_effects_indeterminate(&mut self.workspace, payload_byte_len)
            {
                Ok(progress) => pending.settlement.completion_progress = Some(progress),
                Err(_) => {
                    if already_committed {
                        self.unresolved.insert(key, pending);
                    } else {
                        self.pending.insert(key, pending);
                    }
                    return Err(WorthUiPresentationSettlementDenial::Progress(
                        WorthUiPresentationSettlementStop::QueryCompletion,
                    ));
                }
            }
        }
        let unresolved = match pending.admission.resume_completion(
            &mut self.workspace,
            pending
                .settlement
                .completion_progress
                .as_mut()
                .expect("committed indeterminate completion retains resumable progress"),
        ) {
            Ok(unresolved) => unresolved,
            Err(_) => {
                self.unresolved.insert(key, pending);
                return Err(WorthUiPresentationSettlementDenial::Progress(
                    WorthUiPresentationSettlementStop::QueryCompletion,
                ));
            }
        };
        if unresolved.observation().posture() != WorthUiPresentationAsyncPosture::Unresolved {
            self.unresolved.insert(key, pending);
            return Err(WorthUiPresentationSettlementDenial::Progress(
                WorthUiPresentationSettlementStop::UnexpectedQueryPosture,
            ));
        }
        let result = WorthUiPresentationUnresolvedReceipt {
            authority: std::sync::Arc::clone(&self.correspondence_authority),
            attempt: key.attempt,
            binding: key.binding,
            nonce: pending.nonce,
            observation: unresolved.observation(),
        };
        self.unresolved.insert(key, pending);
        self.record_transition(WorthUiPresentationTransitionKind::Unresolved, key);
        Ok(result)
    }

    pub fn require_reconstruction(
        &mut self,
        receipt: &WorthUiPresentationUnresolvedReceipt,
    ) -> Result<WorthUiPresentationRecoveryRequiredReceipt, WorthUiPresentationSettlementDenial>
    {
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
        let receipt_is_current = self
            .unresolved
            .get(&key)
            .filter(|pending| pending.nonce == receipt.nonce)
            .is_some();
        if !receipt_is_current {
            return Err(WorthUiPresentationSettlementDenial::InvalidPendingReceipt);
        }
        Ok(self.mark_reconstruction_required(receipt))
    }

    pub fn admit_effects_indeterminate_requiring_reconstruction(
        &mut self,
        receipt: &WorthUiPresentationPendingReceipt,
        observation: WorthUiPresentationEffectsIndeterminateObservation,
    ) -> Result<WorthUiPresentationRecoveryRequiredReceipt, WorthUiPresentationSettlementDenial>
    {
        let unresolved = self.admit_effects_indeterminate(receipt, observation)?;
        Ok(self.mark_reconstruction_required(&unresolved))
    }

    pub(super) fn mark_reconstruction_required(
        &mut self,
        receipt: &WorthUiPresentationUnresolvedReceipt,
    ) -> WorthUiPresentationRecoveryRequiredReceipt {
        let key = PresentationAdmissionKey {
            attempt: receipt.attempt,
            binding: receipt.binding,
        };
        let pending = self
            .unresolved
            .get_mut(&key)
            .filter(|pending| pending.nonce == receipt.nonce)
            .expect("owner-issued unresolved receipt remains retained until reconstruction");
        pending.recovery_required = true;
        self.record_transition(WorthUiPresentationTransitionKind::RecoveryRequired, key);
        WorthUiPresentationRecoveryRequiredReceipt {
            attempt: key.attempt,
            binding: key.binding,
            observation: receipt.observation,
        }
    }
}
