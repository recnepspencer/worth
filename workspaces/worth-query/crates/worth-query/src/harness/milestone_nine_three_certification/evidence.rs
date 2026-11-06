use super::lanes::CertifiedLaneArtifacts;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::harness::certification::digest_parts;
use crate::subscription::*;

use super::{
    MilestoneNineThreeCertificationBundle, MilestoneNineThreeFailureClass,
    MilestoneNineThreeRejectionBundle, MILESTONE_NINE_THREE_REQUIRED_COMPILE_FAIL_TARGETS,
};

pub(super) struct SupportDeniedArtifacts {
    pub(super) denied_bundle: QuerySubscriptionDeniedDiagnosticBundle,
    pub(super) failure: QuerySubscriptionDiagnosticFailure,
}

pub(super) fn certification_bundle(
    artifacts: &CertifiedLaneArtifacts,
) -> MilestoneNineThreeCertificationBundle {
    MilestoneNineThreeCertificationBundle {
        query_family_label: artifacts
            .parity_explanation
            .query_family_label()
            .to_string(),
        declaration_family_label: artifacts
            .parity_explanation
            .declaration_family_label()
            .to_string(),
        bridge_family_label: artifacts
            .parity_explanation
            .bridge_family_label()
            .to_string(),
        support_class_label: artifacts
            .support_report
            .support_subject()
            .support_class()
            .as_str()
            .to_string(),
        support_resolution_posture_label: artifacts
            .support_lookup_receipt
            .resolution_posture()
            .as_str()
            .to_string(),
        coverage_resolution_posture_label: artifacts
            .coverage_receipt
            .coverage_resolution_posture()
            .as_str()
            .to_string(),
        query_digest: artifacts
            .lifecycle_bundle
            .query_scope_projection()
            .label()
            .to_string(),
        subscription_family_digest: artifacts
            .lifecycle_bundle
            .subscription_family_projection()
            .label()
            .to_string(),
        subscription_declaration_digest: artifacts
            .lifecycle_bundle
            .subscription_declaration_projection()
            .label()
            .to_string(),
        subscription_equivalence_digest: artifacts
            .lifecycle_bundle
            .subscription_equivalence_projection()
            .label()
            .to_string(),
        bridge_declaration_digest: artifacts
            .lifecycle_bundle
            .bridge_declaration_projection()
            .label()
            .to_string(),
        bridge_basis_digest: artifacts
            .lifecycle_bundle
            .basis_posture_projection()
            .label()
            .to_string(),
        signal_strategy_digest: artifacts
            .lifecycle_bundle
            .signal_strategy_projection()
            .label()
            .to_string(),
        support_report_digest: artifacts
            .support_report
            .report_projection()
            .label()
            .to_string(),
        support_matrix_digest: artifacts
            .support_report
            .support_matrix()
            .matrix_projection()
            .label()
            .to_string(),
        support_lookup_receipt_digest: artifacts
            .support_lookup_receipt
            .lookup_receipt_projection()
            .label()
            .to_string(),
        manual_bridge_witness_digest: artifacts.witness.witness_projection().label().to_string(),
        bridge_parity_digest: artifacts
            .parity_explanation
            .explanation_projection()
            .label()
            .to_string(),
        bridge_parity_receipt_digest: artifacts
            .parity_receipt
            .receipt_projection()
            .label()
            .to_string(),
        diagnostic_trace_digest: artifacts
            .admitted_trace
            .trace_projection()
            .label()
            .to_string(),
        admitted_diagnostic_bundle_digest: artifacts
            .admitted_bundle
            .bundle_projection()
            .label()
            .to_string(),
        denied_diagnostic_bundle_digest: "none".to_string(),
        diagnostic_assembly_receipt_digest: artifacts
            .diagnostic_receipt
            .assembly_receipt_projection()
            .label()
            .to_string(),
        lifecycle_certification_digest: artifacts
            .lifecycle_bundle
            .certification_bundle_projection()
            .label()
            .to_string(),
        runtime_certification_bundle_digest: artifacts
            .runtime_bundle
            .runtime_certification_bundle_projection()
            .label()
            .to_string(),
        certification_coverage_receipt_digest: artifacts
            .coverage_receipt
            .receipt_projection()
            .label()
            .to_string(),
        continuation_digest: artifacts.continuation_digest.clone(),
        preview_isolation_digest: artifacts.preview_isolation_digest.clone(),
        failure_digest: "none".to_string(),
        counter_snapshot: artifacts
            .runtime_bundle
            .counter_snapshot_projection()
            .label()
            .to_string(),
        compile_fail_boundary_digest: compile_fail_boundary_digest(
            MILESTONE_NINE_THREE_REQUIRED_COMPILE_FAIL_TARGETS,
        ),
    }
}

