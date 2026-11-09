use super::super::materialization::{
    materialize_admission_summary, materialize_admission_support_report,
    materialize_admission_trace_artifact, materialize_explanation_explanation_bundle,
    materialize_explanation_trace_artifact,
};
use super::super::payloads::{
    WorthQueryAdmissionContributionPayload, WorthQueryAdmissionContributionPosture,
    WorthQueryExplanationContributionPayload, WorthQueryExplanationContributionPosture,
};
use super::{declaration_ready_contribution, forensic_support_profile};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_foundational::{
    DiagnosticRichnessProfile, FoundationalDiagnosticDeliveryClass,
    FoundationalDiagnosticOutcomeKind,
};

#[test]
fn admission_support_report_materializes_foundational_rows_and_fresh_provenance(
) -> Result<(), IdentityDerivationDenial> {
    let report_ready = declaration_ready_contribution(
        "intent-a",
        WorthQueryAdmissionContributionPayload::new(
            WorthQueryAdmissionContributionPosture::Advisory,
            "spatial.arbitration.requires_clarification",
            "multiple candidates remain admissible",
        )?,
    )?;

    let report = materialize_admission_support_report(
        report_ready,
        forensic_support_profile(),
        FoundationalDiagnosticDeliveryClass::MustBeHot,
    )
    .expect("support report should materialize");

    assert_eq!(
        report.report().outcome_kind(),
        FoundationalDiagnosticOutcomeKind::Advisory
    );
    assert_eq!(report.report().decision_rows().count(), 1);
    assert_eq!(report.report().support_rows().count(), 2);
    assert_eq!(report.report().provenance_ready_rows().count(), 1);
    assert_eq!(
        report.provenance().freshness_posture(),
        worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained
    );
    assert!(report.provenance().strategy_basis().is_none());
    assert_eq!(
        report
            .profile_progression()
            .materialized()
            .payload()
            .materialized()
            .diagnostic_richness(),
        DiagnosticRichnessProfile::Forensic
    );

    let summary_ready = declaration_ready_contribution(
        "intent-a",
        WorthQueryAdmissionContributionPayload::new(
            WorthQueryAdmissionContributionPosture::Advisory,
            "spatial.arbitration.requires_clarification",
            "multiple candidates remain admissible",
        )?,
    )?;
    let summary = materialize_admission_summary(summary_ready, forensic_support_profile())
        .expect("summary should materialize");
    assert_eq!(
        summary.outcome_kind(),
        FoundationalDiagnosticOutcomeKind::Advisory
    );
    assert_eq!(
        summary.provenance().freshness_posture(),
        worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::ReducedRetained
    );
    assert_eq!(
        summary
            .profile_progression()
            .materialized()
            .payload()
            .materialized()
            .diagnostic_richness(),
        DiagnosticRichnessProfile::Standard
    );
    assert_eq!(summary.required_row_count(), 2);
    assert_eq!(summary.standard_row_count(), 1);
    assert_eq!(summary.forensic_row_count(), 1);

    let trace_ready = declaration_ready_contribution(
        "intent-a",
        WorthQueryAdmissionContributionPayload::new(
            WorthQueryAdmissionContributionPosture::Advisory,
            "spatial.arbitration.requires_clarification",
            "multiple candidates remain admissible",
        )?,
    )?;
    let trace = materialize_admission_trace_artifact(trace_ready, forensic_support_profile())
        .expect("trace artifact should materialize");
    assert_eq!(
        trace.outcome_kind(),
        FoundationalDiagnosticOutcomeKind::Advisory
    );
    assert_eq!(trace.required_rows().len(), 2);
    assert_eq!(trace.standard_rows().len(), 1);
    assert_eq!(trace.forensic_rows().len(), 1);
    assert_eq!(
        trace.provenance().freshness_posture(),
        worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained
    );

    Ok(())
}

#[test]
fn explanation_bundle_narrows_support_posture_to_internal_only(
) -> Result<(), IdentityDerivationDenial> {
    let bundle_ready = declaration_ready_contribution(
        "intent-b",
        WorthQueryExplanationContributionPayload::new(
            WorthQueryExplanationContributionPosture::ExplainsFallback,
            "spatial.reorient.fallback",
            "used canonical perpendicular fallback",
        )?,
    )?;

    let bundle = materialize_explanation_explanation_bundle(
        bundle_ready,
        forensic_support_profile(),
        FoundationalDiagnosticDeliveryClass::MustBeHot,
    )
    .expect("explanation bundle should materialize");

    assert_eq!(
        bundle
            .profile_progression()
            .admitted()
            .payload()
            .admitted()
            .support_posture(),
        worth_foundational::SupportPostureProfile::InternalOnly
    );
    assert_eq!(
        bundle
            .profile_progression()
            .materialized()
            .payload()
            .materialized()
            .support_posture(),
        worth_foundational::SupportPostureProfile::InternalOnly
    );

    let trace_ready = declaration_ready_contribution(
        "intent-b",
        WorthQueryExplanationContributionPayload::new(
            WorthQueryExplanationContributionPosture::ExplainsFallback,
            "spatial.reorient.fallback",
            "used canonical perpendicular fallback",
        )?,
    )?;
    let trace = materialize_explanation_trace_artifact(trace_ready, forensic_support_profile())
        .expect("trace artifact should materialize");
    assert_eq!(
        trace
            .profile_progression()
            .materialized()
            .payload()
            .materialized()
            .support_posture(),
        worth_foundational::SupportPostureProfile::InternalOnly
    );
    assert_eq!(trace.required_rows().len(), 2);
    assert_eq!(trace.standard_rows().len(), 1);
    assert_eq!(trace.forensic_rows().len(), 1);

    Ok(())
}
