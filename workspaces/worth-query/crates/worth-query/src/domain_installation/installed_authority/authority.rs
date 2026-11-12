use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use std::any::TypeId;
use std::sync::Arc;

use crate::application::{
    WorthQueryCapabilityFamily, WorthQueryConfigSectionFamily,
    WorthQueryDeclarationEntryContributionCategoryFamily, WorthQueryDomainEntrySupportSnapshot,
    WorthQueryDomainOperatingRequirement,
};
use crate::domain_capabilities::WorthQueryDomainCapabilityCategory;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::runtime::WorthQueryRuntimeAuthorityIdentity;

use super::{
    WorthQueryDomainInstallationGeneration, WorthQueryDomainInstallationGenerationLease,
    WorthQueryDomainPackageIdentity, WorthQueryInstalledDomainSemantics,
};

#[derive(Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledDomainAuthority {
    runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    generation: WorthQueryDomainInstallationGeneration,
    generation_lease: WorthQueryDomainInstallationGenerationLease,
    marker_type: TypeId,
    domain_key: &'static str,
    display_name: &'static str,
    domain_owner: String,
    package_identity: WorthQueryDomainPackageIdentity,
    installation_identity: WorthQueryEvidenceIdentity,
    support_snapshot: WorthQueryDomainEntrySupportSnapshot,
    required_capabilities: Vec<WorthQueryCapabilityFamily>,
    required_configuration: Vec<WorthQueryConfigSectionFamily>,
    operating_requirements: Vec<WorthQueryDomainOperatingRequirement>,
    semantics: Arc<WorthQueryInstalledDomainSemantics>,
    portable_authority: worth_query_installation::facade::WorthQueryInstalledPackageAuthority,
    world_identity: WorthQueryEvidenceIdentity,
    authority_identity: WorthQueryEvidenceIdentity,
}

pub(crate) struct WorthQueryInstalledDomainAuthorityInputs {
    pub(crate) runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    pub(crate) generation: WorthQueryDomainInstallationGeneration,
    pub(crate) generation_lease: WorthQueryDomainInstallationGenerationLease,
    pub(crate) marker_type: TypeId,
    pub(crate) domain_key: &'static str,
    pub(crate) display_name: &'static str,
    pub(crate) domain_owner: String,
    pub(crate) package_identity: WorthQueryDomainPackageIdentity,
    pub(crate) installation_identity: WorthQueryEvidenceIdentity,
    pub(crate) support_snapshot: WorthQueryDomainEntrySupportSnapshot,
    pub(crate) required_capabilities: Vec<WorthQueryCapabilityFamily>,
    pub(crate) required_configuration: Vec<WorthQueryConfigSectionFamily>,
    pub(crate) operating_requirements: Vec<WorthQueryDomainOperatingRequirement>,
    pub(crate) semantics: Arc<WorthQueryInstalledDomainSemantics>,
    pub(crate) portable_authority:
        worth_query_installation::facade::WorthQueryInstalledPackageAuthority,
}

impl WorthQueryInstalledDomainAuthority {
    pub(crate) fn new(
        inputs: WorthQueryInstalledDomainAuthorityInputs,
    ) -> Result<Self, IdentityDerivationDenial> {
        let world_identity = installed_world_identity(inputs.runtime_authority)?;
        let authority_identity = installed_authority_identity(
            inputs.runtime_authority,
            inputs.generation,
            &inputs.installation_identity,
            inputs.semantics.identity(),
        )?;
        Ok(Self {
            runtime_authority: inputs.runtime_authority,
            generation: inputs.generation,
            generation_lease: inputs.generation_lease,
            marker_type: inputs.marker_type,
            domain_key: inputs.domain_key,
            display_name: inputs.display_name,
            domain_owner: inputs.domain_owner,
            package_identity: inputs.package_identity,
            installation_identity: inputs.installation_identity,
            support_snapshot: inputs.support_snapshot,
            required_capabilities: inputs.required_capabilities,
            required_configuration: inputs.required_configuration,
            operating_requirements: inputs.operating_requirements,
            semantics: inputs.semantics,
            portable_authority: inputs.portable_authority,
            world_identity,
            authority_identity,
        })
    }

