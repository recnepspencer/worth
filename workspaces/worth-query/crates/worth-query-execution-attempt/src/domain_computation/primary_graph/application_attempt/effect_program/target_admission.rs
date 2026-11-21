//! Admission of authored effect targets against the exact installed contract.

use worth_query_installation::facade::{
    ApplicationOperationProgramTarget, WorthQueryCompiledApplicationOperationContracts,
};

use crate::domain_computation::application_contract_admission::application_contract_admits_program_target;

pub(super) fn installed_contract_admits_program_target(
    contracts: &WorthQueryCompiledApplicationOperationContracts,
    target: &ApplicationOperationProgramTarget,
) -> bool {
    application_contract_admits_program_target(contracts, target)
}
