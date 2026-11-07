//! Production preview execution admission and envelope construction.

use crate::execution::execute_preflight_bundle;
use crate::preview::binding::contract::{
    PreviewSessionPlanBinding, PromotionEligiblePreviewSessionPlanBinding,
    ReadOnlyPreviewSessionPlanBinding,
};
use crate::preview::comparison::admission::derive_preview_comparison_eligibility;
use crate::preview::evaluation::PreviewEvaluationClass;
use crate::preview::execution::accounting::PreviewExecutionCounters;
use crate::preview::execution::outcome::{
    PreviewExecutionEnvelope, PreviewExecutionError, PreviewExecutionReport,
    PromotionEligiblePreviewExecutionEnvelope, ReadOnlyPreviewExecutionEnvelope,
};
use crate::preview::workflow_context_identity;
use crate::preview::workflow_foundation::admit_preview_workflow_foundation;

fn execute_preview_session_plan(
    binding: &PreviewSessionPlanBinding,
) -> Result<PreviewExecutionEnvelope, PreviewExecutionError> {
    let execution = execute_preflight_bundle(binding.preflight())
        .map_err(PreviewExecutionError::ExecutionFailure)?;
    let comparison_eligibility = derive_preview_comparison_eligibility(binding)?;
    let workflow_foundation = admit_preview_workflow_foundation(binding)
        .expect("preview execution should admit the comparison-basis workflow foundation");
    let binding_tuple = binding.basis().binding_tuple();
    let execution_record_identity = binding_tuple
        .execution_record_identity()
        .cloned()
        .expect("active preview bindings must carry an execution record identity");
    let is_promotion_eligible = matches!(
        binding_tuple.evaluation_class(),
        PreviewEvaluationClass::PromotionEligible(_)
    );
    let report = PreviewExecutionReport {
        preview_execution_digest:
            workflow_context_identity::compose_preview_execution_report_digest(
                binding_tuple.digest(),
                execution.report().basis_digest().as_str(),
                binding_tuple.preview_session_identity(),
                binding_tuple.lifecycle_state_kind(),
                &execution_record_identity,
                execution.report().result_digest().as_str(),
                comparison_eligibility.digest(),
                workflow_foundation.artifact().artifact_for_reporting(),
            )?,
        binding_digest: binding_tuple.digest().to_string(),
        basis_digest: execution.report().basis_digest().as_str().to_string(),
        preview_session_identity: binding_tuple.preview_session_identity().clone(),
        lifecycle_state_kind: binding_tuple.lifecycle_state_kind(),
        execution_record_identity,
        query_digest: execution.report().query_digest().clone(),
        result_digest: execution.report().result_digest().clone(),
        comparison_eligibility_digest: comparison_eligibility.digest().to_string(),
        workflow_foundation_digest: workflow_foundation
            .artifact()
            .artifact_for_reporting()
            .to_string(),
    };
    let envelope = PreviewExecutionEnvelope {
        binding: binding.clone(),
        counters: PreviewExecutionCounters {
            binding_counters: binding.report().counters().clone(),
            execution_counters: execution.counters().clone(),
            preview_execution_envelope_count: 1,
            preview_execution_count: 1,
            preview_promotable_execution_count: usize::from(is_promotion_eligible),
            preview_read_only_execution_count: usize::from(!is_promotion_eligible),
            preview_comparison_eligibility_proof_count: 1,
            preview_comparison_shape_check_width: comparison_eligibility.shape_check_width(),
            preview_workflow_foundation_admission_count: workflow_foundation
                .counters()
                .preview_workflow_foundation_admission_count(),
            preview_workflow_foundation_denial_count: workflow_foundation
                .counters()
                .preview_workflow_foundation_denial_count(),
            preview_workflow_foundation_artifact_lookup_count: workflow_foundation
                .counters()
                .preview_workflow_foundation_artifact_lookup_count(),
            preview_work_avoided_by_explicit_basis_count: workflow_foundation
                .counters()
                .preview_work_avoided_by_explicit_basis_count(),
        },
        execution,
        comparison_eligibility,
        workflow_foundation,
        report,
    };
    envelope.check_invariants()?;
    Ok(envelope)
}

pub fn admit_read_only_preview_session_plan_binding(
    binding: PreviewSessionPlanBinding,
) -> Result<ReadOnlyPreviewSessionPlanBinding, PreviewExecutionError> {
    if !matches!(
        binding.basis().binding_tuple().evaluation_class(),
        PreviewEvaluationClass::ReadOnly(_)
    ) {
        return Err(PreviewExecutionError::EvaluationClassMismatch {
            expected: "read_only",
            actual: binding.basis().binding_tuple().evaluation_class().as_str(),
        });
    }

    Ok(ReadOnlyPreviewSessionPlanBinding::from_admitted(binding))
}

pub fn admit_promotion_eligible_preview_session_plan_binding(
    binding: PreviewSessionPlanBinding,
) -> Result<PromotionEligiblePreviewSessionPlanBinding, PreviewExecutionError> {
    if !matches!(
        binding.basis().binding_tuple().evaluation_class(),
        PreviewEvaluationClass::PromotionEligible(_)
    ) {
        return Err(PreviewExecutionError::EvaluationClassMismatch {
            expected: "promotion_eligible",
            actual: binding.basis().binding_tuple().evaluation_class().as_str(),
        });
    }

    Ok(PromotionEligiblePreviewSessionPlanBinding::from_admitted(
        binding,
    ))
}

pub fn execute_read_only_preview_session_plan(
    binding: &ReadOnlyPreviewSessionPlanBinding,
) -> Result<ReadOnlyPreviewExecutionEnvelope, PreviewExecutionError> {
    Ok(ReadOnlyPreviewExecutionEnvelope {
        inner: execute_preview_session_plan(binding.as_preview_binding())?,
    })
}

pub fn execute_promotion_eligible_preview_session_plan(
    binding: &PromotionEligiblePreviewSessionPlanBinding,
) -> Result<PromotionEligiblePreviewExecutionEnvelope, PreviewExecutionError> {
    Ok(PromotionEligiblePreviewExecutionEnvelope {
        inner: execute_preview_session_plan(binding.as_preview_binding())?,
    })
}
