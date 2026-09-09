use super::*;

impl WorthUiPresentationAsyncOwner {
    pub fn reject_admission_recovery_before_effects(
        &mut self,
        receipt: &WorthUiPresentationAdmissionRecovery,
    ) -> Result<(), WorthUiPresentationSettlementDenial> {
        let (key, nonce, cleanup_phase) = self.validate_admission_recovery(receipt)?;
        if cleanup_phase && self.runtime_cleanup_matches(key, nonce) {
            return self.resume_runtime_admission_cleanup(key);
        }
        let mut pending = self
            .pending
            .remove(&key)
            .filter(|pending| pending.nonce == nonce)
            .ok_or(WorthUiPresentationSettlementDenial::InvalidPendingReceipt)?;
        if let Err(denial) = self.advance_rejection(&mut pending) {
            self.pending.insert(key, pending);
            return Err(denial);
        }
        self.active_keys.remove(&key);
        Ok(())
    }

    fn validate_admission_recovery(
        &self,
        receipt: &WorthUiPresentationAdmissionRecovery,
    ) -> Result<(PresentationAdmissionKey, u64, bool), WorthUiPresentationSettlementDenial> {
        let (authority, attempt, binding, nonce, cleanup_phase) = match receipt {
            WorthUiPresentationAdmissionRecovery::Incomplete(receipt) => (
                &receipt.authority,
                receipt.attempt,
                receipt.binding,
                receipt.nonce,
                false,
            ),
            WorthUiPresentationAdmissionRecovery::Cleanup(receipt) => (
                &receipt.authority,
                receipt.attempt,
                receipt.binding,
                receipt.nonce,
                true,
            ),
        };
        if !correspondence::is_correspondence_authority(&self.correspondence_authority, authority) {
            return Err(WorthUiPresentationSettlementDenial::ForeignPendingReceiptAuthority);
        }
        Ok((
            PresentationAdmissionKey { attempt, binding },
            nonce,
            cleanup_phase,
        ))
    }

    fn runtime_cleanup_matches(&self, key: PresentationAdmissionKey, nonce: u64) -> bool {
        self.runtime_cleanups
            .get(&key)
            .is_some_and(|cleanup| cleanup.nonce == nonce)
    }

    fn resume_runtime_admission_cleanup(
        &mut self,
        key: PresentationAdmissionKey,
    ) -> Result<(), WorthUiPresentationSettlementDenial> {
        let mut cleanup = self
            .runtime_cleanups
            .remove(&key)
            .expect("validated runtime cleanup remains retained");
        if let Err(denial) = cleanup.cleanup.resume(&mut self.workspace) {
            self.runtime_cleanups.insert(key, cleanup);
            let stop = match denial {
                super::super::runtime_bridge::WorthUiPresentationRuntimeCleanupDenial::Query(_) => {
                    WorthUiPresentationRuntimeCleanupStop::Query
                }
            };
            return Err(WorthUiPresentationSettlementDenial::RuntimeCleanup(stop));
        }
        self.active_keys.remove(&key);
        Ok(())
    }
}
