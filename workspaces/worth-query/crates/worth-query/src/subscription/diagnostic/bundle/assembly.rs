use super::super::super::admission::QuerySubscriptionAdmissionArtifact;
use super::super::super::bridge_lowering::BridgeSubscriptionLoweringPlan;
use super::super::super::certification::SubscriptionLifecycleCertificationBundle;
use super::super::super::closeout::SubscriptionLifecycleCloseout;
use super::super::super::continuation::SubscriptionContinuationReport;
use super::super::super::declaration::QuerySubscriptionDeclarationArtifact;
use super::super::super::evidence_identities::{
    diagnostic_admitted_bundle_identity, diagnostic_denied_bundle_identity,
};
use super::super::super::preview_isolation::PreviewSubscriptionIsolationArtifact;
use super::super::super::support::QuerySubscriptionSupportReport;
use super::super::context::QuerySubscriptionDiagnosticSelectionContext;
use super::super::trace::QuerySubscriptionDiagnosticTrace;
use super::admitted::{AdmittedDiagnosticBundleParts, QuerySubscriptionAdmittedDiagnosticBundle};
use super::denied::{DeniedDiagnosticBundleParts, QuerySubscriptionDeniedDiagnosticBundle};
use super::evidence::{
    BundleAssemblyPosture, DiagnosticAssemblyReceipt, QuerySubscriptionDiagnosticBundleWidth,
    QuerySubscriptionDiagnosticCounters,
};
use super::failure::{QuerySubscriptionDiagnosticBundleError, QuerySubscriptionDiagnosticFailure};
use super::labels::{
    omitted_stages_after_failure, semantic_label_count, semantic_labels_for_denied_bundle,
    semantic_labels_for_support,
};
use super::selection::validate_denied_selection_context;
use super::source::{
    validate_admitted_sources, validate_declaration_and_admission,
    validate_declaration_and_lowering, validate_declaration_and_support,
    validate_selection_and_declaration,
};
use super::trace_admitted::{
    validate_admitted_trace_sources, validate_admitted_trace_terminal_stage,
};
use super::trace_denied::validate_denied_trace_sources;
use super::trace_source::validate_trace_terminal_stage;

pub fn bundle_admitted_query_subscription_diagnostics(
    trace: QuerySubscriptionDiagnosticTrace,
    selection: &super::super::super::selection::QuerySubscriptionFamilySelection,
    declaration: &QuerySubscriptionDeclarationArtifact,
    lowering: &BridgeSubscriptionLoweringPlan,
    admission: &QuerySubscriptionAdmissionArtifact,
    support: &QuerySubscriptionSupportReport,
    lifecycle: &SubscriptionLifecycleCertificationBundle,
    continuation: Option<&SubscriptionContinuationReport>,
    preview: Option<&PreviewSubscriptionIsolationArtifact>,
    lifecycle_closeout: Option<&SubscriptionLifecycleCloseout>,
) -> Result<
    (
        QuerySubscriptionAdmittedDiagnosticBundle,
        DiagnosticAssemblyReceipt,
    ),
    QuerySubscriptionDiagnosticBundleError,
> {
    let refusal_counters = trace.counters().clone();
    let identity_refused = |cause| {
        QuerySubscriptionDiagnosticBundleError::identity_refused(cause, refusal_counters.clone())
    };

    let selection_context = QuerySubscriptionDiagnosticSelectionContext::from_selection(selection);
    validate_selection_and_declaration(&selection_context, declaration)?;
    validate_declaration_and_lowering(declaration, lowering)?;
    validate_declaration_and_admission(declaration, admission)?;
    validate_declaration_and_support(declaration, support)?;
    validate_admitted_sources(declaration, lowering, lifecycle)?;
    validate_admitted_trace_terminal_stage(&trace)?;
    validate_admitted_trace_sources(
        &trace,
        selection,
        declaration,
        lowering,
        admission,
        support,
        lifecycle,
        continuation,
        preview,
        lifecycle_closeout,
    )?;

    let semantic_labels = semantic_labels_for_support(
        selection.family().as_str(),
        declaration,
        lowering,
        support.support_posture(),
        "runtime_lifecycle_certified",
    )
    .map_err(&identity_refused)?;
    let bundle_width =
        QuerySubscriptionDiagnosticBundleWidth::new(trace.stage_traces().len(), 0, 0);
    let receipt = DiagnosticAssemblyReceipt::new(
        BundleAssemblyPosture::ComposedFromCanonicalArtifacts,
        trace.stage_traces().len(),
        semantic_label_count(&semantic_labels),
        0,
        bundle_width.clone(),
    )
    .map_err(&identity_refused)?;
    let counters = QuerySubscriptionDiagnosticCounters::admitted_bundle_emitted(
        trace.counters().diagnostic_trace_emission_count(),
        bundle_width.stage_evidence_count() as u64,
    );
    let bundle_identity = diagnostic_admitted_bundle_identity(
        trace.trace_identity(),
        semantic_labels.labels_identity(),
        support.report_identity(),
        lifecycle.certification_bundle_identity(),
        receipt.assembly_receipt_identity(),
        &counters.evidence_identity(),
        admission.evidence_identity(),
        continuation.map(|value| value.evidence_identity()),
        preview.map(|value| value.isolation_identity()),
        lifecycle_closeout.map(|value| value.evidence_identity()),
    )
    .map_err(&identity_refused)?;

    Ok((
        QuerySubscriptionAdmittedDiagnosticBundle::from_parts(AdmittedDiagnosticBundleParts {
            trace,
            semantic_labels,
            support_report_identity: support.report_identity().clone(),
            lifecycle_certification_identity: lifecycle.certification_bundle_identity().clone(),
            continuation_identity: continuation.map(|value| value.evidence_identity().clone()),
            preview_isolation_identity: preview.map(|value| value.isolation_identity().clone()),
            lifecycle_closeout_identity: lifecycle_closeout
                .map(|value| value.evidence_identity().clone()),
            bundle_identity,
            counters,
        }),
        receipt,
    ))
}

