//! Publication and checkpoint backing are admitted together before release effects.
use super::super::PhysicalCurrentRootOwner;
use super::*;

impl PhysicalCurrentRootOwner {
    /// Released-generation custody needs a selected tag7 basis before any
    /// subsequent checkpoint. Mark it while the admitted reclaim attempt is
    /// still pre-effect; no ordinary legacy capture may bypass this handoff.
    pub(in crate::physical_runtime) fn require_release_certificate_for_attempt(
        &self,
        attempt: &PhysicalReclaimAttempt,
    ) -> Result<(), ReleaseCertificateCapacityDenial> {
        let mut state = self.lock_publication_state();
        let fence = self.lock_reclaim();
        if !fence.as_ref().is_some_and(|active| {
            active.matches_attempt(attempt.bytes()) && active.is_pre_effect_payload_drop()
        }) {
            return Err(ReleaseCertificateCapacityDenial::ReclaimFenceMismatch);
        }
        let fence_bytes = fence
            .as_ref()
            .and_then(ReclaimFenceState::owned_heap_bytes)
            .ok_or(ReleaseCertificateCapacityDenial::CapacityExhausted)?;
        let ledger = state
            .release_ledger
            .selected_mut()
            .ok_or(ReleaseCertificateCapacityDenial::SelectedLedgerUnavailable)?;
        // Fund the pending custody Box before require_release_certificate allocates it.
        ledger.prepare_control_backing(
            &self.release_allocation,
            self.recovery_allocation,
            fence_bytes,
        )?;
        if !state
            .checkpoint_custody
            .require_release_certificate(attempt.bytes())
        {
            return Err(ReleaseCertificateCapacityDenial::ReclaimFenceMismatch);
        }
        Ok(())
    }

    /// Admits the complete release closure and acquires mandatory publication
    /// backing before the manifest, reservation, descriptor, or root effects.
    pub(in crate::physical_runtime) fn reserve_release_certificate_capacity(
        &self,
        attempt: &PhysicalReclaimAttempt,
        key: ReleaseCustodyHeadKeyV1,
        needed_records: u16,
        worst_case_encoded_bytes: u32,
        head_charge: ReleaseHeadCapacityCharge,
    ) -> Result<ReleaseCertificateCapacityLease, ReleaseCertificateCapacityDenial> {
        let mut state = self.lock_publication_state();
        let mut fence = self.lock_reclaim();
        let active = fence
            .as_mut()
            .filter(|active| active.matches_attempt(attempt.bytes()))
            .ok_or(ReleaseCertificateCapacityDenial::ReclaimFenceMismatch)?;
        if !active.is_pre_effect_payload_drop() || active.release_certificate_pending.is_some() {
            return Err(ReleaseCertificateCapacityDenial::ReclaimFenceMismatch);
        }
        let anchored = state.current_root.tier_epoch_anchor().is_some();
        let current_head_root = state.current_root.release_custody_head_root();
        let ledger = state
            .release_ledger
            .selected_mut()
            .ok_or(ReleaseCertificateCapacityDenial::SelectedLedgerUnavailable)?;
        if !ledger.admits_worst_case(anchored, needed_records, worst_case_encoded_bytes) {
            return Err(ReleaseCertificateCapacityDenial::CapacityExhausted);
        }
        if ledger.effective_heads.root() != current_head_root {
            return Err(ReleaseCertificateCapacityDenial::SelectedFactMismatch);
        }
        let closure_bytes = ledger.head_closure_bytes(key, head_charge)?;
        let fence_bytes = active.owned_heap_bytes().ok_or_else(|| {
            ReleaseCertificateCapacityDenial::Resident(
                crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial::SizeOverflow {
                    admitted: self.recovery_allocation.byte_limit(),
                },
            )
        })?;
        let combined = ledger
            .retained_requirement(self.recovery_allocation, closure_bytes, fence_bytes)?
            .checked_add(ledger.checkpoint_backing_requirement(Some(key), anchored)?)
            .ok_or(ReleaseCertificateCapacityDenial::CapacityExhausted)?;
        if combined > self.recovery_allocation.byte_limit() {
            return Err(ReleaseCertificateCapacityDenial::Resident(
                crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial::BudgetExceeded {
                    required: combined,
                    admitted: self.recovery_allocation.byte_limit(),
                },
            ));
        }
        let root_frame = ledger.prepare_publication_backing(
            &self.release_allocation,
            self.recovery_allocation,
            closure_bytes,
            fence_bytes,
            key,
        )?;
        ledger.prepare_checkpoint_backing(
            &self.release_allocation,
            self.recovery_allocation,
            Some(key),
            anchored,
        )?;
        active.release_certificate_pending = Some(ReleaseCertificatePending {
            key,
            root_frame,
            needed_records,
            worst_case_encoded_bytes,
            effect_may_exist: false,
        });
        Ok(ReleaseCertificateCapacityLease {
            fence: attempt.certificate_fence(),
            attempt: attempt.bytes(),
        })
    }
}
