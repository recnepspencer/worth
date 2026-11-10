use std::sync::Arc;

use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

use super::{WorthQueryDomainPackageIdentity, WorthQueryInstalledDomainAuthority};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledDomainAuthorityWitness {
    authority: Arc<WorthQueryInstalledDomainAuthority>,
    witness_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryInstalledDomainAuthorityWitness {
    pub(crate) fn from_authority(authority: Arc<WorthQueryInstalledDomainAuthority>) -> Self {
        let witness_identity =
            worth_query_evidence_identity(WorthQueryEvidenceScope::InstalledDomainExecution)
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("installed_authority"),
                    authority.authority_identity(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("world"),
                    authority.world_identity(),
                )
                .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens");
        Self {
            authority,
            witness_identity,
        }
    }

    pub fn authority(&self) -> &WorthQueryInstalledDomainAuthority {
        &self.authority
    }
    pub fn witness_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.witness_identity
    }
    pub fn package_identity(&self) -> &WorthQueryDomainPackageIdentity {
        self.authority.package_identity()
    }
    pub fn world_identity(&self) -> &WorthQueryEvidenceIdentity {
        self.authority.world_identity()
    }
    pub(crate) fn authority_arc(&self) -> Arc<WorthQueryInstalledDomainAuthority> {
        Arc::clone(&self.authority)
    }
}
