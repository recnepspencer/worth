use std::sync::Arc;

use crate::identity::BridgeIdentityEvidence;
use crate::relational_source::identity_parts::RelationalBridgeRecordIdentityParts;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeNamingMutationFamily {
    AttachNewTarget,
    AttachExistingTarget,
    RebindTarget,
    Remove,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeNamingMutationOutcome {
    AttachedToNewTarget,
    AttachedToExistingTarget,
    ReboundTarget,
    Removed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeNamingAttachmentIdentity {
    value: Arc<str>,
}

impl BridgeNamingAttachmentIdentity {
    pub fn from_bridge_evidence(evidence_identity: &BridgeIdentityEvidence) -> Self {
        Self {
            value: Arc::from(format!(
                "bridge-naming-attachment:{}",
                evidence_identity.as_str()
            )),
        }
    }

    pub fn as_str(&self) -> &str {
        self.value.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeNamingAuthoritativeIdentity {
    value: Arc<str>,
}

impl BridgeNamingAuthoritativeIdentity {
    pub fn from_bridge_evidence(evidence_identity: &BridgeIdentityEvidence) -> Self {
        Self {
            value: Arc::from(format!(
                "bridge-naming-authoritative:{}",
                evidence_identity.as_str()
            )),
        }
    }

    pub fn as_str(&self) -> &str {
        self.value.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeNamingResolvedTargetIdentity {
    value: Arc<str>,
    parts: RelationalBridgeRecordIdentityParts,
}

impl BridgeNamingResolvedTargetIdentity {
    pub fn from_relational_record(parts: RelationalBridgeRecordIdentityParts) -> Self {
        Self {
            value: Arc::from(parts.bridge_entity_identity()),
            parts,
        }
    }

    pub fn as_str(&self) -> &str {
        self.value.as_ref()
    }

    pub fn relational_record_parts(&self) -> RelationalBridgeRecordIdentityParts {
        self.parts
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeNamingTargetCollection {
    value: Arc<str>,
}

impl BridgeNamingTargetCollection {
    pub fn new(value: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
        }
    }

    pub fn as_str(&self) -> &str {
        self.value.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeNamingMutationBundle {
    family: BridgeNamingMutationFamily,
    outcome: BridgeNamingMutationOutcome,
    attachment_identity: BridgeNamingAttachmentIdentity,
    prior_authoritative_identity: Option<BridgeNamingAuthoritativeIdentity>,
    target_authoritative_identity: Option<BridgeNamingAuthoritativeIdentity>,
    resolved_target_entity_identity: Option<BridgeNamingResolvedTargetIdentity>,
    target_collection: Option<BridgeNamingTargetCollection>,
}

impl BridgeNamingMutationBundle {
    pub fn attach_new_target(
        attachment_identity: BridgeNamingAttachmentIdentity,
        resolved_target_entity_identity: BridgeNamingResolvedTargetIdentity,
        target_collection: Option<BridgeNamingTargetCollection>,
    ) -> Self {
        Self {
            family: BridgeNamingMutationFamily::AttachNewTarget,
            outcome: BridgeNamingMutationOutcome::AttachedToNewTarget,
            attachment_identity,
            prior_authoritative_identity: None,
            target_authoritative_identity: None,
            resolved_target_entity_identity: Some(resolved_target_entity_identity),
            target_collection,
        }
    }

    pub fn attach_existing_target(
        attachment_identity: BridgeNamingAttachmentIdentity,
        target_authoritative_identity: BridgeNamingAuthoritativeIdentity,
        resolved_target_entity_identity: BridgeNamingResolvedTargetIdentity,
        target_collection: Option<BridgeNamingTargetCollection>,
    ) -> Self {
        Self {
            family: BridgeNamingMutationFamily::AttachExistingTarget,
            outcome: BridgeNamingMutationOutcome::AttachedToExistingTarget,
            attachment_identity,
            prior_authoritative_identity: None,
            target_authoritative_identity: Some(target_authoritative_identity),
            resolved_target_entity_identity: Some(resolved_target_entity_identity),
            target_collection,
        }
    }

    pub fn rebind_target(
        attachment_identity: BridgeNamingAttachmentIdentity,
        prior_authoritative_identity: BridgeNamingAuthoritativeIdentity,
        target_authoritative_identity: BridgeNamingAuthoritativeIdentity,
        resolved_target_entity_identity: BridgeNamingResolvedTargetIdentity,
        target_collection: Option<BridgeNamingTargetCollection>,
    ) -> Self {
        Self {
            family: BridgeNamingMutationFamily::RebindTarget,
            outcome: BridgeNamingMutationOutcome::ReboundTarget,
            attachment_identity,
            prior_authoritative_identity: Some(prior_authoritative_identity),
            target_authoritative_identity: Some(target_authoritative_identity),
            resolved_target_entity_identity: Some(resolved_target_entity_identity),
            target_collection,
        }
    }

    pub fn remove(
        attachment_identity: BridgeNamingAttachmentIdentity,
        prior_authoritative_identity: BridgeNamingAuthoritativeIdentity,
        resolved_target_entity_identity: Option<BridgeNamingResolvedTargetIdentity>,
        target_collection: Option<BridgeNamingTargetCollection>,
    ) -> Self {
        Self {
            family: BridgeNamingMutationFamily::Remove,
            outcome: BridgeNamingMutationOutcome::Removed,
            attachment_identity,
            prior_authoritative_identity: Some(prior_authoritative_identity),
            target_authoritative_identity: None,
            resolved_target_entity_identity,
            target_collection,
        }
    }

    pub fn family(&self) -> BridgeNamingMutationFamily {
        self.family
    }

    pub fn outcome(&self) -> BridgeNamingMutationOutcome {
        self.outcome
    }

    pub fn attachment_identity(&self) -> &str {
        self.attachment_identity.as_str()
    }

    pub fn attachment_identity_handle(&self) -> &BridgeNamingAttachmentIdentity {
        &self.attachment_identity
    }

    pub fn prior_authoritative_identity(&self) -> Option<&str> {
        self.prior_authoritative_identity
            .as_ref()
            .map(BridgeNamingAuthoritativeIdentity::as_str)
    }

    pub fn prior_authoritative_identity_handle(
        &self,
    ) -> Option<&BridgeNamingAuthoritativeIdentity> {
        self.prior_authoritative_identity.as_ref()
    }

    pub fn target_authoritative_identity(&self) -> Option<&str> {
        self.target_authoritative_identity
            .as_ref()
            .map(BridgeNamingAuthoritativeIdentity::as_str)
    }

    pub fn target_authoritative_identity_handle(
        &self,
    ) -> Option<&BridgeNamingAuthoritativeIdentity> {
        self.target_authoritative_identity.as_ref()
    }

    pub fn resolved_target_entity_identity(&self) -> Option<&str> {
        self.resolved_target_entity_identity
            .as_ref()
            .map(BridgeNamingResolvedTargetIdentity::as_str)
    }

    pub fn resolved_target_entity_identity_handle(
        &self,
    ) -> Option<&BridgeNamingResolvedTargetIdentity> {
        self.resolved_target_entity_identity.as_ref()
    }

    pub fn target_collection(&self) -> Option<&str> {
        self.target_collection
            .as_ref()
            .map(BridgeNamingTargetCollection::as_str)
    }

    pub fn target_collection_handle(&self) -> Option<&BridgeNamingTargetCollection> {
        self.target_collection.as_ref()
    }
}
