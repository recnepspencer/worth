use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use std::collections::BTreeMap;

use crate::application::WorthQueryDeclarationEntryContributionCategoryFamily;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

use super::super::installation_state::WorthQueryInstalledDomainArtifact;
use super::semantic_keys::InstalledDeclarationFamilyKey;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorthQueryInstalledDomainSemantics {
    declaration_families: BTreeMap<InstalledDeclarationFamilyKey, u32>,
    contribution_policy: Vec<WorthQueryDeclarationEntryContributionCategoryFamily>,
    invariant_identities: BTreeMap<String, String>,
    identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryInstalledDomainSemantics {
    pub(super) fn from_artifact(
        artifact: &WorthQueryInstalledDomainArtifact,
    ) -> Result<Self, IdentityDerivationDenial> {
        let declaration_families = artifact
            .declaration_families
            .iter()
            .map(|family| {
                (
                    InstalledDeclarationFamilyKey::new(family.family_key()),
                    family.version(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let contribution_policy = artifact.contribution_policy.clone();
        let invariant_identities = artifact
            .invariant_definitions
            .iter()
            .map(|definition| (definition.slot_key(), definition.canonical_part()))
            .collect::<BTreeMap<_, _>>();
        let identity =
            worth_query_evidence_identity(WorthQueryEvidenceScope::InstalledDomainExecutionIndex)
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("package"),
                    artifact.package_identity.evidence_identity(),
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("declaration_family"),
                    declaration_families
                        .iter()
                        .map(|(family, version)| format!("{}:{version}", family.as_str())),
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("contribution_policy"),
                    contribution_policy.iter().map(|category| category.as_str()),
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("invariant"),
                    invariant_identities
                        .iter()
                        .map(|(slot, identity)| format!("{slot}:{identity}")),
                )
                .seal()?;
        Ok(Self {
            declaration_families,
            contribution_policy,
            invariant_identities,
            identity,
        })
    }

    pub(crate) fn declaration_family_version(&self, family_key: &str) -> Option<u32> {
        self.declaration_families.get(family_key).copied()
    }

    pub(crate) fn contribution_policy(
        &self,
    ) -> &[WorthQueryDeclarationEntryContributionCategoryFamily] {
        &self.contribution_policy
    }

    pub(crate) fn permits_contribution(
        &self,
        category: WorthQueryDeclarationEntryContributionCategoryFamily,
    ) -> bool {
        self.contribution_policy.contains(&category)
    }

    pub(crate) fn identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.identity
    }

    pub(crate) fn invariant_identity(&self, slot: &str) -> Option<&str> {
        self.invariant_identities.get(slot).map(String::as_str)
    }
}
