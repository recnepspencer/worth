use worth_proof::TransitionOutcome;

use crate::branch::{
    AdmittedSignalBranchBasis, AdmittedSignalBranchSnapshot, PlannedSignalBranchRetirement,
    PlannedSignalBranchRetirementBatch, SignalBranchBasisDescriptor,
    SignalBranchBasisObservationDenial, SignalBranchBasisReadmissionDenial,
    SignalBranchRetentionTerminalCounts, SignalBranchRetirementBatchDenial,
    SignalBranchRetirementBatchReceipt, SignalBranchRetirementDenial, SignalBranchRetirementReason,
    SignalBranchRetirementReceipt,
};
use crate::state::{SignalBranchHandle, SignalSnapshotId};

use super::basis_port::denial_mapping::{
    map_basis_admission_denial, map_observation_admission_denial,
    map_observation_readmission_denial, map_observation_retention_denial,
    map_readmission_retention_denial,
};
use super::basis_port::descriptor_validation::compare_descriptor_with_observation;
use super::lifecycle_port::map_admission_denial as map_retirement_admission_denial;
use super::owner::basis::map_basis_registry_denial;
use super::owner::retirement_reservation::map_retirement_registry_denial;
use super::owner_metadata::SignalOwnerMetadataAuthorizationDenial;
use super::{SignalBranchRegistryDenial, SignalOwner, SignalOwnerRoot, SignalOwnerUnavailable};

