use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::memory_workspace::WorthQueryEntityIdentity;
use crate::runtime::{
    WorthQueryMutationAuthorityIdentity, WorthQueryMutationEvidenceDigest,
    WorthQueryMutationTargetCollectionIdentity,
};

pub(super) struct ContinuityDigestInput<'a> {
    pub(super) family: &'a str,
    pub(super) outcome: &'a str,
    pub(super) prior: &'a WorthQueryMutationAuthorityIdentity,
    pub(super) successors: &'a [WorthQueryMutationAuthorityIdentity],
    pub(super) basis_binding: Option<&'a WorthQueryMutationEvidenceDigest>,
    pub(super) resolved_target: Option<&'a WorthQueryEntityIdentity>,
    pub(super) target_collection: Option<&'a WorthQueryMutationTargetCollectionIdentity>,
}

pub(super) fn continuity_digests(
    input: ContinuityDigestInput<'_>,
) -> Result<
    (
        WorthQueryMutationEvidenceDigest,
        WorthQueryMutationEvidenceDigest,
    ),
    IdentityDerivationDenial,
> {
    let lineage_digest = continuity_lineage_digest(&input)?;
    let resolution_digest = continuity_resolution_digest(&input, &lineage_digest)?;
    Ok((lineage_digest, resolution_digest))
}

fn continuity_lineage_digest(
    input: &ContinuityDigestInput<'_>,
) -> Result<WorthQueryMutationEvidenceDigest, IdentityDerivationDenial> {
    let lineage_identity =
        worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
            .field_shape(WorthQueryEvidenceTag::new("role"), "continuity-lineage")
            .field_shape(WorthQueryEvidenceTag::new("family"), input.family)
            .field_shape(WorthQueryEvidenceTag::new("outcome"), input.outcome)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("prior"),
                input.prior.evidence_identity(),
            )
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("successor"),
                input
                    .successors
                    .iter()
                    .map(WorthQueryMutationAuthorityIdentity::evidence_identity),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("basis_binding"),
                input
                    .basis_binding
                    .map(WorthQueryMutationEvidenceDigest::evidence_identity),
            )
            .seal()?;
    WorthQueryMutationEvidenceDigest::aggregate("continuity-lineage", lineage_identity)
}

fn continuity_resolution_digest(
    input: &ContinuityDigestInput<'_>,
    lineage_digest: &WorthQueryMutationEvidenceDigest,
) -> Result<WorthQueryMutationEvidenceDigest, IdentityDerivationDenial> {
    let resolution_identity =
        worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
            .field_shape(WorthQueryEvidenceTag::new("role"), "continuity-resolution")
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("lineage"),
                lineage_digest.evidence_identity(),
            )
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("successor"),
                input
                    .successors
                    .iter()
                    .map(WorthQueryMutationAuthorityIdentity::evidence_identity),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("basis_binding"),
                input
                    .basis_binding
                    .map(WorthQueryMutationEvidenceDigest::evidence_identity),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("resolved"),
                input
                    .resolved_target
                    .map(WorthQueryEntityIdentity::evidence_identity)
                    .as_ref(),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("collection"),
                input
                    .target_collection
                    .map(WorthQueryMutationTargetCollectionIdentity::evidence_identity),
            )
            .seal()?;
    WorthQueryMutationEvidenceDigest::aggregate("continuity-resolution", resolution_identity)
}
