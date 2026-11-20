use std::sync::Arc;

pub struct WorthQueryArtifactTransferAdmission {
    pub(super) expected_contract:
        Arc<worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority>,
    pub(super) domain_authority:
        Arc<crate::domain_computation::WorthQueryInstalledDomainExecutionAuthority>,
    pub(super) operation_identity: String,
    pub(super) binding_identity: String,
    pub(super) run_identity: String,
    pub(super) predecessor_stage: String,
    pub(super) consumer_stage: String,
    pub(super) basis_identity: String,
}

impl WorthQueryArtifactTransferAdmission {
    pub fn mint(parts: WorthQueryArtifactTransferAdmissionParts) -> Self {
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

pub struct WorthQueryArtifactTransferAdmissionParts {
    pub expected_contract:
        Arc<worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority>,
    pub domain_authority:
        Arc<crate::domain_computation::WorthQueryInstalledDomainExecutionAuthority>,
    pub operation_identity: String,
    pub binding_identity: String,
    pub run_identity: String,
    pub predecessor_stage: String,
    pub consumer_stage: String,
    pub basis_identity: String,
}
