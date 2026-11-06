//! Production admission for comparing preview execution evidence.

use crate::basis::{BasisAuthorityFamily, ExecutionPreflightBundle};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::execution::ExecutionResultEnvelope;
use crate::preview::binding::contract::PreviewSessionPlanBinding;
use crate::preview::comparison::contract::{
    AuthoritativePreviewComparisonCandidate, PreviewComparisonCandidateArtifact,
    PreviewComparisonEligibilityArtifact, PreviewComparisonError, PreviewComparisonFailureClass,
    PreviewExecutionComparisonAdmission, PromotionParityPreviewComparisonAdmission,
};
use crate::preview::comparison::shape::PreviewComparisonShapeContract;
use crate::preview::execution::accounting::PreviewComparisonCounters;
use crate::preview::execution::outcome::{
    PreviewExecutionEnvelope, PromotionEligiblePreviewExecutionEnvelope,
};
use crate::preview::workflow_context_identity;

pub(crate) fn derive_preview_comparison_eligibility(
    binding: &PreviewSessionPlanBinding,
) -> Result<PreviewComparisonEligibilityArtifact, IdentityDerivationDenial> {
    let shape_contract = PreviewComparisonShapeContract::from_preflight(binding.preflight())?;

    let digest = workflow_context_identity::compose_preview_comparison_eligibility_digest(
        binding.basis().binding_tuple().canonical_query_digest(),
        binding
            .basis()
            .binding_tuple()
            .canonical_result_shape_digest(),
        shape_contract.collection_digest.as_ref(),
        &shape_contract.result_family,
        &shape_contract.ordering_digest,
        &shape_contract.materialization_boundary_digest,
        shape_contract.shape_check_width,
    )?;

    Ok(PreviewComparisonEligibilityArtifact {
        digest,
        canonical_query_digest: binding
            .basis()
            .binding_tuple()
            .canonical_query_digest()
            .clone(),
        canonical_result_shape_digest: binding
            .basis()
            .binding_tuple()
            .canonical_result_shape_digest()
            .clone(),
        collection_digest: shape_contract.collection_digest,
        result_family: shape_contract.result_family,
        ordering_digest: shape_contract.ordering_digest,
        materialization_boundary_digest: shape_contract.materialization_boundary_digest,
        shape_check_width: shape_contract.shape_check_width,
    })
}

fn derive_preview_comparison_candidate(
    preflight: &ExecutionPreflightBundle,
    execution: &ExecutionResultEnvelope,
) -> Result<PreviewComparisonCandidateArtifact, IdentityDerivationDenial> {
    let shape_contract = PreviewComparisonShapeContract::from_preflight(preflight)?;
    let digest = workflow_context_identity::compose_preview_comparison_candidate_digest(
        preflight.plan().query().validated_query_digest(),
        execution.report().result_digest(),
        preflight.plan().query().canonical_query_digest(),
        preflight
            .plan()
            .result_shape()
            .canonical_result_shape_digest(),
        shape_contract.collection_digest.as_ref(),
        &shape_contract.result_family,
        &shape_contract.ordering_digest,
        &shape_contract.materialization_boundary_digest,
        shape_contract.shape_check_width,
    )?;

    Ok(PreviewComparisonCandidateArtifact {
        digest,
        validated_query_digest: preflight.plan().query().validated_query_digest().clone(),
        basis_digest: preflight.basis().proof().digest().as_str().to_string(),
        result_digest: execution.report().result_digest().clone(),
        canonical_query_digest: preflight.plan().query().canonical_query_digest().clone(),
        canonical_result_shape_digest: preflight
            .plan()
            .result_shape()
            .canonical_result_shape_digest()
            .clone(),
        collection_digest: shape_contract.collection_digest,
        result_family: shape_contract.result_family,
        ordering_digest: shape_contract.ordering_digest,
        materialization_boundary_digest: shape_contract.materialization_boundary_digest,
        shape_check_width: shape_contract.shape_check_width,
    })
}

pub fn admit_authoritative_preview_comparison_candidate(
    candidate_preflight: &ExecutionPreflightBundle,
    candidate_execution: &ExecutionResultEnvelope,
) -> Result<AuthoritativePreviewComparisonCandidate, PreviewComparisonError> {
    let candidate = derive_preview_comparison_candidate(candidate_preflight, candidate_execution);
    let denial_counters = || PreviewComparisonCounters {
        preview_promotion_comparison_count: 0,
        preview_promotion_comparison_denial_count: 1,
        preview_comparison_eligibility_proof_count: 1,
        preview_comparison_shape_check_width: candidate.shape_check_width(),
        preview_basis_pair_width: 1,
    };

    if matches!(
        candidate_preflight.basis().identity().authority_family(),
        BasisAuthorityFamily::Store
    ) {
        return Err(PreviewComparisonError {
            failure_class: PreviewComparisonFailureClass::CandidateBasisAuthorityMismatch,
            message: "preview comparison only admits runtime-backed authoritative candidates",
            preview_digest: String::new(),
            candidate_digest: candidate.digest().to_string(),
            counters: denial_counters(),
        });
    }

    if candidate_execution.report().query_digest()
        != candidate_preflight.plan().query().validated_query_digest()
        || candidate_execution.report().plan_digest()
            != candidate_preflight.plan().query().plan_digest()
    {
        return Err(PreviewComparisonError {
            failure_class: PreviewComparisonFailureClass::CandidateExecutionPlanMismatch,
            message:
                "preview comparison candidates require execution from the same admitted query plan",
            preview_digest: String::new(),
            candidate_digest: candidate.digest().to_string(),
            counters: denial_counters(),
        });
    }

    if candidate_execution.report().basis_digest() != candidate_preflight.basis().proof().digest() {
        return Err(PreviewComparisonError {
            failure_class: PreviewComparisonFailureClass::CandidateExecutionBasisMismatch,
            message:
                "preview comparison candidates require execution from the same authoritative basis",
            preview_digest: String::new(),
            candidate_digest: candidate.digest().to_string(),
            counters: denial_counters(),
        });
    }

    Ok(AuthoritativePreviewComparisonCandidate {
        artifact: candidate,
    })
}

