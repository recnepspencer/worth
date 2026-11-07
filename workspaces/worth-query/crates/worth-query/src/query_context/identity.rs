use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

use super::basis::{AdmittedQueryBasisContext, ComparisonBasisFamily, QueryContextFamily};
use super::comparison::{AdmittedDiffQueryContext, QueryDiffChangeSetArtifact};
use super::execution::QueryContextExecutionArtifact;
use super::performance::QueryContextPredictionDriftOutcome;
use super::support::QueryContextDeferredScopeMarker;

pub(crate) fn compose_preview_derived_result_shape_digest(
    validated_query_digest: &str,
    shape_check_width: usize,
) -> Result<String, IdentityDerivationDenial> {
    Ok(WorthQueryEvidenceIdentity::compose(
        WorthQueryEvidenceScope::QueryContextCompatibilityBasisLabel,
    )
    .field_shape(
        WorthQueryEvidenceTag::new("identity_family"),
        "query_context_preview_derived_result_shape_v1",
    )
    .field_shape(
        WorthQueryEvidenceTag::new("validated_query"),
        validated_query_digest,
    )
    .field_usize(
        WorthQueryEvidenceTag::new("shape_check_width"),
        shape_check_width,
    )
    .field_shape(
        WorthQueryEvidenceTag::new("shape_class"),
        "preview_query_context_shape",
    )
    .seal()?
    .as_str()
    .to_string())
}

pub(crate) fn compose_query_basis_replay_digest(
    context: &AdmittedQueryBasisContext,
    result_digest: &str,
    metadata_result_digest: &str,
    prediction_drift: Option<&QueryContextPredictionDriftOutcome>,
) -> Result<String, IdentityDerivationDenial> {
    Ok(WorthQueryEvidenceIdentity::compose(
        WorthQueryEvidenceScope::QueryContextCompatibilityBasisLabel,
    )
    .field_shape(
        WorthQueryEvidenceTag::new("identity_family"),
        "query_context_basis_replay_v1",
    )
    .field_shape(WorthQueryEvidenceTag::new("query"), context.query_digest())
    .field_shape(WorthQueryEvidenceTag::new("basis"), context.basis_digest())
    .field_shape(
        WorthQueryEvidenceTag::new("family"),
        context.family().as_str(),
    )
    .field_shape(WorthQueryEvidenceTag::new("result"), result_digest)
    .field_shape(
        WorthQueryEvidenceTag::new("metadata_result"),
        metadata_result_digest,
    )
    .field_shape(
        WorthQueryEvidenceTag::new("prediction"),
        prediction_drift
            .map(QueryContextPredictionDriftOutcome::as_str)
            .unwrap_or("none"),
    )
    .seal()?
    .as_str()
    .to_string())
}

pub(crate) fn compose_query_basis_counter_snapshot_digest(
    context: &AdmittedQueryBasisContext,
    execution: &QueryContextExecutionArtifact,
) -> String {
    WorthQueryEvidenceIdentity::compose(
        WorthQueryEvidenceScope::QueryContextCompatibilityBasisLabel,
    )
    .field_shape(
        WorthQueryEvidenceTag::new("identity_family"),
        "query_context_basis_counter_snapshot_v1",
    )
    .field_usize(
        WorthQueryEvidenceTag::new("binding_count"),
        context.counters().query_basis_binding_count(),
    )
    .field_usize(
        WorthQueryEvidenceTag::new("historical_lookup"),
        context.counters().historical_basis_lookup_count(),
    )
    .field_usize(
        WorthQueryEvidenceTag::new("binding_width"),
        context.counters().basis_binding_width(),
    )
    .field_usize(
        WorthQueryEvidenceTag::new("historical_width"),
        context.counters().historical_lookup_width(),
    )
    .field_usize(
        WorthQueryEvidenceTag::new("execution_count"),
        execution.counters().context_execution_count(),
    )
    .field_usize(
        WorthQueryEvidenceTag::new("materialized_rows"),
        execution.counters().materialized_row_count(),
    )
    .field_usize(
        WorthQueryEvidenceTag::new("result_shape_width"),
        execution.counters().result_shape_width(),
    )
    .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
    .as_str()
    .to_string()
}