impl<D, I, T> SignalOwnerRoot<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn observe_legacy_branch(
        &self,
        branch: SignalBranchHandle,
    ) -> Result<AdmittedSignalBranchBasis, SignalBranchBasisObservationDenial> {
        let owner = self
            .upgrade_sealed_owner()
            .map_err(SignalBranchBasisObservationDenial::OwnerUnavailable)?;
        let admission = owner.admit().map_err(map_observation_admission_denial)?;
        let branch_id = branch.id;
        let cell = owner
            .lookup_cell(&admission, branch_id)
            .map_err(|denial| map_basis_registry_denial(denial, branch_id))?;
        let observation = cell.observe_exact(&admission)?;
        owner
            .admit_canonical_basis_with_retention(
                &admission,
                observation,
                branch_id,
                cell.incarnation().get(),
                || owner.acquire_admitted_retention(&admission, branch_id),
            )
            .map_err(|denial| {
                map_observation_retention_denial(&owner, &admission, denial, branch_id)
            })
    }

    pub(crate) fn readmit_legacy_descriptor(
        &self,
        descriptor: SignalBranchBasisDescriptor,
    ) -> Result<AdmittedSignalBranchBasis, SignalBranchBasisReadmissionDenial> {
        let owner = self
            .upgrade_sealed_owner()
            .map_err(SignalBranchBasisReadmissionDenial::OwnerUnavailable)?;
        let branch_id = descriptor.owner_branch_id();
        owner.validate_managed_basis_descriptor(&descriptor, branch_id)?;
        let admission = owner.admit().map_err(map_basis_admission_denial)?;
        let cell = match owner.lookup_cell(&admission, branch_id) {
            Ok(cell) => cell,
            Err(SignalBranchRegistryDenial::UnknownBranch(_)) => {
                return match owner.metadata.retirement_receipt(&admission, branch_id) {
                    Ok(Some(_)) => {
                        Err(SignalBranchBasisReadmissionDenial::RetiredBranch { branch_id })
                    }
                    Ok(None) => {
                        Err(SignalBranchBasisReadmissionDenial::UnknownBranch { branch_id })
                    }
                    Err(denial) => Err(map_readmission_metadata_denial(denial, branch_id)),
                }
            }
            Err(denial) => {
                let denial = map_basis_registry_denial(denial, branch_id);
                return Err(map_observation_readmission_denial(
                    &owner, &admission, denial, branch_id,
                ));
            }
        };
        if let Some(snapshot_id) = descriptor
            .observation()
            .target()
            .as_basis()
            .and_then(|target| target.snapshot_id())
            .map(SignalSnapshotId)
        {
            let available = owner
                .metadata
                .has_snapshot_state(&admission, branch_id, snapshot_id)
                .map_err(|denial| map_readmission_metadata_denial(denial, branch_id))?;
            if !available {
                return Err(SignalBranchBasisReadmissionDenial::UnavailableSnapshot {
                    branch_id,
                    snapshot_id,
                });
            }
        }
        let observation = cell.observe_exact(&admission).map_err(|denial| {
            map_observation_readmission_denial(&owner, &admission, denial, branch_id)
        })?;
        compare_descriptor_with_observation(&descriptor, &observation)?;
        owner
            .admit_canonical_basis_with_retention(
                &admission,
                observation,
                branch_id,
                cell.incarnation().get(),
                || owner.acquire_admitted_retention(&admission, branch_id),
            )
            .map_err(|denial| {
                map_readmission_retention_denial(&owner, &admission, denial, branch_id)
            })
    }

    pub(crate) fn plan_legacy_retirement(
        &self,
        branch: SignalBranchHandle,
        expected: AdmittedSignalBranchBasis,
        reason: SignalBranchRetirementReason,
    ) -> TransitionOutcome<PlannedSignalBranchRetirement, SignalBranchRetirementDenial> {
        let owner = match self.upgrade_sealed_owner() {
            Ok(owner) => owner,
            Err(unavailable) => {
                return TransitionOutcome::denied(SignalBranchRetirementDenial::OwnerUnavailable(
                    unavailable,
                ))
            }
        };
        let admission = match owner.admit() {
            Ok(admission) => admission,
            Err(denial) => {
                return TransitionOutcome::denied(map_retirement_admission_denial(denial))
            }
        };
        if branch.id != expected.owner_branch_id() {
            if let Err(denial) = owner.lookup_cell(&admission, branch.id) {
                return TransitionOutcome::denied(map_retirement_registry_denial(
                    denial, branch.id,
                ));
            }
            return TransitionOutcome::denied(SignalBranchRetirementDenial::CanonicalBasisMismatch);
        }
        owner.plan_retirement_exact(&admission, expected, reason)
    }

    pub(crate) fn plan_legacy_retirement_releasing_snapshots(
        &self,
        branch: SignalBranchHandle,
        expected: AdmittedSignalBranchBasis,
        releasing_snapshots: &[&AdmittedSignalBranchSnapshot],
        reason: SignalBranchRetirementReason,
    ) -> TransitionOutcome<PlannedSignalBranchRetirement, SignalBranchRetirementDenial> {
        let owner = match self.upgrade_sealed_owner() {
            Ok(owner) => owner,
            Err(unavailable) => {
                return TransitionOutcome::denied(SignalBranchRetirementDenial::OwnerUnavailable(
                    unavailable,
                ))
            }
        };
        let admission = match owner.admit() {
            Ok(admission) => admission,
            Err(denial) => {
                return TransitionOutcome::denied(map_retirement_admission_denial(denial))
            }
        };
        if branch.id != expected.owner_branch_id() {
            if let Err(denial) = owner.retirement_snapshot_allowance(branch.id, releasing_snapshots)
            {
                return TransitionOutcome::denied(denial);
            }
            if let Err(denial) = owner.lookup_cell(&admission, branch.id) {
                return TransitionOutcome::denied(map_retirement_registry_denial(
                    denial, branch.id,
                ));
            }
            return TransitionOutcome::denied(SignalBranchRetirementDenial::CanonicalBasisMismatch);
        }
        owner.plan_retirement_releasing_snapshots_exact(
            &admission,
            expected,
            releasing_snapshots,
            reason,
        )
    }

    pub(crate) fn legacy_retention_terminal_counts(&self) -> SignalBranchRetentionTerminalCounts {
        self.upgrade_sealed_owner()
            .expect("a sealed runtime root retains its canonical Signal owner")
            .retention_terminal_counts()
    }

    pub(crate) fn plan_legacy_retirement_batch(
        &self,
        requests: Vec<(
            SignalBranchHandle,
            AdmittedSignalBranchBasis,
            SignalBranchRetirementReason,
        )>,
    ) -> TransitionOutcome<PlannedSignalBranchRetirementBatch, SignalBranchRetirementBatchDenial>
    {
        self.plan_legacy_retirement_batch_releasing_snapshots(
            requests
                .into_iter()
                .map(|(branch, basis, reason)| (branch, basis, Vec::new(), reason))
                .collect(),
        )
    }

    pub(crate) fn plan_legacy_retirement_batch_releasing_snapshots(
        &self,
        requests: Vec<(
            SignalBranchHandle,
            AdmittedSignalBranchBasis,
            Vec<&AdmittedSignalBranchSnapshot>,
            SignalBranchRetirementReason,
        )>,
    ) -> TransitionOutcome<PlannedSignalBranchRetirementBatch, SignalBranchRetirementBatchDenial>
    {
        if requests.is_empty() {
            return TransitionOutcome::denied(SignalBranchRetirementBatchDenial::Empty);
        }
        let owner = match self.upgrade_sealed_owner() {
            Ok(owner) => owner,
            Err(unavailable) => {
                return TransitionOutcome::denied(SignalBranchRetirementBatchDenial::Retirement {
                    position: 0,
                    denial: SignalBranchRetirementDenial::OwnerUnavailable(unavailable),
                })
            }
        };
        let admission = match owner.admit() {
            Ok(admission) => admission,
            Err(denial) => {
                return TransitionOutcome::denied(SignalBranchRetirementBatchDenial::Retirement {
                    position: 0,
                    denial: map_retirement_admission_denial(denial),
                })
            }
        };
        owner.plan_legacy_retirement_batch(&admission, requests)
    }

    pub(crate) fn retire_legacy_batch(
        &self,
        batch: PlannedSignalBranchRetirementBatch,
    ) -> TransitionOutcome<SignalBranchRetirementBatchReceipt, SignalBranchRetirementBatchDenial>
    {
        let owner = match self.upgrade_sealed_owner() {
            Ok(owner) => owner,
            Err(unavailable) => {
                return TransitionOutcome::denied(SignalBranchRetirementBatchDenial::Retirement {
                    position: 0,
                    denial: SignalBranchRetirementDenial::OwnerUnavailable(unavailable),
                })
            }
        };
        let admission = match owner.admit() {
            Ok(admission) => admission,
            Err(denial) => {
                return TransitionOutcome::denied(SignalBranchRetirementBatchDenial::Retirement {
                    position: 0,
                    denial: map_retirement_admission_denial(denial),
                })
            }
        };
        let cancellation = super::SignalOwnerCancellationSource::new();
        owner.retire_legacy_batch(&admission, batch, &cancellation.token())
    }

    pub(crate) fn legacy_retirement_receipt(
        &self,
        branch_id: crate::state::SignalBranchId,
    ) -> Option<SignalBranchRetirementReceipt> {
        let owner = self.upgrade_sealed_owner().ok()?;
        let admission = owner.admit().ok()?;
        owner
            .metadata
            .retirement_receipt(&admission, branch_id)
            .ok()
            .flatten()
    }

    fn upgrade_sealed_owner(
        &self,
    ) -> Result<std::sync::Arc<SignalOwner<D, I, T>>, SignalOwnerUnavailable> {
        SignalOwner::upgrade(&self.downgrade_owner()?)
    }
}

fn map_readmission_metadata_denial(
    denial: SignalOwnerMetadataAuthorizationDenial,
    branch_id: crate::state::SignalBranchId,
) -> SignalBranchBasisReadmissionDenial {
    match denial {
        SignalOwnerMetadataAuthorizationDenial::OwnerUnavailable => {
            SignalBranchBasisReadmissionDenial::OwnerUnavailable(SignalOwnerUnavailable)
        }
        SignalOwnerMetadataAuthorizationDenial::OwnerCellMisuse => {
            SignalBranchBasisReadmissionDenial::OwnerCellMisuse { branch_id }
        }
        SignalOwnerMetadataAuthorizationDenial::OwnerReentry => {
            SignalBranchBasisReadmissionDenial::OwnerReentry
        }
    }
}
