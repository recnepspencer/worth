use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use worth_query_declaration::facade::identity::{
    CanonicalQueryDigest, CanonicalResultShapeDigest, CollectionPlanDigest, ValidatedQueryDigest,
};

fn declaration_digest_evidence(
    family: &'static str,
    field: &'static str,
    digest: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceSourceDigest)
            .field_shape(WorthQueryEvidenceTag::new("identity_family"), family)
            .field_value(WorthQueryEvidenceTag::new(field), digest)
            .seal()?,
    )
}

pub(crate) fn canonical_query_evidence_identity(
    digest: &CanonicalQueryDigest,
) -> WorthQueryEvidenceIdentity {
    declaration_digest_evidence(
        "canonical_query_digest_v1",
        "canonical_query_digest",
        digest.as_str(),
    )
}

pub(crate) fn validated_query_evidence_identity(
    digest: &ValidatedQueryDigest,
) -> WorthQueryEvidenceIdentity {
    declaration_digest_evidence(
        "validated_query_digest_v1",
        "validated_query_digest",
        digest.as_str(),
    )
}

pub(crate) fn canonical_result_shape_evidence_identity(
    digest: &CanonicalResultShapeDigest,
) -> WorthQueryEvidenceIdentity {
    declaration_digest_evidence(
        "canonical_result_shape_digest_v1",
        "result_shape_digest",
        digest.as_str(),
    )
}

pub(crate) fn collection_plan_evidence_identity(
    digest: &CollectionPlanDigest,
) -> WorthQueryEvidenceIdentity {
    declaration_digest_evidence(
        "collection_plan_digest_v1",
        "collection_plan_digest",
        digest.as_str(),
    )
}
