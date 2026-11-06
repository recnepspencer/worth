use super::super::materialization::{
    materialize_continuity_explanation_bundle, materialize_continuity_summary,
    materialize_invariant_capability_support_report,
    materialize_support_traceability_support_report,
    materialize_support_traceability_trace_artifact, materialize_workflow_support_report,
};
use super::super::payloads::{
    WorthQueryContinuityContributionPayload, WorthQueryContinuityContributionPosture,
    WorthQueryInvariantCapabilityContributionPayload,
    WorthQueryInvariantCapabilityContributionPosture, WorthQuerySupportContributionPayload,
    WorthQuerySupportContributionPosture, WorthQueryWorkflowContributionPayload,
    WorthQueryWorkflowContributionPosture,
};
use super::{
    admitted_plan_ready_contribution, forensic_support_profile, lower_runtime_ready_contribution,
    standard_profile,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use worth_foundational::{FoundationalDiagnosticDeliveryClass, FoundationalDiagnosticOutcomeKind};

#[test]
fn lower_runtime_support_report_preserves_requested_richness_and_current_provenance(
) -> Result<(), IdentityDerivationDenial> {
    let ready = lower_runtime_ready_contribution(
        "boundary-a",
        WorthQuerySupportContributionPayload::new(
            WorthQuerySupportContributionPosture::DeclarationTraceability,
            "runtime.boundary.traceability",
            "lower runtime support derives from reconstructed boundary evidence",
        ),
    )?;

    let report = materialize_support_traceability_support_report(
        ready,
        forensic_support_profile(),
        FoundationalDiagnosticDeliveryClass::ReconstructableFromReplay,
    )
    .expect("support report should materialize");

    assert_eq!(
        report
            .profile_progression()
            .materialized()
            .payload()
            .materialized()
            .diagnostic_richness(),
        worth_foundational::DiagnosticRichnessProfile::Forensic
    );
    assert_eq!(
        report.provenance().locality(),
        worth_foundational::FoundationalBoundaryEvidenceLocality::ReplayDerived
    );
    assert_eq!(
        report.provenance().freshness_posture(),
        worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::ReconstructedFromReplay
    );
    assert!(report.provenance().strategy_basis().is_some());
    assert!(report
        .report()
        .support_rows()
        .all(|row| row.outcome_kind() == FoundationalDiagnosticOutcomeKind::Accepted));
    assert!(report.report().support_rows().all(|row| matches!(
        row.evidence_posture(),
        worth_foundational::FoundationalDiagnosticSupportEvidencePosture::Present(
            worth_foundational::FoundationalDiagnosticEvidencePosture::Summarized
        )
    )));

    let trace = materialize_support_traceability_trace_artifact(
        lower_runtime_ready_contribution(
            "boundary-a",
            WorthQuerySupportContributionPayload::new(
                WorthQuerySupportContributionPosture::DeclarationTraceability,
                "runtime.boundary.traceability",
                "lower runtime support derives from reconstructed boundary evidence",
            ),
        )?,
        forensic_support_profile(),
    )
    .expect("trace artifact should materialize");
    assert_eq!(
        trace.provenance().freshness_posture(),
        worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::ReconstructedFromReplay
    );

    Ok(())
}

#[test]
fn remaining_categories_materialize_real_descriptive_outputs(
) -> Result<(), IdentityDerivationDenial> {
    let invariant_ready = super::declaration_ready_contribution(
        "intent-c",
        WorthQueryInvariantCapabilityContributionPayload::new(
            WorthQueryInvariantCapabilityContributionPosture::CapabilityGap,
            "graph.face_inner_loop_insertion",
            "topology substrate is not available",
        )?,
    )?;
    let workflow_ready = admitted_plan_ready_contribution(
        "plan-a",
        WorthQueryWorkflowContributionPayload::new(
            WorthQueryWorkflowContributionPosture::ConfirmationRequired,
            "preview.confirmation.required",
            "destructive promotion requires confirmation",
        )?,
    )?;
    let continuity_bundle_ready = admitted_plan_ready_contribution(
        "plan-b",
        WorthQueryContinuityContributionPayload::new(
            WorthQueryContinuityContributionPosture::Split,
            "continuity.identity.split",
            "edge split produces two descendant identities",
        )?,
    )?;

    let invariant_report = materialize_invariant_capability_support_report(
        invariant_ready,
        standard_profile(),
        FoundationalDiagnosticDeliveryClass::MustBeHot,
    )
    .expect("invariant report");
    let workflow_report = materialize_workflow_support_report(
        workflow_ready,
        standard_profile(),
        FoundationalDiagnosticDeliveryClass::MustBeHot,
    )
    .expect("workflow report");
    let continuity_bundle = materialize_continuity_explanation_bundle(
        continuity_bundle_ready,
        standard_profile(),
        FoundationalDiagnosticDeliveryClass::MustBeHot,
    )
    .expect("continuity bundle");
    let continuity_summary_ready = admitted_plan_ready_contribution(
        "plan-b",
        WorthQueryContinuityContributionPayload::new(
            WorthQueryContinuityContributionPosture::Split,
            "continuity.identity.split",
            "edge split produces two descendant identities",
        )?,
    )?;
    let continuity_summary =
        materialize_continuity_summary(continuity_summary_ready, standard_profile())
            .expect("continuity summary");

    assert_eq!(
        invariant_report.report().outcome_kind(),
        FoundationalDiagnosticOutcomeKind::Unsupported
    );
    assert_eq!(
        workflow_report.report().outcome_kind(),
        FoundationalDiagnosticOutcomeKind::Advisory
    );
    assert!(workflow_report.provenance().strategy_basis().is_some());
    assert_eq!(
        continuity_bundle.bundle().outcome_kind(),
        FoundationalDiagnosticOutcomeKind::Advisory
    );
    assert!(continuity_bundle.provenance().strategy_basis().is_some());
    assert_eq!(
        continuity_summary.outcome_kind(),
        FoundationalDiagnosticOutcomeKind::Advisory
    );

    Ok(())
}