pub fn bundle_denied_query_subscription_diagnostics(
    trace: QuerySubscriptionDiagnosticTrace,
    selection_context: &QuerySubscriptionDiagnosticSelectionContext,
    declaration: Option<&QuerySubscriptionDeclarationArtifact>,
    lowering: Option<&BridgeSubscriptionLoweringPlan>,
    admission: Option<&QuerySubscriptionAdmissionArtifact>,
    support: Option<&QuerySubscriptionSupportReport>,
    failure: QuerySubscriptionDiagnosticFailure,
) -> Result<
    (
        QuerySubscriptionDeniedDiagnosticBundle,
        DiagnosticAssemblyReceipt,
    ),
    QuerySubscriptionDiagnosticBundleError,
> {
    let refusal_counters = trace.counters().clone();
    let identity_refused = |cause| {
        QuerySubscriptionDiagnosticBundleError::identity_refused(cause, refusal_counters.clone())
    };

    validate_denied_selection_context(
        selection_context,
        failure.stage(),
        &failure,
        declaration.is_some() || lowering.is_some() || admission.is_some() || support.is_some(),
    )?;
    if let Some(declaration) = declaration {
        validate_selection_and_declaration(selection_context, declaration)?;
    }
    if let (Some(declaration), Some(lowering)) = (declaration, lowering) {
        validate_declaration_and_lowering(declaration, lowering)?;
    }
    if let (Some(declaration), Some(admission)) = (declaration, admission) {
        validate_declaration_and_admission(declaration, admission)?;
    }
    if let (Some(declaration), Some(support)) = (declaration, support) {
        validate_declaration_and_support(declaration, support)?;
    }
    validate_trace_terminal_stage(&trace, *failure.stage())?;
    validate_denied_trace_sources(
        &trace,
        selection_context,
        declaration,
        lowering,
        admission,
        support,
        &failure,
    )?;

    let semantic_labels = semantic_labels_for_denied_bundle(
        selection_context,
        declaration,
        lowering,
        support,
        failure.stage().as_str(),
    )
    .map_err(&identity_refused)?;
    let omitted_stages = omitted_stages_after_failure(*failure.stage());
    let bundle_width =
        QuerySubscriptionDiagnosticBundleWidth::new(trace.stage_traces().len(), 1, 0);
    let receipt = DiagnosticAssemblyReceipt::new(
        BundleAssemblyPosture::ComposedFromCanonicalArtifacts,
        trace.stage_traces().len(),
        semantic_label_count(&semantic_labels),
        0,
        bundle_width.clone(),
    )
    .map_err(&identity_refused)?;
    let counters = QuerySubscriptionDiagnosticCounters::denied_bundle_emitted(
        trace.counters().diagnostic_trace_emission_count(),
        (bundle_width.stage_evidence_count() + bundle_width.failure_evidence_count()) as u64,
    );
    let bundle_identity = diagnostic_denied_bundle_identity(
        trace.trace_identity(),
        semantic_labels.labels_identity(),
        failure.failure_identity(),
        receipt.assembly_receipt_identity(),
        &counters.evidence_identity(),
        support.map(|value| value.report_identity()),
    )
    .map_err(&identity_refused)?;

    Ok((
        QuerySubscriptionDeniedDiagnosticBundle::from_parts(DeniedDiagnosticBundleParts {
            trace,
            semantic_labels,
            failure,
            omitted_stages,
            support_report_identity: support.map(|value| value.report_identity().clone()),
            bundle_identity,
            counters,
        }),
        receipt,
    ))
}
