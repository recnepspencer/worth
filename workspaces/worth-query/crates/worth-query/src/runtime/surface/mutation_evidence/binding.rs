use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::memory_workspace::WorthQueryEntityIdentity;
use crate::runtime::{
    WorthQueryMutationAuthorityIdentity, WorthQueryMutationEvidenceDigest,
    WorthQueryMutationTargetCollectionIdentity,
};
use worth_runtime_bridge::facade::{
    BridgeExistingTruthBindingBundle, BridgeExistingTruthBindingFamily,
    BridgeExistingTruthBindingOutcome,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryExistingTruthBindingOutcome {
    ExistingAuthoritativeTarget,
}

impl WorthQueryExistingTruthBindingOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ExistingAuthoritativeTarget => "existing-authoritative-target",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryExistingTruthBindingEvidence {
    family: crate::runtime::WorthQueryExistingTruthBindingFamily,
    outcome: WorthQueryExistingTruthBindingOutcome,
    authoritative_identity: WorthQueryMutationAuthorityIdentity,
    resolved_entity_identity: WorthQueryEntityIdentity,
    target_collection: Option<WorthQueryMutationTargetCollectionIdentity>,
    binding_digest: WorthQueryMutationEvidenceDigest,
}

impl WorthQueryExistingTruthBindingEvidence {
    pub(in crate::runtime) fn from_bridge(
        binding: &BridgeExistingTruthBindingBundle,
    ) -> Result<Self, IdentityDerivationDenial> {
        let family = match binding.family() {
            BridgeExistingTruthBindingFamily::DirectEntityIdentity => {
                crate::runtime::WorthQueryExistingTruthBindingFamily::DirectEntityIdentity
            }
            BridgeExistingTruthBindingFamily::DirectRelationIdentity => {
                crate::runtime::WorthQueryExistingTruthBindingFamily::DirectRelationIdentity
            }
        };
        let outcome = match binding.outcome() {
            BridgeExistingTruthBindingOutcome::ExistingAuthoritativeTarget => {
                WorthQueryExistingTruthBindingOutcome::ExistingAuthoritativeTarget
            }
        };
        let authoritative_identity =
            WorthQueryMutationAuthorityIdentity::from_bridge_existing_truth_authority(
                "existing-truth-binding-authority",
                binding.authoritative_identity_handle(),
            );
        let resolved_entity_identity = WorthQueryEntityIdentity::from_relational_record(
            binding
                .resolved_target_identity_handle()
                .relational_record_parts(),
        );
        let target_collection = binding.target_collection().map(|collection| {
            WorthQueryMutationTargetCollectionIdentity::new("existing-truth-binding", collection)
        });
        let binding_identity =
            worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
                .field_shape(WorthQueryEvidenceTag::new("role"), "existing-truth-binding")
                .field_shape(WorthQueryEvidenceTag::new("family"), family.as_str())
                .field_shape(WorthQueryEvidenceTag::new("outcome"), outcome.as_str())
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("authoritative"),
                    authoritative_identity.evidence_identity(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("resolved"),
                    &resolved_entity_identity.evidence_identity(),
                )
                .optional_evidence_identity(
                    WorthQueryEvidenceTag::new("collection"),
                    target_collection
                        .as_ref()
                        .map(WorthQueryMutationTargetCollectionIdentity::evidence_identity),
                )
                .seal()?;
        Ok(Self {
            family,
            outcome,
            authoritative_identity,
            resolved_entity_identity,
            target_collection,
            binding_digest: WorthQueryMutationEvidenceDigest::aggregate(
                "existing-truth-binding",
                binding_identity,
            ),
        })
    }

    pub(in crate::runtime) fn from_binding(
        binding: &crate::runtime::WorthQueryExistingTruthTargetBinding,
    ) -> Self {
        let resolved_entity_identity = binding
            .resolved_target_identity()
            .relational_record_parts()
            .map(WorthQueryEntityIdentity::from_relational_record)
            .expect(
                "existing-truth binding evidence must carry a relational record target identity",
            );
        Self {
            family: binding.family(),
            outcome: WorthQueryExistingTruthBindingOutcome::ExistingAuthoritativeTarget,
            authoritative_identity: binding.authoritative_identity().clone(),
            resolved_entity_identity,
            target_collection: binding.target_collection_identity().cloned(),
            binding_digest: WorthQueryMutationEvidenceDigest::source_identity(
                "existing-truth-binding",
                binding.binding_evidence_identity(),
            ),
        }
    }

    pub fn family(&self) -> crate::runtime::WorthQueryExistingTruthBindingFamily {
        self.family
    }

    pub fn outcome(&self) -> WorthQueryExistingTruthBindingOutcome {
        self.outcome
    }

    pub fn authoritative_identity(&self) -> &WorthQueryMutationAuthorityIdentity {
        &self.authoritative_identity
    }

    pub fn resolved_target_identity(&self) -> &WorthQueryEntityIdentity {
        &self.resolved_entity_identity
    }

    pub fn resolved_entity_identity(&self) -> &WorthQueryEntityIdentity {
        &self.resolved_entity_identity
    }

    pub fn resolved_relation_identity(&self) -> &WorthQueryEntityIdentity {
        &self.resolved_entity_identity
    }

    pub fn target_collection(&self) -> Option<&WorthQueryMutationTargetCollectionIdentity> {
        self.target_collection.as_ref()
    }

    pub fn binding_digest(&self) -> &WorthQueryMutationEvidenceDigest {
        &self.binding_digest
    }
}
