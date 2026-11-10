use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use std::marker::PhantomData;

use crate::application::{
    WorthQueryCapabilityFamily, WorthQueryCapabilityStatus, WorthQueryConfigSectionFamily,
    WorthQueryDomainEntryMarker, WorthQueryDomainEntrySupportSnapshot,
    WorthQueryDomainOperatingContext, WorthQueryDomainOperatingRequirement,
};
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

use super::{
    WorthQueryDomainHandleDenial, WorthQueryInstalledDomainAuthorityWitness,
    WorthQueryInstalledDomainExecutionDrift,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInstalledDomainDeclarationContextDenialKind {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    HandleAuthority,
    UndeclaredCapability,
    UndeclaredConfiguration,
    UndeclaredOperatingRequirement,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledDomainDeclarationContextDenial {
    kind: WorthQueryInstalledDomainDeclarationContextDenialKind,
    requirement: String,
    handle_denial: Option<WorthQueryDomainHandleDenial>,
}

impl WorthQueryInstalledDomainDeclarationContextDenial {
    pub(crate) fn handle(denial: WorthQueryDomainHandleDenial) -> Self {
        Self {
            kind: WorthQueryInstalledDomainDeclarationContextDenialKind::HandleAuthority,
            requirement: format!("{:?}", denial.kind()),
            handle_denial: Some(denial),
        }
    }

    fn for_requirement(
        kind: WorthQueryInstalledDomainDeclarationContextDenialKind,
        requirement: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            requirement: requirement.into(),
            handle_denial: None,
        }
    }

    pub fn kind(&self) -> WorthQueryInstalledDomainDeclarationContextDenialKind {
        self.kind
    }

    pub fn requirement(&self) -> &str {
        &self.requirement
    }

    pub fn handle_denial(&self) -> Option<&WorthQueryDomainHandleDenial> {
        self.handle_denial.as_ref()
    }
}

impl std::fmt::Display for WorthQueryInstalledDomainDeclarationContextDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "installed domain declaration context denied: {:?} ({})",
            self.kind, self.requirement
        )
    }
}

impl std::error::Error for WorthQueryInstalledDomainDeclarationContextDenial {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledDomainDeclarationContext<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
> {
    installed_authority: WorthQueryInstalledDomainAuthorityWitness,
    operating_context: C,
    operating_context_identity: WorthQueryEvidenceIdentity,
    context_identity: WorthQueryEvidenceIdentity,
    marker: PhantomData<fn() -> D>,
}

impl<D, C> WorthQueryInstalledDomainDeclarationContext<D, C>
where
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
{
    pub(crate) fn admit(
        installed_authority: WorthQueryInstalledDomainAuthorityWitness,
        operating_context: C,
    ) -> Result<Self, WorthQueryInstalledDomainDeclarationContextDenial> {
        validate_context_requirements::<D, C>(&installed_authority, &operating_context)?;
        let operating_context_identity = operating_context_identity(&operating_context)?;
        let context_identity = worth_query_evidence_identity(
            WorthQueryEvidenceScope::InstalledDomainDeclarationContext,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("installed_authority"),
            installed_authority.witness_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("operating_context"),
            &operating_context_identity,
        )
        .seal()?;
        Ok(Self {
            installed_authority,
            operating_context,
            operating_context_identity,
            context_identity,
            marker: PhantomData,
        })
    }

    pub fn domain_key(&self) -> &'static str {
        self.installed_authority.authority().domain_key()
    }