pub(crate) fn compose_query_diff_replay_digest(
    context: &AdmittedDiffQueryContext,
    comparison_result_digest: &str,
    prediction_drift: &QueryContextPredictionDriftOutcome,
) -> Result<String, IdentityDerivationDenial> {
    Ok(WorthQueryEvidenceIdentity::compose(
        WorthQueryEvidenceScope::QueryContextCompatibilityBasisLabel,
    )
    .field_shape(
        WorthQueryEvidenceTag::new("identity_family"),
        "query_context_diff_replay_v1",
    )
    .field_shape(
        WorthQueryEvidenceTag::new("query"),
        context.left().query_digest(),
    )
    .field_shape(
        WorthQueryEvidenceTag::new("comparison_family"),
        context.family().as_str(),
    )
    .field_shape(
        WorthQueryEvidenceTag::new("left_basis"),
        context.left().basis_digest(),
    )
    .field_shape(
        WorthQueryEvidenceTag::new("right_basis"),
        context.right().basis_digest(),
    )
    .field_shape(
        WorthQueryEvidenceTag::new("comparison_result"),
        comparison_result_digest,
    )
    .field_shape(
        WorthQueryEvidenceTag::new("prediction"),
        prediction_drift.as_str(),
    )
    .seal()?
    .as_str()
    .to_string())
}

pub(crate) fn compose_query_diff_counter_snapshot_digest(
    context: &AdmittedDiffQueryContext,
    change_set: &QueryDiffChangeSetArtifact,
) -> String {
    WorthQueryEvidenceIdentity::compose(
        WorthQueryEvidenceScope::QueryContextCompatibilityBasisLabel,
    )
    .field_shape(
        WorthQueryEvidenceTag::new("identity_family"),
        "query_context_diff_counter_snapshot_v1",
    )
    .field_usize(
        WorthQueryEvidenceTag::new("comparison_lookups"),
        context.counters().comparison_basis_lookup_count(),
    )
    .field_usize(
        WorthQueryEvidenceTag::new("comparison_scope_width"),
        context.counters().comparison_scope_width(),
    )
    .field_usize(
        WorthQueryEvidenceTag::new("comparison_row_width"),
        context.counters().comparison_row_width(),
    )
    .field_usize(
        WorthQueryEvidenceTag::new("diff_input_breadth"),
        context.counters().diff_input_breadth(),
    )
    .field_usize(
        WorthQueryEvidenceTag::new("comparison_broadening_denials"),
        context.counters().comparison_broadening_denial_count(),
    )
    .field_usize(
        WorthQueryEvidenceTag::new("change_rows"),
        change_set.rows().len(),
    )
    .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
    .as_str()
    .to_string()
}

pub(crate) fn compose_query_context_support_profile_digest(
    admitted_basis_families: &[QueryContextFamily],
    admitted_comparison_families: &[ComparisonBasisFamily],
    _deferred_scope_markers: &[QueryContextDeferredScopeMarker],
) -> Result<String, IdentityDerivationDenial> {
    Ok(WorthQueryEvidenceIdentity::compose(
        WorthQueryEvidenceScope::QueryContextCompatibilityBasisLabel,
    )
    .field_shape(
        WorthQueryEvidenceTag::new("identity_family"),
        "query_context_support_profile_v1",
    )
    .field_value_sequence(
        WorthQueryEvidenceTag::new("basis_family"),
        admitted_basis_families
            .iter()
            .map(QueryContextFamily::as_str),
    )
    .field_value_sequence(
        WorthQueryEvidenceTag::new("comparison_family"),
        admitted_comparison_families
            .iter()
            .map(ComparisonBasisFamily::as_str),
    )
    .seal()?
    .as_str()
    .to_string())
}

/// Typed composition for construction-test branch/preview basis preparation.
pub fn compose_construction_branch_basis_preparation_digest(
    construction_family: &str,
    branch_preview_contract_digest: &str,
    preview_basis_admission: &crate::WorthQueryEvidenceIdentity,
    branch_basis_admission: &crate::WorthQueryEvidenceIdentity,
) -> Result<String, IdentityDerivationDenial> {
    Ok(WorthQueryEvidenceIdentity::compose(
        WorthQueryEvidenceScope::QueryContextCompatibilityBasisLabel,
    )
    .field_shape(
        WorthQueryEvidenceTag::new("identity_family"),
        "worth_kernel_branch_basis_preparation_v1",
    )
    .field_shape(
        WorthQueryEvidenceTag::new("construction_family"),
        construction_family,
    )
    .field_shape(
        WorthQueryEvidenceTag::new("branch_preview_contract"),
        branch_preview_contract_digest,
    )
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("preview_basis_admission"),
        preview_basis_admission,
    )
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("branch_basis_admission"),
        branch_basis_admission,
    )
    .seal()?
    .as_str()
    .to_string())
}
