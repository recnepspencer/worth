use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorthQueryInstalledDomainSubstrateProvenance {
    domain_owner: String,
    package_version: String,
    package_identity: String,
    identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryInstalledDomainSubstrateProvenance {
    pub(crate) fn new(
        domain_owner: impl Into<String>,
        package_version_major: u32,
        package_version_minor: u32,
        package_identity: impl Into<String>,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        let domain_owner = domain_owner.into();
        let package_version = format!("{package_version_major}.{package_version_minor}");
        let package_identity = package_identity.into();
        let identity = worth_query_evidence_identity(
            WorthQueryEvidenceScope::InstalledDomainSubstrateProvenance,
        )
        .field_value(
            WorthQueryEvidenceTag::new("domain_owner"),
            domain_owner.as_str(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("package_version"),
            package_version.as_str(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("package_identity"),
            package_identity.as_str(),
        )
        .seal()?;
        Ok(Self {
            domain_owner,
            package_version,
            package_identity,
            identity,
        })
    }

    pub(crate) fn domain_owner(&self) -> &str {
        &self.domain_owner
    }

    pub(crate) fn package_version(&self) -> &str {
        &self.package_version
    }

    pub(crate) fn package_identity(&self) -> &str {
        &self.package_identity
    }

    pub(crate) fn identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.identity
    }
}
