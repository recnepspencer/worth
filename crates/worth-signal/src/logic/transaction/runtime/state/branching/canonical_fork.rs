use worth_proof::TransitionOutcome;

use crate::branch::{
    validate_signal_branch_name, AdmittedSignalBranchBasis, SignalBranchForkOperationDenial,
    SignalBranchForkOutcome,
};
use crate::data::error::SignalError;

use super::super::runtime_state::SignalRuntime;
use super::{SignalBranchForkDenial, SignalBranchForkRequest};

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn fork_signal_branch(
        &mut self,
        name: impl Into<String>,
        source: &AdmittedSignalBranchBasis,
    ) -> Result<SignalBranchForkOutcome, SignalBranchForkOperationDenial> {
        let name = name.into();
        let validated_name = validate_signal_branch_name(name.clone())
            .map_err(|denial| SignalBranchForkOperationDenial::InvalidIdentity { denial })?;
        if let Some((_, mutation, _)) = self.sealed_owner_port_slots() {
            let cancellation = crate::branch::SignalOwnerCancellationSource::new();
            return mutation.fork_exact(validated_name, source, &cancellation.token());
        }
        let branch_id = source.owner_branch_id();
        let branch = self
            .branches
            .branch_handle(branch_id)
            .ok_or(SignalBranchForkOperationDenial::UnknownBranch { branch_id })?;
        let live = self
            .signal_branch_observation(&branch)
            .map_err(|_| SignalBranchForkOperationDenial::UnknownBranch { branch_id })?;
        if let Err(mismatch) = live.compare(source.observation()) {
            return Err(SignalBranchForkOperationDenial::BasisMismatch {
                axes: mismatch.axes().to_vec(),
            });
        }
        let mut retention = self
            .branches
            .acquire_admitted_retention(branch_id)
            .map_err(|denial| SignalBranchForkOperationDenial::RetentionUnavailable { denial })?;
        let request = SignalBranchForkRequest::from_parent_branch_head(name, branch_id);
        let receipt = match self.fork_branch(request) {
            TransitionOutcome::Success(receipt) => receipt,
            TransitionOutcome::Denied(SignalBranchForkDenial::BranchIdentityExhausted) => {
                return Err(SignalBranchForkOperationDenial::BranchIdentityExhausted)
            }
            TransitionOutcome::Denied(SignalBranchForkDenial::InvalidBranchIdentity) => {
                unreachable!("canonical fork identity was validated before owner effects")
            }
            TransitionOutcome::Denied(denial) => {
                return Err(SignalBranchForkOperationDenial::OwnerDeniedNoMovement {
                    error: Self::fork_denial_to_signal_error(denial),
                })
            }
            other => {
                return Err(SignalBranchForkOperationDenial::OwnerDeniedNoMovement {
                    error: SignalError::internal(format!(
                        "unexpected non-terminal Signal fork outcome: {other:?}"
                    )),
                })
            }
        };
        let created_branch = receipt.created_branch().clone();
        retention.rebind_branch(created_branch.id);
        let created_basis = self
            .admit_signal_branch_with_retention(created_branch.clone(), retention)
            .expect("validated created branch must admit its canonical basis");
        Ok(
            SignalBranchForkOutcome::owner_issued_without_service_reference(
                created_branch,
                created_basis,
            ),
        )
    }
}
