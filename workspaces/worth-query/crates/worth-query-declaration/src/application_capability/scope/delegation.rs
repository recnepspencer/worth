//! Who holds a grant, how it was delegated, and how it is minted and revoked.
use super::{ApplicationCapabilityFieldBinding, ApplicationCapabilityRelationBinding};
use crate::application_capability::{
    ApplicationCapabilityDelegationActivationDefinition, ApplicationCapabilityProvenanceRef,
    ApplicationCapabilityRevocationDefinition,
};
use crate::portable_identity::WorthQueryPortableTypeIdentity;

/// The delegation lineage of a workflow-tenure grant: its parent grant, the
/// grantor who delegated it and its remaining delegation allowance.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationCapabilityDelegationLineage {
    parent: ApplicationCapabilityRelationBinding,
    grantor: ApplicationCapabilityRelationBinding,
    limit: ApplicationCapabilityFieldBinding,
}

impl ApplicationCapabilityDelegationLineage {
    pub fn new(
        parent: ApplicationCapabilityRelationBinding,
        grantor: ApplicationCapabilityRelationBinding,
        limit: ApplicationCapabilityFieldBinding,
    ) -> Self {
        Self {
            parent,
            grantor,
            limit,
        }
    }

    pub fn parent(&self) -> &ApplicationCapabilityRelationBinding {
        &self.parent
    }

    pub fn grantor(&self) -> &ApplicationCapabilityRelationBinding {
        &self.grantor
    }

    pub fn limit(&self) -> &ApplicationCapabilityFieldBinding {
        &self.limit
    }
}

/// Whether an admitted operation may link its own principal as a grantee.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationCapabilitySelfGrantRule {
    Permitted,
    /// No admitted operation may create a grantee link whose principal end is
    /// the operation's own admitted principal.
    Forbidden,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationCapabilityDelegationDefinition {
    pub(super) lineage: Option<ApplicationCapabilityDelegationLineage>,
    pub(super) grantee: ApplicationCapabilityRelationBinding,
    pub(super) provenance: String,
    pub(super) provenance_type: WorthQueryPortableTypeIdentity,
    pub(super) activation: Option<ApplicationCapabilityDelegationActivationDefinition>,
    pub(super) revocation: Option<ApplicationCapabilityRevocationDefinition>,
    pub(super) self_grant: ApplicationCapabilitySelfGrantRule,
}

impl ApplicationCapabilityDelegationDefinition {
    /// A workflow-tenure grant with a delegation lineage.
    pub fn new<Schema, Provenance>(
        parent: ApplicationCapabilityRelationBinding,
        grantor: ApplicationCapabilityRelationBinding,
        grantee: ApplicationCapabilityRelationBinding,
        limit: ApplicationCapabilityFieldBinding,
        provenance: ApplicationCapabilityProvenanceRef<Schema, Provenance>,
    ) -> Self {
        Self::with_lineage(
            Some(ApplicationCapabilityDelegationLineage::new(
                parent, grantor, limit,
            )),
            grantee,
            provenance,
        )
    }

    /// A standing grant: held through `grantee`, never delegated. Pair it with
    /// `ApplicationCapabilityCurrentnessDefinition::standing`.
    pub fn standing<Schema, Provenance>(
        grantee: ApplicationCapabilityRelationBinding,
        provenance: ApplicationCapabilityProvenanceRef<Schema, Provenance>,
    ) -> Self {
        Self::with_lineage(None, grantee, provenance)
    }

    fn with_lineage<Schema, Provenance>(
        lineage: Option<ApplicationCapabilityDelegationLineage>,
        grantee: ApplicationCapabilityRelationBinding,
        provenance: ApplicationCapabilityProvenanceRef<Schema, Provenance>,
    ) -> Self {
        Self {
            lineage,
            grantee,
            provenance: provenance.name().to_string(),
            provenance_type: provenance.marker_identity(),
            activation: None,
            revocation: None,
            self_grant: ApplicationCapabilitySelfGrantRule::Permitted,
        }
    }

    pub fn with_activation(
        mut self,
        activation: ApplicationCapabilityDelegationActivationDefinition,
    ) -> Self {
        self.activation = Some(activation);
        self
    }

    pub fn with_revocation(
        mut self,
        revocation: ApplicationCapabilityRevocationDefinition,
    ) -> Self {
        self.revocation = Some(revocation);
        self
    }

    /// Refuses any admitted operation that links its own principal as a
    /// grantee of this grant kind, through any handler or declarative body
    /// link. A standing grant only; installation refuses it with a lineage.
    pub fn forbid_self_grant(mut self) -> Self {
        self.self_grant = ApplicationCapabilitySelfGrantRule::Forbidden;
        self
    }

    pub const fn lineage(&self) -> Option<&ApplicationCapabilityDelegationLineage> {
        self.lineage.as_ref()
    }

    pub fn grantee(&self) -> &ApplicationCapabilityRelationBinding {
        &self.grantee
    }

    pub fn provenance(&self) -> &str {
        &self.provenance
    }

    pub fn provenance_type(&self) -> &str {
        self.provenance_type.as_str()
    }

    pub fn provenance_identity(&self) -> WorthQueryPortableTypeIdentity {
        self.provenance_type.clone()
    }

    pub const fn activation(&self) -> Option<&ApplicationCapabilityDelegationActivationDefinition> {
        self.activation.as_ref()
    }

    pub const fn revocation(&self) -> Option<&ApplicationCapabilityRevocationDefinition> {
        self.revocation.as_ref()
    }

    pub const fn self_grant(&self) -> ApplicationCapabilitySelfGrantRule {
        self.self_grant
    }
}
