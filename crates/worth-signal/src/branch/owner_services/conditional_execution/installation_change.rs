use crate::branch::owner_services::basis_port::denial_mapping::map_observation_admission_denial;
use crate::branch::owner_services::owner::basis::map_basis_registry_denial;
use crate::branch::owner_services::{SignalOwner, SignalOwnerUnavailable};
use crate::branch::{SignalBranchBasisObservationDenial, SignalConditionalExecutionPort};
use crate::data::conditional_execution::InstalledSignalConditionalContract;
use crate::data::error::SignalError;
use crate::data::handle::NodeId;

#[derive(Debug)]
pub enum SignalConditionalInstallationChangeDenial {
    OwnerUnavailable(SignalOwnerUnavailable),
    OwnerAdmission(SignalBranchBasisObservationDenial),
    StaleBasisAdmission,
    DefinitionReadmissionRequired,
    DefinitionMismatch,
    SuccessorCaptureCapacityExhausted,
    SuccessorCaptureWorkExhausted { maximum_visits: usize },
    SuccessorCaptureUnavailable,
    SignalMutation(SignalError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignalConditionalRetirementCompletion {
    node: NodeId,
}

impl SignalConditionalRetirementCompletion {
    pub(crate) const fn new(node: NodeId) -> Self {
        Self { node }
    }

    pub const fn node(self) -> NodeId {
        self.node
    }
}

impl<D, I, T> SignalConditionalExecutionPort<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn retire_installed_contract(
        &self,
        contract: &InstalledSignalConditionalContract,
    ) -> Result<SignalConditionalRetirementCompletion, SignalConditionalInstallationChangeDenial>
    {
        use SignalConditionalInstallationChangeDenial as Denial;
        let owner = SignalOwner::upgrade(&self.owner).map_err(Denial::OwnerUnavailable)?;
        let admission = owner
            .admit()
            .map_err(map_observation_admission_denial)
            .map_err(Denial::OwnerAdmission)?;
        let branch = self.basis.owner_branch_id();
        let cell = owner
            .lookup_cell(&admission, branch)
            .map_err(|denial| Denial::OwnerAdmission(map_basis_registry_denial(denial, branch)))?;
        if cell.incarnation() != self.incarnation {
            return Err(Denial::StaleBasisAdmission);
        }
        cell.retire_conditional_contract(
            &admission,
            &self.basis,
            &self.definition,
            contract,
            &owner.conditional_retention,
        )
    }
}
