use std::sync::Arc;

use crate::identity::BridgeIdentityEvidence;
use crate::relational_source::identity_parts::RelationalBridgeRecordIdentityParts;

use super::digest::existing_truth_binding_digest;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeExistingTruthBindingFamily {
    DirectEntityIdentity,
    DirectRelationIdentity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeExistingTruthBindingOutcome {
    ExistingAuthoritativeTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeExistingTruthBindingAuthoritativeIdentity {
    value: Arc<str>,
}

impl BridgeExistingTruthBindingAuthoritativeIdentity {
    pub fn from_bridge_evidence(evidence_identity: &BridgeIdentityEvidence) -> Self {
        Self {
            value: Arc::from(format!(
                "bridge-existing-truth-authority:{}",
                evidence_identity.as_str()
            )),
        }
    }

    pub fn as_str(&self) -> &str {
        self.value.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeExistingTruthBindingResolvedTargetIdentity {
    value: Arc<str>,
    parts: RelationalBridgeRecordIdentityParts,
}

impl BridgeExistingTruthBindingResolvedTargetIdentity {
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
pub struct BridgeExistingTruthBindingTargetCollection {
    value: Arc<str>,
}

impl BridgeExistingTruthBindingTargetCollection {
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
pub struct BridgeExistingTruthBindingBundle {
    family: BridgeExistingTruthBindingFamily,
    outcome: BridgeExistingTruthBindingOutcome,
    authoritative_identity: BridgeExistingTruthBindingAuthoritativeIdentity,
    resolved_target_identity: BridgeExistingTruthBindingResolvedTargetIdentity,
    target_collection: Option<BridgeExistingTruthBindingTargetCollection>,
    binding_digest: Arc<str>,
}

impl BridgeExistingTruthBindingBundle {
    pub fn direct_entity(
        authoritative_identity: BridgeExistingTruthBindingAuthoritativeIdentity,
        resolved_entity_identity: BridgeExistingTruthBindingResolvedTargetIdentity,
        target_collection: Option<BridgeExistingTruthBindingTargetCollection>,
    ) -> Self {
        Self::new(
            BridgeExistingTruthBindingFamily::DirectEntityIdentity,
            authoritative_identity,
            resolved_entity_identity,
            target_collection,
        )
    }

    pub fn direct_relation(
        authoritative_identity: BridgeExistingTruthBindingAuthoritativeIdentity,
        resolved_relation_identity: BridgeExistingTruthBindingResolvedTargetIdentity,
        target_collection: Option<BridgeExistingTruthBindingTargetCollection>,
    ) -> Self {
        Self::new(
            BridgeExistingTruthBindingFamily::DirectRelationIdentity,
            authoritative_identity,
            resolved_relation_identity,
            target_collection,
        )
    }

    fn new(
        family: BridgeExistingTruthBindingFamily,
        authoritative_identity: BridgeExistingTruthBindingAuthoritativeIdentity,
        resolved_target_identity: BridgeExistingTruthBindingResolvedTargetIdentity,
        target_collection: Option<BridgeExistingTruthBindingTargetCollection>,
    ) -> Self {
        let target_collection_basis: &str = match target_collection.as_ref() {
            Some(value) => value.as_str(),
            None => "none",
        };
        let binding_digest = existing_truth_binding_digest([
            format!("family:{family:?}"),
            format!(
                "outcome:{:?}",
                BridgeExistingTruthBindingOutcome::ExistingAuthoritativeTarget
            ),
            format!("authoritative:{}", authoritative_identity.as_str()),
            format!("resolved:{}", resolved_target_identity.as_str()),
            format!("collection:{target_collection_basis}"),
        ]);
        Self {
            family,
            outcome: BridgeExistingTruthBindingOutcome::ExistingAuthoritativeTarget,
            authoritative_identity,
            resolved_target_identity,
            target_collection,
            binding_digest,
        }
    }

    pub fn family(&self) -> BridgeExistingTruthBindingFamily {
        self.family
    }

    pub fn outcome(&self) -> BridgeExistingTruthBindingOutcome {
        self.outcome
    }

    pub fn authoritative_identity(&self) -> &str {
        self.authoritative_identity.as_str()
    }

    pub fn authoritative_identity_handle(
        &self,
    ) -> &BridgeExistingTruthBindingAuthoritativeIdentity {
        &self.authoritative_identity
    }

    pub fn resolved_target_identity(&self) -> &str {
        self.resolved_target_identity.as_str()
    }

    pub fn resolved_target_identity_handle(
        &self,
    ) -> &BridgeExistingTruthBindingResolvedTargetIdentity {
        &self.resolved_target_identity
    }

    pub fn resolved_entity_identity(&self) -> &str {
        self.resolved_target_identity.as_str()
    }

    pub fn resolved_relation_identity(&self) -> &str {
        self.resolved_target_identity.as_str()
    }

    pub fn target_collection(&self) -> Option<&str> {
        self.target_collection
            .as_ref()
            .map(BridgeExistingTruthBindingTargetCollection::as_str)
    }

    pub fn target_collection_handle(&self) -> Option<&BridgeExistingTruthBindingTargetCollection> {
        self.target_collection.as_ref()
    }

    pub fn binding_digest(&self) -> &str {
        self.binding_digest.as_ref()
    }
}