pub(super) fn compile_fail_rejection(target: &'static str) -> MilestoneNineThreeRejectionBundle {
    let digest = compile_fail_boundary_digest(&[target]);
    MilestoneNineThreeRejectionBundle {
        failure_class: MilestoneNineThreeFailureClass::CompileFailBoundary,
        failure_kind: "compile_fail_boundary".to_string(),
        failure_digest: digest.clone(),
        denied_bundle_digest: "compile_fail_boundary".to_string(),
        counter_snapshot: digest.clone(),
        compile_fail_boundary_digest: digest,
    }
}

pub(super) fn denied_support_failure(
    artifacts: &CertifiedLaneArtifacts,
) -> Result<MilestoneNineThreeRejectionBundle, IdentityDerivationDenial> {
    let denied = denied_support_artifacts(artifacts)?;
    Ok(MilestoneNineThreeRejectionBundle {
        failure_class: MilestoneNineThreeFailureClass::SupportDenied,
        failure_kind: denied.failure.stage().as_str().to_string(),
        failure_digest: denied.failure.failure_projection().label().to_string(),
        denied_bundle_digest: denied.denied_bundle.bundle_projection().label().to_string(),
        counter_snapshot: denied
            .denied_bundle
            .counters()
            .counter_projection()
            .label()
            .to_string(),
        compile_fail_boundary_digest: compile_fail_boundary_digest(&[
            "subscription_support_report_durable_overclaim_forbidden.rs",
        ]),
    })
}

pub(super) fn denied_bridge_parity_failure(
    detail: &CertifiedLaneArtifacts,
    foreign: &CertifiedLaneArtifacts,
) -> Result<MilestoneNineThreeRejectionBundle, IdentityDerivationDenial> {
    let activation = prepare_subscription_activation(foreign.admission.clone())?;
    let error = explain_query_subscription_bridge_parity(
        &foreign.declaration,
        &foreign.lowering,
        &activation,
        detail.witness.clone(),
    )
    .unwrap_err();

    Ok(MilestoneNineThreeRejectionBundle {
        failure_class: MilestoneNineThreeFailureClass::BridgeParityDenied,
        failure_kind: error.failure().failure_kind().as_str().to_string(),
        failure_digest: error.failure().failure_projection().label().to_string(),
        denied_bundle_digest: "none".to_string(),
        counter_snapshot: error.counters().counter_projection().label().to_string(),
        compile_fail_boundary_digest: compile_fail_boundary_digest(&[
            "subscription_bridge_parity_mismatched_declaration_forbidden.rs",
        ]),
    })
}

pub(super) fn denied_runtime_certification_failure(
    artifacts: &CertifiedLaneArtifacts,
) -> Result<MilestoneNineThreeRejectionBundle, IdentityDerivationDenial> {
    let admitted_row = QuerySubscriptionFamilyCoverageRow::admitted(
        artifacts.declaration.family(),
        &artifacts.support_report,
        &artifacts.parity_explanation,
        &artifacts.lifecycle_bundle,
        &artifacts.admitted_bundle,
        QuerySubscriptionLifecycleCoverageClass::LifecycleCloseout,
    )
    .unwrap();
    let matrix = build_query_subscription_family_coverage_matrix(vec![admitted_row])?;
    let handle = build_certified_family_coverage_handle(
        &matrix,
        artifacts.declaration.family(),
        CoverageResolutionPosture::IndexedCoverageSet,
    )
    .unwrap();
    let scope = build_query_subscription_runtime_certification_scope(
        artifacts.support_report.clone(),
        artifacts.parity_explanation.clone(),
        artifacts.admitted_bundle.clone(),
        artifacts.lifecycle_bundle.clone(),
        handle,
    )
    .unwrap();
    let error = certify_query_subscription_runtime_family(scope).unwrap_err();

    Ok(MilestoneNineThreeRejectionBundle {
        failure_class: MilestoneNineThreeFailureClass::RuntimeCertificationDenied,
        failure_kind: error.error_kind().as_str().to_string(),
        failure_digest: error.failure_projection().label().to_string(),
        denied_bundle_digest: "none".to_string(),
        counter_snapshot: error.counters().counter_projection().label().to_string(),
        compile_fail_boundary_digest: compile_fail_boundary_digest(&[
            "subscription_diagnostic_bundle_missing_hostile_coverage_forbidden.rs",
        ]),
    })
}

