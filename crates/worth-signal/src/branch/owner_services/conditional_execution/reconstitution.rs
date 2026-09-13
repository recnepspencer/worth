use crate::branch::owner_services::basis_port::denial_mapping::map_observation_admission_denial;
use crate::branch::owner_services::owner::basis::map_basis_registry_denial;
use crate::branch::owner_services::SignalOwner;
use crate::branch::AdmittedSignalBranchBasis;
use crate::data::conditional_execution::InstalledSignalConditionalContract;

use super::{
    SignalConditionalExecutionPort, SignalConditionalServiceExecutionDenial,
    SignalConditionalServiceIssuanceDenial,
};

/// Reconstructive service readmission against unchanged authoritative meaning.
/// It grants no definition publication and creates no Signal graph or branch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SignalConditionalReconstitutionReport {
    owner_runtime_instance_id: u64,
    definition_basis: u64,
}

impl SignalConditionalReconstitutionReport {
    pub const fn owner_runtime_instance_id(self) -> u64 {
        self.owner_runtime_instance_id
    }
    pub const fn definition_basis(self) -> u64 {
        self.definition_basis
    }
    pub const fn service_readmission_count(self) -> usize {
        1
    }
}

impl<D, I, T> SignalConditionalExecutionPort<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn reconstitute_at_basis(
        &self,
        basis: &AdmittedSignalBranchBasis,
    ) -> Result<(Self, SignalConditionalReconstitutionReport), SignalConditionalServiceIssuanceDenial>
    {
        let successor = self.reissue_for_successor_basis(basis)?;
        if !successor.definition.matches(&self.definition) {
            return Err(SignalConditionalServiceIssuanceDenial::DefinitionReadmissionRequired);
        }
        let owner = SignalOwner::upgrade(&successor.owner)
            .map_err(SignalConditionalServiceIssuanceDenial::OwnerUnavailable)?;
        let report = SignalConditionalReconstitutionReport {
            owner_runtime_instance_id: owner.runtime_instance_id(),
            definition_basis: successor.definition.definition_basis(),
        };
        Ok((successor, report))
    }

    /// Re-admits the exact installed contract without creating a new definition.
    pub fn readmit_installed_contract(
        &self,
        contract: &InstalledSignalConditionalContract,
    ) -> Result<InstalledSignalConditionalContract, SignalConditionalServiceExecutionDenial> {
        use SignalConditionalServiceExecutionDenial as Denial;
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
        cell.readmit_conditional_contract(&admission, &self.basis, &self.definition, contract)
    }
}
