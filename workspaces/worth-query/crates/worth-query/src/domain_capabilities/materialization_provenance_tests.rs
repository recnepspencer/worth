use super::materialization::{
    materialize_explanation_explanation_bundle, materialize_support_traceability_support_report,
};
use super::payloads::{
    WorthQueryExplanationContributionPayload, WorthQueryExplanationContributionPosture,
    WorthQuerySupportContributionPayload, WorthQuerySupportContributionPosture,
};
use super::targets::WorthQueryLowerRuntimeBoundaryBoundContributionTarget;
use super::test_support::{lower_runtime_target, ready_payload};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use worth_foundational::{
    AdmissionReadinessProfile, CertificationPostureProfile, CompatibilityPostureProfile,
    DiagnosticRichnessProfile, ExecutionObjectiveProfile, FoundationalDiagnosticDeliveryClass,
    FoundationalProfileSet, FoundationalProfileSetInput, ObservationActivationProfile,
    RetentionDeliveryProfile, SupportPostureProfile,
};

#[test]
fn lower_runtime_descriptive_outputs_do_not_claim_current_fresh_provenance(
) -> Result<(), IdentityDerivationDenial> {
    let support = materialize_support_traceability_support_report(
        lower_runtime_ready_contribution(WorthQuerySupportContributionPayload::new(
            WorthQuerySupportContributionPosture::DeclarationTraceability,
            "runtime.boundary.traceability.replay",
            "lower runtime support remains replay-derived through replay-capable delivery",
        ))?,
        forensic_support_profile(),
        FoundationalDiagnosticDeliveryClass::ReconstructableFromReplay,
    )
    .expect("support report should materialize");
    let explanation = materialize_explanation_explanation_bundle(
        lower_runtime_ready_contribution(WorthQueryExplanationContributionPayload::new(
            WorthQueryExplanationContributionPosture::ExplainsFallback,
            "runtime.boundary.explanation.hot",
            "lower runtime explanation remains replay-derived even when callers request hot delivery",
        )?)?,
        forensic_support_profile(),
        FoundationalDiagnosticDeliveryClass::MustBeHot,
    )
    .expect("explanation bundle should materialize");

    for provenance in [support.provenance(), explanation.provenance()] {
        assert_eq!(
            provenance.locality(),
            worth_foundational::FoundationalBoundaryEvidenceLocality::ReplayDerived
        );
        assert_eq!(
            provenance.freshness_posture(),
            worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::ReconstructedFromReplay
        );
    }

    Ok(())
}

fn lower_runtime_ready_contribution<P>(
    payload: P,
) -> Result<
    super::WorthQueryMaterializationReadyDomainCapabilityContribution<
        P,
        WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
    >,
    IdentityDerivationDenial,
>
where
    P: super::WorthQueryDomainCapabilityPayload,
    (P, WorthQueryLowerRuntimeBoundaryBoundContributionTarget):
        super::proof_integration::AllowedContributionBinding<
            P,
            WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
        >,
{
    Ok(ready_payload(
        lower_runtime_target("boundary-provenance")?,
        payload,
    )?)
}

fn forensic_support_profile() -> FoundationalProfileSet {
    FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness: DiagnosticRichnessProfile::Forensic,
        support_posture: SupportPostureProfile::SupportReady,
        compatibility_posture: CompatibilityPostureProfile::NativeOnly,
        admission_readiness: AdmissionReadinessProfile::Admitted,
        retention_delivery: RetentionDeliveryProfile::Retained,
        certification_posture: CertificationPostureProfile::Uncertified,
        execution_objective: ExecutionObjectiveProfile::Balanced,
        observation_activation: ObservationActivationProfile::Continuous,
    })
    .expect("valid profile")
}