pub(super) fn denied_cross_family_scope_failure(
    detail: &CertifiedLaneArtifacts,
    foreign: &CertifiedLaneArtifacts,
) -> Result<MilestoneNineThreeRejectionBundle, IdentityDerivationDenial> {
    let hostile = denied_support_artifacts(foreign)?;
    let matrix = build_query_subscription_family_coverage_matrix(vec![
        QuerySubscriptionFamilyCoverageRow::admitted(
            foreign.declaration.family(),
            &foreign.support_report,
            &foreign.parity_explanation,
            &foreign.lifecycle_bundle,
            &foreign.admitted_bundle,
            QuerySubscriptionLifecycleCoverageClass::LifecycleCloseout,
        )
        .unwrap(),
        QuerySubscriptionFamilyCoverageRow::hostile(
            foreign.declaration.family(),
            &foreign.support_report,
            &foreign.parity_explanation,
            &foreign.lifecycle_bundle,
            &hostile.denied_bundle,
            &hostile.failure,
            QuerySubscriptionLifecycleCoverageClass::ActiveLifecycle,
        )
        .unwrap(),
    ])?;
    let handle = build_certified_family_coverage_handle(
        &matrix,
        foreign.declaration.family(),
        CoverageResolutionPosture::IndexedCoverageSet,
    )
    .unwrap();
    let error = build_query_subscription_runtime_certification_scope(
        detail.support_report.clone(),
        detail.parity_explanation.clone(),
        detail.admitted_bundle.clone(),
        detail.lifecycle_bundle.clone(),
        handle,
    )
    .unwrap_err();

    Ok(MilestoneNineThreeRejectionBundle {
        failure_class: MilestoneNineThreeFailureClass::RuntimeCertificationDenied,
        failure_kind: error.error_kind().as_str().to_string(),
        failure_digest: error.failure_projection().label().to_string(),
        denied_bundle_digest: "none".to_string(),
        counter_snapshot: error.counters().counter_projection().label().to_string(),
        compile_fail_boundary_digest: compile_fail_boundary_digest(&[
            "subscription_runtime_certification_uncertified_family_forbidden.rs",
        ]),
    })
}

fn denied_support_artifacts(
    artifacts: &CertifiedLaneArtifacts,
) -> Result<SupportDeniedArtifacts, IdentityDerivationDenial> {
    Ok(denied_support_artifacts_from_parts(
        &artifacts.selection,
        &artifacts.declaration,
        &artifacts.lowering,
        &artifacts.admission,
    )?)
}

pub(super) fn denied_support_artifacts_from_parts(
    selection: &QuerySubscriptionFamilySelection,
    declaration: &QuerySubscriptionDeclarationArtifact,
    lowering: &BridgeSubscriptionLoweringPlan,
    admission: &QuerySubscriptionAdmissionArtifact,
) -> Result<SupportDeniedArtifacts, IdentityDerivationDenial> {
    let failure = QuerySubscriptionDiagnosticFailure::from_support_report_error(
        &report_query_subscription_support(
            QuerySubscriptionSupportSubject::activation(
                declaration,
                &prepare_subscription_activation(admission.clone())?,
            )?,
            QuerySubscriptionSupportEvidence::declaration(declaration),
        )
        .unwrap_err(),
    )?;
    let selection_context = QuerySubscriptionDiagnosticSelectionContext::from_selection(selection)?;
    let denied_trace = trace_denied_query_subscription_diagnostics(
        &selection_context,
        Some(declaration),
        Some(lowering),
        Some(admission),
        None,
        failure.clone(),
    )
    .unwrap();
    let denied_bundle = bundle_denied_query_subscription_diagnostics(
        denied_trace,
        &selection_context,
        Some(declaration),
        Some(lowering),
        Some(admission),
        None,
        failure.clone(),
    )
    .unwrap()
    .0;

    Ok(SupportDeniedArtifacts {
        denied_bundle,
        failure,
    })
}

fn compile_fail_boundary_digest(targets: &[&str]) -> String {
    digest_parts(
        &targets
            .iter()
            .map(|target| format!("compile_fail:{target}"))
            .collect::<Vec<_>>(),
    )
}
