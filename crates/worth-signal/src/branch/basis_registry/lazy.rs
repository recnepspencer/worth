use std::sync::Arc;

use crate::branch::{
    AdmittedSignalBranchBasis, SignalBranchAdmissionLease, SignalBranchObservation,
    SignalBranchRetentionAcquisitionDenial,
};
use crate::state::SignalBranchId;

use super::acquisition::{AcquiringEntry, RegistryEntry};
use super::{
    begin_admission, finish_denied, new_basis, AcquisitionClaimGuard, AdmissionDecision,
    SignalBranchBasisRegistry,
};

impl SignalBranchBasisRegistry {
    /// Admit lazily when the caller's owner lease can only be acquired after
    /// the exact weak canonical entry has been checked. Only the claimant
    /// executes `acquire_retention`; joiners await its typed result. A live
    /// canonical entry is returned only after the owner validates that its
    /// lifecycle and retirement posture still admit reuse.
    pub(crate) fn admit_with_retention<ValidateReady, Acquire>(
        &self,
        runtime_instance_id: u64,
        definition_basis: u64,
        branch_id: SignalBranchId,
        cell_incarnation: u64,
        observation: SignalBranchObservation,
        validate_ready: ValidateReady,
        acquire_retention: Acquire,
    ) -> Result<AdmittedSignalBranchBasis, SignalBranchRetentionAcquisitionDenial>
    where
        ValidateReady: FnOnce(
            &AdmittedSignalBranchBasis,
        ) -> Result<(), SignalBranchRetentionAcquisitionDenial>,
        Acquire:
            FnOnce() -> Result<SignalBranchAdmissionLease, SignalBranchRetentionAcquisitionDenial>,
    {
        let key = super::key::SignalBranchBasisRegistryKey::new(
            runtime_instance_id,
            definition_basis,
            branch_id,
            cell_incarnation,
            &observation,
        );
        let decision = {
            let mut state = self.lock_state();
            begin_admission(&mut state, &key)?
        };
        match decision {
            AdmissionDecision::Ready(existing) => {
                let existing = AdmittedSignalBranchBasis::from_inner(existing);
                validate_ready(&existing)?;
                Ok(existing)
            }
            AdmissionDecision::Join(completion) => completion.wait(),
            AdmissionDecision::OwnerReentry => {
                Err(SignalBranchRetentionAcquisitionDenial::OwnerReentry)
            }
            AdmissionDecision::Claim {
                reservation_id,
                completion,
            } => {
                let mut claim = AcquisitionClaimGuard::new(
                    &self.state,
                    key.clone(),
                    reservation_id,
                    Arc::clone(&completion),
                );
                let retention = acquire_retention();
                let retention = match retention {
                    Ok(retention) => retention,
                    Err(denial) => {
                        finish_denied(
                            &self.state,
                            &key,
                            reservation_id,
                            &completion,
                            denial.clone(),
                        );
                        claim.disarm();
                        return Err(denial);
                    }
                };
                let result = {
                    let mut state = self.lock_state();
                    if matches!(
                        state.entries.get(&key),
                        Some(RegistryEntry::Acquiring(AcquiringEntry {
                            reservation_id: current,
                            ..
                        })) if *current == reservation_id
                    ) {
                        new_basis(
                            &mut state,
                            &self.state,
                            key.clone(),
                            observation,
                            branch_id,
                            retention,
                        )
                    } else {
                        drop(state);
                        drop(retention);
                        Err(SignalBranchRetentionAcquisitionDenial::OwnerOperationPanicked)
                    }
                };
                match result {
                    Ok(basis) => {
                        completion.finish(Ok(basis.clone()));
                        claim.disarm();
                        Ok(basis)
                    }
                    Err(denial) => {
                        finish_denied(
                            &self.state,
                            &key,
                            reservation_id,
                            &completion,
                            denial.clone(),
                        );
                        claim.disarm();
                        Err(denial)
                    }
                }
            }
        }
    }
}
