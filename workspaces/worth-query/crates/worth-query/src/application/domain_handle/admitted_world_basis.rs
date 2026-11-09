use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryAdmittedWorldBasis {
    domain_key: &'static str,
    display_name: &'static str,
    operating_context_identity: WorthQueryEvidenceIdentity,
    handle_identity: WorthQueryEvidenceIdentity,
    support_snapshot_digest: String,
    basis_lifecycle_support_identity: WorthQueryEvidenceIdentity,
    installed_authority: crate::domain_installation::WorthQueryInstalledDomainAuthorityWitness,
}

impl WorthQueryAdmittedWorldBasis {
    pub(crate) fn new(
        domain_key: &'static str,
        display_name: &'static str,
        operating_context_identity: WorthQueryEvidenceIdentity,
        handle_identity: WorthQueryEvidenceIdentity,
        support_snapshot_digest: String,
        basis_lifecycle_support_identity: WorthQueryEvidenceIdentity,
        installed_authority: crate::domain_installation::WorthQueryInstalledDomainAuthorityWitness,
    ) -> Self {
        Self {
            domain_key,
            display_name,
            operating_context_identity,
            handle_identity,
            support_snapshot_digest,
            basis_lifecycle_support_identity,
            installed_authority,
        }
    }

    pub fn domain_key(&self) -> &'static str {
        self.domain_key
    }

    pub fn display_name(&self) -> &'static str {
        self.display_name
    }

    pub fn operating_context_identity_digest(&self) -> &str {
        self.operating_context_identity.as_str()
    }

    pub fn operating_context_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.operating_context_identity
    }

    pub fn handle_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.handle_identity
    }

    pub fn handle_identity_for_reporting(&self) -> &str {
        self.handle_identity.as_str()
    }

    pub fn support_snapshot_digest(&self) -> &str {
        &self.support_snapshot_digest
    }

    pub fn basis_lifecycle_support_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.basis_lifecycle_support_identity
    }

    pub fn basis_lifecycle_support_for_reporting(&self) -> &str {
        self.basis_lifecycle_support_identity.as_str()
    }

    pub fn installed_authority(
        &self,
    ) -> &crate::domain_installation::WorthQueryInstalledDomainAuthorityWitness {
        &self.installed_authority
    }
}

pub(crate) fn compose_basis_lifecycle_support_identity(
    matrix_digest: &str,
) -> WorthQueryEvidenceIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "basis_lifecycle_support_matrix_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("support_matrix"), matrix_digest)
        .seal().expect("private lifecycle matrix identity contains fixed metadata and one 64-byte SHA256 projection")
}
