use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::memory_workspace::WorthQueryEntityIdentity;
use crate::runtime::{
    WorthQueryMutationAuthorityIdentity, WorthQueryMutationEvidenceDigest,
    WorthQueryMutationTargetCollectionIdentity,
};
use worth_runtime_bridge::facade::{
    BridgeContinuityMutationBundle, BridgeContinuityMutationFamily,
    RelationalBridgeRecordIdentityParts,
};
#[path = "continuity_mutation_evidence/digests.rs"]
mod digests;
#[path = "continuity_mutation_evidence/taxonomy.rs"]
mod taxonomy;
use digests::{continuity_digests, ContinuityDigestInput};
use taxonomy::map_outcome_class;
pub use taxonomy::{
    WorthQueryContinuityClass, WorthQueryContinuityOutcomeClass, WorthQueryContinuityRejectionClass,
};

#[cfg(test)]
#[path = "continuity_mutation_evidence/test_support.rs"]
#[cfg(test)]
mod test_support;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryContinuityMutationEvidence {
    family: crate::runtime::WorthQueryContinuityMutationFamily,
    outcome_class: WorthQueryContinuityOutcomeClass,
    prior_authoritative_identity: WorthQueryMutationAuthorityIdentity,
    successor_authoritative_identities: Vec<WorthQueryMutationAuthorityIdentity>,
    basis_binding_digest: Option<WorthQueryMutationEvidenceDigest>,
    resolved_target_entity_identity: Option<WorthQueryEntityIdentity>,
    target_collection: Option<WorthQueryMutationTargetCollectionIdentity>,
    lineage_digest: WorthQueryMutationEvidenceDigest,
    continuity_resolution_digest: WorthQueryMutationEvidenceDigest,
}

