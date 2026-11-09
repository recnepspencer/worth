use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::identity::{CanonicalQueryDigest, CanonicalResultShapeDigest, ValidatedQueryDigest};
use crate::identity::{CollectionPlanDigest, ResultDigest};
use crate::workflow::{
    workflow_canonical_query_digest_evidence, workflow_validated_query_digest_evidence,
};
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

pub(in crate::preview) fn compose_preview_comparison_ordering_digest(
    parts: &[String],
) -> Result<String, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_preview_comparison_ordering_v1",
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("ordering_part"),
                parts.iter().map(String::as_str),
            )
            .seal()?
            .as_str()
            .to_string(),
    )
}

pub(in crate::preview) fn compose_preview_comparison_materialization_boundary_digest(
    parts: &[String],
) -> Result<String, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_preview_comparison_materialization_boundary_v1",
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("materialization_part"),
                parts.iter().map(String::as_str),
            )
            .seal()?
            .as_str()
            .to_string(),
    )
}

pub(in crate::preview) fn compose_preview_comparison_eligibility_digest(
    canonical_query_digest: &CanonicalQueryDigest,
    canonical_result_shape_digest: &CanonicalResultShapeDigest,
    collection_digest: Option<&CollectionPlanDigest>,
    result_family: &str,
    ordering_digest: &str,
    materialization_boundary_digest: &str,
    shape_check_width: usize,
) -> Result<String, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_preview_comparison_eligibility_v1",
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("canonical_query"),
                &workflow_canonical_query_digest_evidence(canonical_query_digest),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("canonical_result_shape"),
                canonical_result_shape_digest.as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("collection"),
                collection_digest
                    .map(CollectionPlanDigest::as_str)
                    .unwrap_or("detail"),
            )
            .field_shape(WorthQueryEvidenceTag::new("result_family"), result_family)
            .field_shape(WorthQueryEvidenceTag::new("ordering"), ordering_digest)
            .field_shape(
                WorthQueryEvidenceTag::new("materialization_boundary"),
                materialization_boundary_digest,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("shape_check_width"),
                shape_check_width,
            )
            .seal()?
            .as_str()
            .to_string(),
    )
}

pub(in crate::preview) fn compose_preview_comparison_candidate_digest(
    validated_query_digest: &ValidatedQueryDigest,
    result_digest: &ResultDigest,
    canonical_query_digest: &CanonicalQueryDigest,
    canonical_result_shape_digest: &CanonicalResultShapeDigest,
    collection_digest: Option<&CollectionPlanDigest>,
    result_family: &str,
    ordering_digest: &str,
    materialization_boundary_digest: &str,
    shape_check_width: usize,
) -> Result<String, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_preview_comparison_candidate_v1",
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("validated_query"),
                &workflow_validated_query_digest_evidence(validated_query_digest),
            )
            .field_shape(WorthQueryEvidenceTag::new("result"), result_digest.as_str())
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("canonical_query"),
                &workflow_canonical_query_digest_evidence(canonical_query_digest),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("canonical_result_shape"),
                canonical_result_shape_digest.as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("collection"),
                collection_digest
                    .map(CollectionPlanDigest::as_str)
                    .unwrap_or("detail"),
            )
            .field_shape(WorthQueryEvidenceTag::new("result_family"), result_family)
            .field_shape(WorthQueryEvidenceTag::new("ordering"), ordering_digest)
            .field_shape(
                WorthQueryEvidenceTag::new("materialization_boundary"),
                materialization_boundary_digest,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("shape_check_width"),
                shape_check_width,
            )
            .seal()?
            .as_str()
            .to_string(),
    )
}