    pub fn domain_owner(&self) -> &str {
        &self.domain_owner
    }
    pub fn domain_key(&self) -> &'static str {
        self.domain_key
    }
    pub fn display_name(&self) -> &'static str {
        self.display_name
    }
    pub fn package_identity(&self) -> &WorthQueryDomainPackageIdentity {
        &self.package_identity
    }
    pub fn installation_identity(&self) -> &str {
        self.installation_identity.as_str()
    }
    pub const fn installation_generation(&self) -> WorthQueryDomainInstallationGeneration {
        self.generation
    }
    pub(crate) fn is_current_installation_generation(&self) -> bool {
        self.generation_lease.is_current(self.generation)
    }
    pub fn authority_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.authority_identity
    }
    pub fn contribution_policy(&self) -> &[WorthQueryDeclarationEntryContributionCategoryFamily] {
        self.semantics.contribution_policy()
    }
    pub fn required_capabilities(&self) -> &[WorthQueryCapabilityFamily] {
        &self.required_capabilities
    }
    pub fn required_configuration(&self) -> &[WorthQueryConfigSectionFamily] {
        &self.required_configuration
    }
    pub fn operating_requirements(&self) -> &[WorthQueryDomainOperatingRequirement] {
        &self.operating_requirements
    }
    pub fn support_snapshot(&self) -> &WorthQueryDomainEntrySupportSnapshot {
        &self.support_snapshot
    }
    pub fn declaration_family_version(&self, family_key: &str) -> Option<u32> {
        self.semantics.declaration_family_version(family_key)
    }
    pub(crate) fn installed_invariant_identity(&self, slot: &str) -> Option<&str> {
        self.semantics.invariant_identity(slot)
    }
    pub fn world_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.world_identity
    }
    pub(crate) fn permits_contribution(
        &self,
        category: WorthQueryDeclarationEntryContributionCategoryFamily,
    ) -> bool {
        self.semantics.permits_contribution(category)
    }
    pub(crate) fn permits_domain_capability_category(
        &self,
        category: WorthQueryDomainCapabilityCategory,
    ) -> bool {
        self.permits_contribution(contribution_category_family(category))
    }
    pub(crate) fn runtime_authority(&self) -> WorthQueryRuntimeAuthorityIdentity {
        self.runtime_authority
    }
    pub(crate) fn marker_type(&self) -> TypeId {
        self.marker_type
    }
    pub(crate) fn portable_authority(
        &self,
    ) -> &worth_query_installation::facade::WorthQueryInstalledPackageAuthority {
        &self.portable_authority
    }
}

fn installed_world_identity(
    runtime_authority: WorthQueryRuntimeAuthorityIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    worth_query_evidence_identity(WorthQueryEvidenceScope::InstalledDomainWorld)
        .field_value(
            WorthQueryEvidenceTag::new("runtime_authority"),
            runtime_authority.as_u64().to_string(),
        )
        .seal()
}

fn installed_authority_identity(
    runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    generation: WorthQueryDomainInstallationGeneration,
    installation_identity: &WorthQueryEvidenceIdentity,
    installed_semantics_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    worth_query_evidence_identity(WorthQueryEvidenceScope::InstalledDomainHandle)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("installation"),
            installation_identity,
        )
        .field_value(
            WorthQueryEvidenceTag::new("runtime_authority"),
            runtime_authority.as_u64().to_string(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("generation"),
            generation.ordinal().to_string(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("installed_semantics"),
            installed_semantics_identity,
        )
        .seal()
}

fn contribution_category_family(
    category: WorthQueryDomainCapabilityCategory,
) -> WorthQueryDeclarationEntryContributionCategoryFamily {
    match category {
        WorthQueryDomainCapabilityCategory::Admission => {
            WorthQueryDeclarationEntryContributionCategoryFamily::Admission
        }
        WorthQueryDomainCapabilityCategory::SupportTraceability => {
            WorthQueryDeclarationEntryContributionCategoryFamily::SupportTraceability
        }
        WorthQueryDomainCapabilityCategory::InvariantCapability => {
            WorthQueryDeclarationEntryContributionCategoryFamily::InvariantCapability
        }
        WorthQueryDomainCapabilityCategory::WorkflowPreview => {
            WorthQueryDeclarationEntryContributionCategoryFamily::WorkflowPreview
        }
        WorthQueryDomainCapabilityCategory::ContinuityLineage => {
            WorthQueryDeclarationEntryContributionCategoryFamily::ContinuityLineage
        }
        WorthQueryDomainCapabilityCategory::ConsequenceAftermath => {
            WorthQueryDeclarationEntryContributionCategoryFamily::ConsequenceAftermath
        }
        WorthQueryDomainCapabilityCategory::ExplanationInspection => {
            WorthQueryDeclarationEntryContributionCategoryFamily::ExplanationInspection
        }
    }
}
