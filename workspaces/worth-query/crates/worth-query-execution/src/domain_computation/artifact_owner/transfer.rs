use std::sync::Arc;

pub struct WorthQueryArtifactTransferAdmission {
    pub(super) expected_contract:
        Arc<worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority>,
    pub(super) domain_authority:
        Arc<crate::domain_computation::vocabulary::operation_binding::installed_domain_authority::WorthQueryInstalledDomainExecutionAuthority>,
    pub(super) operation_identity: String,
    pub(super) binding_identity: String,
    pub(super) run_identity: String,
    pub(super) predecessor_stage: String,
    pub(super) consumer_stage: String,
    pub(super) basis_identity: String,
}

impl WorthQueryArtifactTransferAdmission {
    pub(crate) fn mint(parts: WorthQueryArtifactTransferAdmissionParts) -> Self {
        Self {
            expected_contract: parts.expected_contract,
            domain_authority: parts.domain_authority,
            operation_identity: parts.operation_identity,
            binding_identity: parts.binding_identity,
            run_identity: parts.run_identity,
            predecessor_stage: parts.predecessor_stage,
            consumer_stage: parts.consumer_stage,
            basis_identity: parts.basis_identity,
        }
    }

    pub fn predecessor_stage(&self) -> &str {
        &self.predecessor_stage
    }

    pub fn consumer_stage(&self) -> &str {
        &self.consumer_stage
    }

    pub fn run_identity(&self) -> &str {
        &self.run_identity
    }

    pub fn basis_identity(&self) -> &str {
        &self.basis_identity
    }
}

pub(crate) struct WorthQueryArtifactTransferAdmissionParts {
    pub(crate) expected_contract:
        Arc<worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority>,
    pub(crate) domain_authority:
        Arc<crate::domain_computation::vocabulary::operation_binding::installed_domain_authority::WorthQueryInstalledDomainExecutionAuthority>,
    pub(crate) operation_identity: String,
    pub(crate) binding_identity: String,
    pub(crate) run_identity: String,
    pub(crate) predecessor_stage: String,
    pub(crate) consumer_stage: String,
    pub(crate) basis_identity: String,
}
