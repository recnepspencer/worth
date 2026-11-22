use std::sync::Arc;

use super::WorthQueryArtifactProductionAuthority;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryArtifactProductionEvidence {
    provenance_identity: String,
    dependency_identity: String,
}

impl WorthQueryArtifactProductionEvidence {
    pub fn new(
        provenance_identity: impl Into<String>,
        dependency_identity: impl Into<String>,
    ) -> Self {
        Self {
            provenance_identity: provenance_identity.into(),
            dependency_identity: dependency_identity.into(),
        }
    }

    pub fn provenance_identity(&self) -> &str {
        &self.provenance_identity
    }

    pub fn dependency_identity(&self) -> &str {
        &self.dependency_identity
    }

    pub(super) fn is_valid(&self) -> bool {
        [&self.provenance_identity, &self.dependency_identity]
            .into_iter()
            .all(|value| !value.trim().is_empty() && value.trim() == value)
    }
}

pub struct WorthQueryArtifactProductionAdmission {
    authority: Arc<WorthQueryArtifactProductionAuthority>,
    pub(super) evidence: WorthQueryArtifactProductionEvidence,
}

impl WorthQueryArtifactProductionAdmission {
    pub(super) fn mint(
        authority: Arc<WorthQueryArtifactProductionAuthority>,
        evidence: WorthQueryArtifactProductionEvidence,
    ) -> Self {
        Self {
            authority,
            evidence,
        }
    }

    pub fn contract(
        &self,
    ) -> &worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority {
        &self.authority.contract
    }

    pub fn operation_identity(&self) -> &str {
        &self.authority.operation_identity
    }

    pub fn run_identity(&self) -> &str {
        &self.authority.run_identity
    }

    pub fn stage_identity(&self) -> &str {
        &self.authority.stage_identity
    }

    pub fn basis_identity(&self) -> &str {
        &self.authority.basis_identity
    }

    pub fn production_generation(&self) -> u64 {
        self.authority.production_generation.ordinal()
    }

    pub(super) fn authority(&self) -> &Arc<WorthQueryArtifactProductionAuthority> {
        &self.authority
    }
}
