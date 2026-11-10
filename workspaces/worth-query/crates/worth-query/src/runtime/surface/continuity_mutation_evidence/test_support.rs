use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::memory_workspace::WorthQueryEntityIdentity;
use crate::runtime::{
    WorthQueryMutationAuthorityIdentity, WorthQueryMutationEvidenceDigest,
    WorthQueryMutationTargetCollectionIdentity,
};

use super::{WorthQueryContinuityMutationEvidence, WorthQueryContinuityOutcomeClass};

impl WorthQueryContinuityMutationEvidence {
    pub(crate) fn with_test_family(
        mut self,
        family: crate::runtime::WorthQueryContinuityMutationFamily,
    ) -> Self {
        self.family = family;
        self
    }

    pub(crate) fn test_only(
        family: crate::runtime::WorthQueryContinuityMutationFamily,
        outcome_class: WorthQueryContinuityOutcomeClass,
        prior_authoritative_identity: impl Into<String>,
        successor_authoritative_identities: Vec<String>,
        resolved_target_entity_identity: Option<WorthQueryEntityIdentity>,
        target_collection: Option<&str>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self {
            family,
            outcome_class,
            prior_authoritative_identity: WorthQueryMutationAuthorityIdentity::new(
                "continuity-prior",
                prior_authoritative_identity,
            )?,
            successor_authoritative_identities: successor_authoritative_identities
                .into_iter()
                .map(|identity| {
                    WorthQueryMutationAuthorityIdentity::new("continuity-successor", identity)
                })
                .collect::<Result<Vec<_>, IdentityDerivationDenial>>()?,
            basis_binding_digest: Some(test_only_digest(
                "continuity-basis-binding",
                "basis-binding:test",
            )?),
            resolved_target_entity_identity,
            target_collection: target_collection.map(|collection| {
                WorthQueryMutationTargetCollectionIdentity::new("continuity-target", collection)
                    .expect("fixture digest must fit")
            }),
            lineage_digest: test_only_digest("continuity-lineage", "lineage:test")?,
            continuity_resolution_digest: test_only_digest(
                "continuity-resolution",
                "continuity-resolution:test",
            )?,
        })
    }
}

fn test_only_digest(
    role: &'static str,
    value: &str,
) -> Result<WorthQueryMutationEvidenceDigest, IdentityDerivationDenial> {
    let identity =
        worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
            .field_shape(WorthQueryEvidenceTag::new("role"), role)
            .field_value(WorthQueryEvidenceTag::new("test_value"), value)
            .seal()?;
    WorthQueryMutationEvidenceDigest::aggregate(role, identity)
}
