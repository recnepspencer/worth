use super::super::bundle::QuerySubscriptionDiagnosticBundleError;
use super::super::stage::QuerySubscriptionDiagnosticEvidence;

use super::super::super::admission::QuerySubscriptionAdmissionArtifact;
use super::super::super::bridge_lowering::BridgeSubscriptionLoweringPlan;
use super::super::super::certification::SubscriptionLifecycleCertificationBundle;
use super::super::super::closeout::SubscriptionLifecycleCloseout;
use super::super::super::continuation::SubscriptionContinuationReport;
use super::super::super::declaration::QuerySubscriptionDeclarationArtifact;
use super::super::super::preview_isolation::PreviewSubscriptionIsolationArtifact;
use super::super::super::support::QuerySubscriptionSupportReport;
use super::super::stage::QuerySubscriptionDiagnosticStage;
use super::identity::trace_from_stage_evidence;
use super::validation::validate_admitted_sources;
use super::vocabulary::QuerySubscriptionDiagnosticTrace;

pub fn trace_admitted_query_subscription_diagnostics(
    selection: &super::super::super::selection::QuerySubscriptionFamilySelection,
    declaration: &QuerySubscriptionDeclarationArtifact,
    lowering: &BridgeSubscriptionLoweringPlan,
    admission: &QuerySubscriptionAdmissionArtifact,
    support: &QuerySubscriptionSupportReport,
    lifecycle: &SubscriptionLifecycleCertificationBundle,
    continuation: Option<&SubscriptionContinuationReport>,
    preview: Option<&PreviewSubscriptionIsolationArtifact>,
    closeout: Option<&SubscriptionLifecycleCloseout>,
) -> Result<QuerySubscriptionDiagnosticTrace, QuerySubscriptionDiagnosticBundleError> {
    validate_admitted_sources(
        selection,
        declaration,
        lowering,
        admission,
        support,
        lifecycle,
    )?;

    let mut stage_evidence = vec![
        QuerySubscriptionDiagnosticEvidence::admitted(
            QuerySubscriptionDiagnosticStage::FamilySelection,
            format!(
                "query subscription family {} selected through canonical admission classification",
                selection.family().as_str()
            ),
            selection.equivalence_basis().evidence_identity(),
            &selection.counters().evidence_identity(),
        ),
        QuerySubscriptionDiagnosticEvidence::admitted(
            QuerySubscriptionDiagnosticStage::Declaration,
            format!(
                "query subscription declaration {} preserved canonical family semantics",
                declaration.family().as_str()
            ),
            declaration.declaration_identity(),
            &declaration.counters().evidence_identity(),
        ),
        QuerySubscriptionDiagnosticEvidence::admitted(
            QuerySubscriptionDiagnosticStage::BridgeFamilyLowering,
            format!(
                "bridge lowering admitted family {} for canonical query declaration",
                lowering.bridge_family().as_str()
            ),
            lowering.bridge_declaration_identity(),
            &lowering.counters().evidence_identity(),
        ),
        QuerySubscriptionDiagnosticEvidence::admitted(
            QuerySubscriptionDiagnosticStage::RuntimeBackedAdmission,
            "runtime-backed subscription admission preserved declaration, bridge, basis, and signal identity",
            admission.evidence_identity(),
            &admission.counters().evidence_identity(),
        ),
        QuerySubscriptionDiagnosticEvidence::admitted(
            QuerySubscriptionDiagnosticStage::SupportReporting,
            format!(
                "support reporting certified {} for subject class {}",
                support.support_posture().as_str(),
                support.support_subject().support_class().as_str()
            ),
            support.report_identity(),
            &support.counters().evidence_identity(),
        ),
        QuerySubscriptionDiagnosticEvidence::admitted(
            QuerySubscriptionDiagnosticStage::Certification,
            "subscription lifecycle certification closed the admitted runtime-backed proof chain",
            lifecycle.certification_bundle_identity(),
            lifecycle.counter_sequence_identity(),
        ),
    ];

    if let Some(continuation) = continuation {
        stage_evidence.push(QuerySubscriptionDiagnosticEvidence::admitted(
            QuerySubscriptionDiagnosticStage::Continuation,
            "subscription continuation report preserved canonical lane identity",
            continuation.evidence_identity(),
            continuation
                .performance_receipt()
                .performance_receipt_identity(),
        ));
    }

    if let Some(preview) = preview {
        stage_evidence.push(QuerySubscriptionDiagnosticEvidence::admitted(
            QuerySubscriptionDiagnosticStage::PreviewIsolation,
            "preview isolation artifact remained distinct from authoritative lifecycle state",
            preview.isolation_identity(),
            &preview.counters().evidence_identity(),
        ));
    }

    if let Some(closeout) = closeout {
        stage_evidence.push(QuerySubscriptionDiagnosticEvidence::admitted(
            QuerySubscriptionDiagnosticStage::LifecycleCloseout,
            format!(
                "lifecycle closeout {} preserved terminal runtime evidence",
                closeout.closeout_kind().as_str()
            ),
            closeout.evidence_identity(),
            &closeout.counters().evidence_identity(),
        ));
    }

    let stage_evidence = stage_evidence
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|cause| {
            QuerySubscriptionDiagnosticBundleError::identity_refused(cause, Default::default())
        })?;
    trace_from_stage_evidence(stage_evidence).map_err(|cause| {
        QuerySubscriptionDiagnosticBundleError::identity_refused(cause, Default::default())
    })
}
