use super::WorthQueryInstalledDomainAuthorityWitness;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInstalledDomainCapabilityKind {
    Read,
    Projection,
    Inspection,
    LiveOpen,
    LiveRead,
    LiveDelivery,
    LiveObservation,
    LiveCheckpoint,
    LiveResume,
    LiveClose,
    Mutation,
    Workflow,
}

impl WorthQueryInstalledDomainCapabilityKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Projection => "projection",
            Self::Inspection => "inspection",
            Self::LiveOpen => "live-open",
            Self::LiveRead => "live-read",
            Self::LiveDelivery => "live-delivery",
            Self::LiveObservation => "live-observation",
            Self::LiveCheckpoint => "live-checkpoint",
            Self::LiveResume => "live-resume",
            Self::LiveClose => "live-close",
            Self::Mutation => "mutation",
            Self::Workflow => "workflow",
        }
    }
}

#[derive(Debug)]
pub struct WorthQueryInstalledDomainCapabilityStop<S> {
    installed_authority: WorthQueryInstalledDomainAuthorityWitness,
    capability: WorthQueryInstalledDomainCapabilityKind,
    declaration_identity: WorthQueryEvidenceIdentity,
    stop: S,
}

impl<S> WorthQueryInstalledDomainCapabilityStop<S> {
    pub(crate) fn new(
        installed_authority: WorthQueryInstalledDomainAuthorityWitness,
        capability: WorthQueryInstalledDomainCapabilityKind,
        declaration_identity: WorthQueryEvidenceIdentity,
        stop: S,
    ) -> Self {
        Self {
            installed_authority,
            capability,
            declaration_identity,
            stop,
        }
    }

    pub fn installed_authority(&self) -> &WorthQueryInstalledDomainAuthorityWitness {
        &self.installed_authority
    }

    pub fn capability(&self) -> WorthQueryInstalledDomainCapabilityKind {
        self.capability
    }

    pub fn declaration_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.declaration_identity
    }

    pub fn stop(&self) -> &S {
        &self.stop
    }

    pub fn into_stop(self) -> S {
        self.stop
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledDomainExecutionReceipt {
    installed_authority: WorthQueryInstalledDomainAuthorityWitness,
    capability: WorthQueryInstalledDomainCapabilityKind,
    declaration_identity: WorthQueryEvidenceIdentity,
    basis_identity: WorthQueryEvidenceIdentity,
    operational_identity: WorthQueryEvidenceIdentity,
    managed_live_resource_identity: Option<WorthQueryEvidenceIdentity>,
    receipt_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryInstalledDomainExecutionReceipt {
    pub(crate) fn new(
        installed_authority: WorthQueryInstalledDomainAuthorityWitness,
        capability: WorthQueryInstalledDomainCapabilityKind,
        declaration_identity: WorthQueryEvidenceIdentity,
        basis_identity: WorthQueryEvidenceIdentity,
        operational_identity: WorthQueryEvidenceIdentity,
    ) -> Self {
        Self::new_with_managed_live_resource(
            installed_authority,
            capability,
            declaration_identity,
            basis_identity,
            operational_identity,
            None,
        )
    }

    pub(crate) fn new_managed_live(
        installed_authority: WorthQueryInstalledDomainAuthorityWitness,
        capability: WorthQueryInstalledDomainCapabilityKind,
        declaration_identity: WorthQueryEvidenceIdentity,
        basis_identity: WorthQueryEvidenceIdentity,
        operational_identity: WorthQueryEvidenceIdentity,
        managed_live_resource_identity: WorthQueryEvidenceIdentity,
    ) -> Self {
        Self::new_with_managed_live_resource(
            installed_authority,
            capability,
            declaration_identity,
            basis_identity,
            operational_identity,
            Some(managed_live_resource_identity),
        )
    }

    fn new_with_managed_live_resource(
        installed_authority: WorthQueryInstalledDomainAuthorityWitness,
        capability: WorthQueryInstalledDomainCapabilityKind,
        declaration_identity: WorthQueryEvidenceIdentity,
        basis_identity: WorthQueryEvidenceIdentity,
        operational_identity: WorthQueryEvidenceIdentity,
        managed_live_resource_identity: Option<WorthQueryEvidenceIdentity>,
    ) -> Self {
        let receipt_identity =
            worth_query_evidence_identity(WorthQueryEvidenceScope::InstalledDomainExecution)
                .field_shape(
                    WorthQueryEvidenceTag::new("capability"),
                    capability.as_str(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("installed_authority"),
                    installed_authority.witness_identity(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("declaration"),
                    &declaration_identity,
                )
                .field_evidence_identity(WorthQueryEvidenceTag::new("basis"), &basis_identity)
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("operational"),
                    &operational_identity,
                )
                .optional_evidence_identity(
                    WorthQueryEvidenceTag::new("managed_live_resource"),
                    managed_live_resource_identity.as_ref(),
                )
                .seal().expect("fixed digest grammar: literal tags, primitive values, closed enum labels, or sealed digest tokens");
        Self {
            installed_authority,
            capability,
            declaration_identity,
            basis_identity,
            operational_identity,
            managed_live_resource_identity,
            receipt_identity,
        }
    }

    pub(crate) fn label_identity(
        role: &'static str,
        value: &str,
    ) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        Ok(
            worth_query_evidence_identity(WorthQueryEvidenceScope::InstalledDomainExecution)
                .field_shape(WorthQueryEvidenceTag::new("role"), role)
                .field_value(WorthQueryEvidenceTag::new("query_minted_value"), value)
                .seal()?,
        )
    }

    pub(crate) fn derive(
        &self,
        capability: WorthQueryInstalledDomainCapabilityKind,
        operational_identity: WorthQueryEvidenceIdentity,
    ) -> Self {
        Self::new_with_managed_live_resource(
            self.installed_authority.clone(),
            capability,
            self.declaration_identity.clone(),
            self.basis_identity.clone(),
            operational_identity,
            self.managed_live_resource_identity.clone(),
        )
    }

    pub fn installed_authority(&self) -> &WorthQueryInstalledDomainAuthorityWitness {
        &self.installed_authority
    }

    pub fn capability(&self) -> WorthQueryInstalledDomainCapabilityKind {
        self.capability
    }

    pub fn declaration_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.declaration_identity
    }

    pub fn basis_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.basis_identity
    }

    pub fn operational_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.operational_identity
    }

    /// Compare opaque Query-owned managed-resource provenance without exposing
    /// an identity token that a downstream consumer could mistake for
    /// authority. Non-live receipts never match.
    pub fn shares_managed_live_resource_with(&self, other: &Self) -> bool {
        matches!(
            (
                self.managed_live_resource_identity.as_ref(),
                other.managed_live_resource_identity.as_ref(),
            ),
            (Some(left), Some(right)) if left == right
        )
    }

    pub fn receipt_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.receipt_identity
    }
}
