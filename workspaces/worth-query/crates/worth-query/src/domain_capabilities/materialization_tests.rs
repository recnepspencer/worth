use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
mod admission_explanation;
mod lower_runtime_and_remaining;

use super::targets::{
    WorthQueryAdmittedPlanBoundContributionTarget, WorthQueryDeclarationBoundContributionTarget,
    WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
};
use super::test_support::{
    admitted_plan_target, declaration_target, lower_runtime_target, ready_payload,
};

fn declaration_ready_contribution<P>(
    target_digest: &str,
    payload: P,
) -> Result<
    super::WorthQueryMaterializationReadyDomainCapabilityContribution<
        P,
        WorthQueryDeclarationBoundContributionTarget,
    >,
    IdentityDerivationDenial,
>
where
    P: super::WorthQueryDomainCapabilityPayload,
    (P, WorthQueryDeclarationBoundContributionTarget):
        super::proof_integration::AllowedContributionBinding<
            P,
            WorthQueryDeclarationBoundContributionTarget,
        >,
{
    Ok(ready_payload(declaration_target(target_digest), payload)?)
}

fn admitted_plan_ready_contribution<P>(
    target_digest: &str,
    payload: P,
) -> Result<
    super::WorthQueryMaterializationReadyDomainCapabilityContribution<
        P,
        WorthQueryAdmittedPlanBoundContributionTarget,
    >,
    IdentityDerivationDenial,
>
where
    P: super::WorthQueryDomainCapabilityPayload,
    (P, WorthQueryAdmittedPlanBoundContributionTarget):
        super::proof_integration::AllowedContributionBinding<
            P,
            WorthQueryAdmittedPlanBoundContributionTarget,
        >,
{
    Ok(ready_payload(admitted_plan_target(target_digest), payload)?)
}

fn lower_runtime_ready_contribution<P>(
    target_digest: &str,
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
        lower_runtime_target(target_digest)?,
        payload,
    )?)
}

fn forensic_support_profile() -> worth_foundational::FoundationalProfileSet {
    use worth_foundational::{
        AdmissionReadinessProfile, CertificationPostureProfile, CompatibilityPostureProfile,
        DiagnosticRichnessProfile, ExecutionObjectiveProfile, FoundationalProfileSet,
        FoundationalProfileSetInput, ObservationActivationProfile, RetentionDeliveryProfile,
        SupportPostureProfile,
    };

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

fn standard_profile() -> worth_foundational::FoundationalProfileSet {
    use worth_foundational::{
        AdmissionReadinessProfile, CertificationPostureProfile, CompatibilityPostureProfile,
        DiagnosticRichnessProfile, ExecutionObjectiveProfile, FoundationalProfileSet,
        FoundationalProfileSetInput, ObservationActivationProfile, RetentionDeliveryProfile,
        SupportPostureProfile,
    };

    FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness: DiagnosticRichnessProfile::Standard,
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