    pub fn display_name(&self) -> &'static str {
        self.installed_authority.authority().display_name()
    }

    pub fn operating_context(&self) -> &C {
        &self.operating_context
    }

    pub fn support_snapshot(&self) -> &WorthQueryDomainEntrySupportSnapshot {
        self.installed_authority.authority().support_snapshot()
    }

    pub fn required_capability_families(&self) -> &[WorthQueryCapabilityFamily] {
        self.installed_authority.authority().required_capabilities()
    }

    pub fn required_config_sections(&self) -> &[WorthQueryConfigSectionFamily] {
        self.installed_authority
            .authority()
            .required_configuration()
    }

    pub fn required_operating_requirements(&self) -> &[WorthQueryDomainOperatingRequirement] {
        self.installed_authority
            .authority()
            .operating_requirements()
    }

    pub(crate) fn installed_capability_status(
        &self,
        family: WorthQueryCapabilityFamily,
    ) -> Option<WorthQueryCapabilityStatus> {
        (self.required_capability_families().contains(&family)
            && self
                .operating_context
                .required_capability_families()
                .contains(&family))
        .then(|| self.support_snapshot().capability_status(family))
        .flatten()
    }

    pub(crate) fn installed_configuration_enabled(
        &self,
        section: WorthQueryConfigSectionFamily,
    ) -> bool {
        self.required_config_sections().contains(&section)
            && self
                .operating_context
                .required_config_sections()
                .contains(&section)
            && self
                .support_snapshot()
                .section_postures()
                .iter()
                .find(|posture| posture.section() == section)
                .is_some_and(|posture| posture.enabled())
    }

    pub(crate) fn declares_operating_requirement(
        &self,
        requirement: WorthQueryDomainOperatingRequirement,
    ) -> bool {
        self.required_operating_requirements()
            .contains(&requirement)
            && self
                .operating_context
                .required_operating_requirements()
                .contains(&requirement)
    }

    pub fn handle_identity_digest(&self) -> &str {
        self.context_identity.as_str()
    }

    pub fn handle_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.context_identity
    }

    pub fn operating_context_identity_digest(&self) -> &str {
        self.operating_context_identity.as_str()
    }

    pub fn installed_authority(&self) -> &WorthQueryInstalledDomainAuthorityWitness {
        &self.installed_authority
    }

    pub(crate) fn validate_current_installation(
        &self,
    ) -> Result<(), WorthQueryInstalledDomainExecutionDrift> {
        WorthQueryInstalledDomainExecutionDrift::validate_current(&self.installed_authority)
    }

    pub(crate) fn contribution_target<I>(
        &self,
        declaration: &crate::application::WorthQueryCanonicalDeclarationArtifact<D, I>,
    ) -> Result<
        crate::domain_capabilities::WorthQueryInstalledDeclarationContributionTarget,
        IdentityDerivationDenial,
    >
    where
        I: crate::application::WorthQueryDeclarationInput<D>,
    {
        let semantic_target =
            crate::domain_capabilities::WorthQueryDeclarationBoundContributionTarget::for_canonical_declaration(
                declaration,
            );
        crate::domain_capabilities::WorthQueryInstalledDomainContributionTarget::bind(
            self.installed_authority.authority_arc(),
            semantic_target,
        )
    }

    pub fn declaration_family_version(&self, family_key: &str) -> Option<u32> {
        self.installed_authority
            .authority()
            .declaration_family_version(family_key)
    }

    pub fn retained_world_basis(&self) -> crate::application::WorthQueryAdmittedWorldBasis {
        let basis_lifecycle_support = crate::basis_lifecycle::basis_lifecycle_support_matrix();
        crate::application::WorthQueryAdmittedWorldBasis::new(
            self.domain_key(),
            self.display_name(),
            self.operating_context_identity.clone(),
            self.context_identity.clone(),
            self.support_snapshot().snapshot_digest().to_string(),
            crate::application::compose_basis_lifecycle_support_identity(
                basis_lifecycle_support.matrix_digest(),
            ),
            self.installed_authority.clone(),
        )
    }
}

fn validate_context_requirements<D, C>(
    installed_authority: &WorthQueryInstalledDomainAuthorityWitness,
    operating_context: &C,
) -> Result<(), WorthQueryInstalledDomainDeclarationContextDenial>
where
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
{
    let authority = installed_authority.authority();
    if let Some(missing) = operating_context
        .required_capability_families()
        .iter()
        .find(|family| !authority.required_capabilities().contains(family))
    {
        return Err(
            WorthQueryInstalledDomainDeclarationContextDenial::for_requirement(
                WorthQueryInstalledDomainDeclarationContextDenialKind::UndeclaredCapability,
                missing.as_str(),
            ),
        );
    }
    if let Some(missing) = operating_context
        .required_config_sections()
        .iter()
        .find(|section| !authority.required_configuration().contains(section))
    {
        return Err(
            WorthQueryInstalledDomainDeclarationContextDenial::for_requirement(
                WorthQueryInstalledDomainDeclarationContextDenialKind::UndeclaredConfiguration,
                missing.as_str(),
            ),
        );
    }
    if let Some(missing) = operating_context
        .required_operating_requirements()
        .iter()
        .find(|requirement| !authority.operating_requirements().contains(requirement))
    {
        return Err(WorthQueryInstalledDomainDeclarationContextDenial::for_requirement(
            WorthQueryInstalledDomainDeclarationContextDenialKind::UndeclaredOperatingRequirement,
            missing.as_str(),
        ));
    }
    Ok(())
}

fn operating_context_identity<D, C>(
    operating_context: &C,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial>
where
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
{
    let declaration = operating_context.context_identity();
    worth_query_evidence_identity(WorthQueryEvidenceScope::InstalledDomainDeclarationContext)
        .field_value_sequence(
            WorthQueryEvidenceTag::new("operating_context_field_name"),
            declaration.fields().map(|(name, _)| name),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("operating_context_field_value"),
            declaration.fields().map(|(_, value)| value),
        )
        .seal()
}

impl From<IdentityDerivationDenial> for WorthQueryInstalledDomainDeclarationContextDenial {
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self::for_requirement(
            WorthQueryInstalledDomainDeclarationContextDenialKind::EvidenceIdentityDerivationDenied(
                cause,
            ),
            "canonical declaration context identity",
        )
    }
}