impl WorthQueryContinuityMutationEvidence {
    #[cfg(test)]
    pub(in crate::runtime) fn from_bridge(
        bundle: &BridgeContinuityMutationBundle,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::from_bridge_with_query_context(
            bundle, None, None, None,
        )?)
    }

    pub(in crate::runtime) fn from_bridge_with_query_context(
        bundle: &BridgeContinuityMutationBundle,
        basis_binding_identity: Option<&WorthQueryEvidenceIdentity>,
        resolved_target_entity_identity: Option<&WorthQueryEntityIdentity>,
        target_collection: Option<&WorthQueryMutationTargetCollectionIdentity>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let family = match bundle.family() {
            BridgeContinuityMutationFamily::RebindExistingTarget => {
                crate::runtime::WorthQueryContinuityMutationFamily::RebindExistingTarget
            }
            BridgeContinuityMutationFamily::SplitExistingTarget => {
                crate::runtime::WorthQueryContinuityMutationFamily::SplitExistingTarget
            }
        };
        let outcome_class = map_outcome_class(bundle.outcome_class());
        let prior_authoritative_identity =
            WorthQueryMutationAuthorityIdentity::from_bridge_continuity_authority(
                "continuity-prior",
                bundle.prior_authoritative_identity_handle(),
            )?;
        let successor_authoritative_identities = bundle
            .successor_authoritative_identities()
            .iter()
            .map(|value| {
                WorthQueryMutationAuthorityIdentity::from_bridge_continuity_authority(
                    "continuity-successor",
                    value,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let basis_binding_digest = basis_binding_identity
            .map(|identity| {
                WorthQueryMutationEvidenceDigest::source_identity(
                    "continuity-basis-binding",
                    identity,
                )
            })
            .transpose()?;
        let resolved_target_entity_identity = bundle
            .resolved_target_entity_identity()
            .and_then(RelationalBridgeRecordIdentityParts::from_bridge_entity_identity)
            .map(WorthQueryEntityIdentity::from_runtime_receipt_record)
            .or_else(|| resolved_target_entity_identity.cloned());
        let target_collection = bundle
            .target_collection()
            .map(|collection| {
                WorthQueryMutationTargetCollectionIdentity::new("continuity-target", collection)
            })
            .transpose()?
            .or_else(|| target_collection.cloned());
        let (lineage_digest, continuity_resolution_digest) =
            continuity_digests(ContinuityDigestInput {
                family: family.as_str(),
                outcome: outcome_class.as_str(),
                prior: &prior_authoritative_identity,
                successors: &successor_authoritative_identities,
                basis_binding: basis_binding_digest.as_ref(),
                resolved_target: resolved_target_entity_identity.as_ref(),
                target_collection: target_collection.as_ref(),
            })?;
        Ok(Self {
            family,
            outcome_class,
            prior_authoritative_identity,
            successor_authoritative_identities,
            basis_binding_digest,
            resolved_target_entity_identity,
            target_collection,
            lineage_digest,
            continuity_resolution_digest,
        })
    }

    pub(crate) fn from_intent(
        intent: &crate::runtime::WorthQueryContinuityMutationIntent,
        basis_binding_identity: Option<&WorthQueryEvidenceIdentity>,
        resolved_target_entity_identity: Option<&WorthQueryEntityIdentity>,
        target_collection: Option<&WorthQueryMutationTargetCollectionIdentity>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let outcome_class = match intent.outcome_class() {
            crate::runtime::WorthQueryContinuityMutationOutcomeClass::ContinuesAsSingleSuccessor => {
                WorthQueryContinuityOutcomeClass::ContinuesAsSingleSuccessor
            }
            crate::runtime::WorthQueryContinuityMutationOutcomeClass::ContinuesAsSplitSuccessors => {
                WorthQueryContinuityOutcomeClass::ContinuesAsSplitSuccessors
            }
            crate::runtime::WorthQueryContinuityMutationOutcomeClass::ContinuesViaTruthLoweredCanonicalMergeSuccessor => {
                WorthQueryContinuityOutcomeClass::ContinuesViaTruthLoweredCanonicalMergeSuccessor
            }
        };
        let prior_authoritative_identity = intent.prior_authoritative_identity().clone();
        let successor_authoritative_identities =
            intent.successor_authoritative_identities().to_vec();
        let basis_binding_digest = basis_binding_identity
            .map(|identity| {
                WorthQueryMutationEvidenceDigest::source_identity(
                    "continuity-basis-binding",
                    identity,
                )
            })
            .transpose()?;
        let target_collection = target_collection.cloned();
        let (lineage_digest, continuity_resolution_digest) =
            continuity_digests(ContinuityDigestInput {
                family: intent.family().as_str(),
                outcome: intent.outcome_class().as_str(),
                prior: &prior_authoritative_identity,
                successors: &successor_authoritative_identities,
                basis_binding: basis_binding_digest.as_ref(),
                resolved_target: resolved_target_entity_identity,
                target_collection: target_collection.as_ref(),
            })?;
        Ok(Self {
            family: intent.family(),
            outcome_class,
            prior_authoritative_identity,
            successor_authoritative_identities,
            basis_binding_digest,
            resolved_target_entity_identity: resolved_target_entity_identity.cloned(),
            target_collection,
            lineage_digest,
            continuity_resolution_digest,
        })
    }

    pub fn family(&self) -> crate::runtime::WorthQueryContinuityMutationFamily {
        self.family
    }

    pub fn outcome_class(&self) -> WorthQueryContinuityOutcomeClass {
        self.outcome_class
    }

    pub fn continuity_class(&self) -> Option<WorthQueryContinuityClass> {
        self.outcome_class.continuity_class()
    }

    pub fn rejection_class(&self) -> Option<WorthQueryContinuityRejectionClass> {
        self.outcome_class.rejection_class()
    }

    pub fn prior_authoritative_identity(&self) -> &WorthQueryMutationAuthorityIdentity {
        &self.prior_authoritative_identity
    }

    pub fn successor_authoritative_identity(&self) -> Option<&WorthQueryMutationAuthorityIdentity> {
        match self.successor_authoritative_identities.as_slice() {
            [only] => Some(only),
            _ => None,
        }
    }

    pub fn successor_authoritative_identities(&self) -> &[WorthQueryMutationAuthorityIdentity] {
        &self.successor_authoritative_identities
    }

    pub fn basis_binding_digest(&self) -> Option<&WorthQueryMutationEvidenceDigest> {
        self.basis_binding_digest.as_ref()
    }

    pub fn resolved_target_entity_identity(&self) -> Option<&WorthQueryEntityIdentity> {
        self.resolved_target_entity_identity.as_ref()
    }

    pub fn target_collection(&self) -> Option<&WorthQueryMutationTargetCollectionIdentity> {
        self.target_collection.as_ref()
    }

    pub fn lineage_digest(&self) -> &WorthQueryMutationEvidenceDigest {
        &self.lineage_digest
    }

    pub fn continuity_resolution_digest(&self) -> &WorthQueryMutationEvidenceDigest {
        &self.continuity_resolution_digest
    }
}
