use std::sync::Arc;

use crate::branch::owner_services::basis_port::denial_mapping::map_observation_admission_denial;
use crate::branch::owner_services::owner::basis::map_basis_registry_denial;
use crate::branch::owner_services::SignalOwner;
use crate::data::conditional_execution::{
    InstalledSignalConditionalContract, SignalConditionalServiceContractBinding,
};

use super::{
    SignalConditionalExecutionPort, SignalConditionalServiceExecutionDenial,
    SignalRetainedExecutionBasis,
};

impl<D, I, T> SignalConditionalExecutionPort<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn validate_installed_contract(
        &self,
        contract: &InstalledSignalConditionalContract,
    ) -> Result<(), SignalConditionalServiceExecutionDenial> {
        self.admit_contract_binding(contract).map(|_| ())
    }

    pub(super) fn admit_contract_binding(
        &self,
        contract: &InstalledSignalConditionalContract,
    ) -> Result<
        (
            SignalConditionalServiceContractBinding,
            Arc<SignalRetainedExecutionBasis>,
        ),
        SignalConditionalServiceExecutionDenial,
    > {
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
        cell.admit_conditional_evaluation(
            &admission,
            &self.basis,
            &self.definition,
            contract,
            &self._issuance_basis_custody,
        )
    }
}
