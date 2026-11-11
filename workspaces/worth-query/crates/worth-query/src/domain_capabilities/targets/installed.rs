use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use std::sync::Arc;

use crate::domain_installation::WorthQueryInstalledDomainAuthority;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

use super::{
    WorthQueryAdmittedPlanBoundContributionTarget, WorthQueryDeclarationBoundContributionTarget,
    WorthQueryDomainCapabilityTarget, WorthQueryDomainCapabilityTargetBinding,
    WorthQueryDomainCapabilityTargetKind, WorthQueryDomainCapabilityTargetSemantics,
    WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledDomainContributionTarget<T>
where
    T: WorthQueryDomainCapabilityTargetBinding,
{
    authority: Arc<WorthQueryInstalledDomainAuthority>,
    target: T,
    target_identity: WorthQueryEvidenceIdentity,
    binding_identity: WorthQueryEvidenceIdentity,
    target_digest: String,
    binding_digest: String,
}

impl<T> WorthQueryInstalledDomainContributionTarget<T>
where
    T: WorthQueryDomainCapabilityTargetBinding,
{
    pub(crate) fn bind(
        authority: Arc<WorthQueryInstalledDomainAuthority>,
        target: T,
    ) -> Result<Self, IdentityDerivationDenial> {
        let target_identity = worth_query_evidence_identity(
            WorthQueryEvidenceScope::InstalledDomainContributionTarget,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("installed_authority"),
            authority.authority_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("world"),
            authority.world_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("semantic_target"),
            &target.target_identity()?,
        )
        .seal()?;
        let binding_identity = worth_query_evidence_identity(
            WorthQueryEvidenceScope::InstalledDomainContributionTarget,
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("target"), &target_identity)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("semantic_binding"),
            &target.binding_identity()?,
        )
        .seal()?;
        let target_digest = target_identity.as_str().to_string();
        let binding_digest = binding_identity.as_str().to_string();
        Ok(Self {
            authority,
            target,
            target_identity,
            binding_identity,
            target_digest,
            binding_digest,
        })
    }

    pub fn authority(&self) -> &WorthQueryInstalledDomainAuthority {
        &self.authority
    }

    pub fn world_identity(&self) -> &WorthQueryEvidenceIdentity {
        self.authority.world_identity()
    }

    pub fn semantic_target(&self) -> &T {
        &self.target
    }

    pub(crate) fn authority_arc(&self) -> Arc<WorthQueryInstalledDomainAuthority> {
        Arc::clone(&self.authority)
    }
}

impl<T> WorthQueryDomainCapabilityTargetBinding for WorthQueryInstalledDomainContributionTarget<T>
where
    T: WorthQueryDomainCapabilityTargetBinding,
{
    fn erased_target(&self) -> &WorthQueryDomainCapabilityTarget {
        self.target.erased_target()
    }

    fn into_erased_target(self) -> WorthQueryDomainCapabilityTarget {
        self.target.into_erased_target()
    }

    fn kind(&self) -> WorthQueryDomainCapabilityTargetKind {
        self.target.kind()
    }

    fn target_digest(&self) -> &str {
        &self.target_digest
    }

    fn binding_digest(&self) -> &str {
        &self.binding_digest
    }

    fn target_identity(&self) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        Ok(self.target_identity.clone())
    }

    fn binding_identity(&self) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        Ok(self.binding_identity.clone())
    }

    fn semantics(&self) -> &WorthQueryDomainCapabilityTargetSemantics {
        self.target.semantics()
    }

    fn installed_authority(&self) -> Option<&WorthQueryInstalledDomainAuthority> {
        Some(&self.authority)
    }
}

pub type WorthQueryInstalledDeclarationContributionTarget =
    WorthQueryInstalledDomainContributionTarget<WorthQueryDeclarationBoundContributionTarget>;
pub type WorthQueryInstalledAdmittedPlanContributionTarget =
    WorthQueryInstalledDomainContributionTarget<WorthQueryAdmittedPlanBoundContributionTarget>;
pub type WorthQueryInstalledLowerRuntimeContributionTarget =
    WorthQueryInstalledDomainContributionTarget<
        WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
    >;

pub(crate) trait WorthQueryDeclarationContributionTargetBinding:
    WorthQueryDomainCapabilityTargetBinding
{
}

pub(crate) trait WorthQueryAdmittedPlanContributionTargetBinding:
    WorthQueryDomainCapabilityTargetBinding
{
}

pub(crate) trait WorthQueryLowerRuntimeContributionTargetBinding:
    WorthQueryDomainCapabilityTargetBinding
{
}

impl WorthQueryDeclarationContributionTargetBinding
    for WorthQueryDeclarationBoundContributionTarget
{
}
impl WorthQueryDeclarationContributionTargetBinding
    for WorthQueryInstalledDeclarationContributionTarget
{
}
impl WorthQueryAdmittedPlanContributionTargetBinding
    for WorthQueryAdmittedPlanBoundContributionTarget
{
}
impl WorthQueryAdmittedPlanContributionTargetBinding
    for WorthQueryInstalledAdmittedPlanContributionTarget
{
}
impl WorthQueryLowerRuntimeContributionTargetBinding
    for WorthQueryLowerRuntimeBoundaryBoundContributionTarget
{
}
impl WorthQueryLowerRuntimeContributionTargetBinding
    for WorthQueryInstalledLowerRuntimeContributionTarget
{
}