fn admit_preview_execution_comparison(
    preview_execution: &PreviewExecutionEnvelope,
    candidate: &AuthoritativePreviewComparisonCandidate,
) -> Result<PreviewExecutionComparisonAdmission, PreviewComparisonError> {
    let preview = preview_execution.comparison_eligibility();
    let candidate = candidate.artifact();
    let denial_counters = |shape_width: usize| PreviewComparisonCounters {
        preview_promotion_comparison_count: 0,
        preview_promotion_comparison_denial_count: 1,
        preview_comparison_eligibility_proof_count: 1,
        preview_comparison_shape_check_width: shape_width,
        preview_basis_pair_width: 2,
    };

    if preview_execution.report().query_digest() != candidate.validated_query_digest() {
        return Err(PreviewComparisonError {
            failure_class: PreviewComparisonFailureClass::QueryDigestMismatch,
            message: "preview comparison requires the same validated query digest on both sides",
            preview_digest: preview.digest().to_string(),
            candidate_digest: candidate.digest().to_string(),
            counters: denial_counters(candidate.shape_check_width()),
        });
    }

    if preview.canonical_query_digest() != candidate.canonical_query_digest()
        || preview.canonical_result_shape_digest() != candidate.canonical_result_shape_digest()
    {
        return Err(PreviewComparisonError {
            failure_class: PreviewComparisonFailureClass::ResultShapeMismatch,
            message:
                "preview comparison requires the same canonical query and result-shape digests",
            preview_digest: preview.digest().to_string(),
            candidate_digest: candidate.digest().to_string(),
            counters: denial_counters(
                preview
                    .shape_check_width()
                    .max(candidate.shape_check_width()),
            ),
        });
    }

    if preview.result_family() != candidate.result_family() {
        return Err(PreviewComparisonError {
            failure_class: PreviewComparisonFailureClass::ResultFamilyMismatch,
            message: "preview comparison requires the same result family on both sides",
            preview_digest: preview.digest().to_string(),
            candidate_digest: candidate.digest().to_string(),
            counters: denial_counters(
                preview
                    .shape_check_width()
                    .max(candidate.shape_check_width()),
            ),
        });
    }

    if preview.ordering_digest() != candidate.ordering_digest() {
        return Err(PreviewComparisonError {
            failure_class: PreviewComparisonFailureClass::OrderingBasisMismatch,
            message: "preview comparison requires identical ordering basis proofs",
            preview_digest: preview.digest().to_string(),
            candidate_digest: candidate.digest().to_string(),
            counters: denial_counters(
                preview
                    .shape_check_width()
                    .max(candidate.shape_check_width()),
            ),
        });
    }

    if preview.materialization_boundary_digest() != candidate.materialization_boundary_digest() {
        return Err(PreviewComparisonError {
            failure_class: PreviewComparisonFailureClass::MaterializationBoundaryMismatch,
            message: "preview comparison requires identical materialization boundary proofs",
            preview_digest: preview.digest().to_string(),
            candidate_digest: candidate.digest().to_string(),
            counters: denial_counters(
                preview
                    .shape_check_width()
                    .max(candidate.shape_check_width()),
            ),
        });
    }

    let shape_check_width = preview
        .shape_check_width()
        .max(candidate.shape_check_width());
    Ok(PreviewExecutionComparisonAdmission {
        digest: workflow_context_identity::compose_preview_execution_comparison_admission_digest(
            preview_execution.report().preview_execution_digest(),
            preview.digest(),
            candidate.digest(),
            candidate.basis_digest(),
            candidate.result_digest().as_str(),
        ),
        preview_execution_digest: preview_execution
            .report()
            .preview_execution_digest()
            .to_string(),
        preview_comparison_digest: preview.digest().to_string(),
        candidate_comparison_digest: candidate.digest().to_string(),
        canonical_query_digest: candidate.canonical_query_digest().clone(),
        validated_query_digest: candidate.validated_query_digest().clone(),
        candidate_basis_digest: candidate.basis_digest().to_string(),
        candidate_result_digest: candidate.result_digest().clone(),
        shape_check_width,
        counters: PreviewComparisonCounters {
            preview_promotion_comparison_count: 1,
            preview_promotion_comparison_denial_count: 0,
            preview_comparison_eligibility_proof_count: 1,
            preview_comparison_shape_check_width: shape_check_width,
            preview_basis_pair_width: 2,
        },
    })
}

pub fn admit_preview_promotion_parity_comparison(
    preview_execution: &PromotionEligiblePreviewExecutionEnvelope,
    candidate: &AuthoritativePreviewComparisonCandidate,
) -> Result<PromotionParityPreviewComparisonAdmission, PreviewComparisonError> {
    let admission =
        admit_preview_execution_comparison(preview_execution.as_preview_execution(), candidate)?;

    Ok(PromotionParityPreviewComparisonAdmission { inner: admission })
}
