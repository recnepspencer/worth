#[cfg(test)]
use worth_foundational::{
    materialize_diagnostic_support_report, FoundationalDiagnosticDeliveryClass,
    FoundationalDiagnosticSupportClaimStrength, FoundationalDiagnosticSupportInput,
    FoundationalDiagnosticSurfaceAvailability, FoundationalProfileSet,
};

#[cfg(test)]
use super::super::foundational_integration::{
    build_provenance, build_rows, materialize_profile_progression,
    WorthQueryDomainCapabilityProvenanceFreshnessPolicy,
};
#[cfg(test)]
use super::super::materialization::{
    WorthQueryDomainCapabilityDescriptiveArtifactKind,
    WorthQueryDomainCapabilityDescriptiveMaterializationDenial,
};
#[cfg(test)]
use super::super::payloads::WorthQueryDomainCapabilityPayload;
use super::super::payloads::{
    WorthQueryAdmissionContributionPayload, WorthQueryInvariantCapabilityContributionPayload,
    WorthQuerySupportContributionPayload, WorthQueryWorkflowContributionPayload,
};
#[cfg(test)]
use super::super::targets::WorthQueryDomainCapabilityTargetBinding;
#[cfg(test)]
use super::super::{
    WorthQueryMaterializationReadyAdmissionContribution,
    WorthQueryMaterializationReadyDomainCapabilityContribution,
    WorthQueryMaterializationReadyInvariantCapabilityContribution,
    WorthQueryMaterializationReadySupportContribution,
    WorthQueryMaterializationReadyWorkflowContribution,
};
use super::artifacts::WorthQueryDomainCapabilitySupportReport;

pub type WorthQueryAdmissionContributionSupportReport<T> =
    WorthQueryDomainCapabilitySupportReport<WorthQueryAdmissionContributionPayload, T>;
pub type WorthQuerySupportContributionSupportReport<T> =
    WorthQueryDomainCapabilitySupportReport<WorthQuerySupportContributionPayload, T>;
pub type WorthQueryInvariantCapabilityContributionSupportReport<T> =
    WorthQueryDomainCapabilitySupportReport<WorthQueryInvariantCapabilityContributionPayload, T>;
pub type WorthQueryWorkflowContributionSupportReport<T> =
    WorthQueryDomainCapabilitySupportReport<WorthQueryWorkflowContributionPayload, T>;

#[cfg(test)]
pub fn materialize_domain_capability_support_report<P, T>(
    contribution: WorthQueryMaterializationReadyDomainCapabilityContribution<P, T>,
    requested_profile: FoundationalProfileSet,
    delivery_class: FoundationalDiagnosticDeliveryClass,
) -> Result<
    WorthQueryDomainCapabilitySupportReport<P, T>,
    WorthQueryDomainCapabilityDescriptiveMaterializationDenial,
>
where
    P: WorthQueryDomainCapabilityPayload,
    T: WorthQueryDomainCapabilityTargetBinding,
{
    let rows = build_rows(&contribution)?;
    let profile_progression = materialize_profile_progression(
        &contribution,
        requested_profile,
        WorthQueryDomainCapabilityDescriptiveArtifactKind::SupportReport,
    )?;
    let provenance = build_provenance(
        &contribution,
        &profile_progression,
        &rows,
        WorthQueryDomainCapabilityDescriptiveArtifactKind::SupportReport,
        WorthQueryDomainCapabilityProvenanceFreshnessPolicy::SupportSurface(delivery_class),
    )?;
    let report = materialize_diagnostic_support_report(
        FoundationalDiagnosticSupportInput::new(
            rows.subject.clone(),
            rows.outcome_kind,
            rows.required_rows,
            rows.standard_rows,
            rows.forensic_rows,
            availability_for(&contribution),
            FoundationalDiagnosticSupportClaimStrength::DescriptiveOnly,
            worth_foundational::FoundationalDiagnosticPartiality::Complete,
            counters_for(&contribution),
            Vec::new(),
        ),
        *profile_progression.materialized().payload().materialized(),
        delivery_class,
    )
    .map_err(|denial| {
        WorthQueryDomainCapabilityDescriptiveMaterializationDenial::SupportReport {
            category: contribution.payload().category(),
            denial,
        }
    })?;

    Ok(WorthQueryDomainCapabilitySupportReport::new(
        contribution,
        profile_progression,
        provenance,
        report,
    ))
}

#[cfg(test)]
pub fn materialize_admission_support_report<T>(
    contribution: WorthQueryMaterializationReadyAdmissionContribution<T>,
    requested_profile: FoundationalProfileSet,
    delivery_class: FoundationalDiagnosticDeliveryClass,
) -> Result<
    WorthQueryAdmissionContributionSupportReport<T>,
    WorthQueryDomainCapabilityDescriptiveMaterializationDenial,
>
where
    T: WorthQueryDomainCapabilityTargetBinding,
{
    materialize_domain_capability_support_report(contribution, requested_profile, delivery_class)
}

#[cfg(test)]
pub fn materialize_support_traceability_support_report<T>(
    contribution: WorthQueryMaterializationReadySupportContribution<T>,
    requested_profile: FoundationalProfileSet,
    delivery_class: FoundationalDiagnosticDeliveryClass,
) -> Result<
    WorthQuerySupportContributionSupportReport<T>,
    WorthQueryDomainCapabilityDescriptiveMaterializationDenial,
>
where
    T: WorthQueryDomainCapabilityTargetBinding,
{
    materialize_domain_capability_support_report(contribution, requested_profile, delivery_class)
}

#[cfg(test)]
pub fn materialize_invariant_capability_support_report<T>(
    contribution: WorthQueryMaterializationReadyInvariantCapabilityContribution<T>,
    requested_profile: FoundationalProfileSet,
    delivery_class: FoundationalDiagnosticDeliveryClass,
) -> Result<
    WorthQueryInvariantCapabilityContributionSupportReport<T>,
    WorthQueryDomainCapabilityDescriptiveMaterializationDenial,
>
where
    T: WorthQueryDomainCapabilityTargetBinding,
{
    materialize_domain_capability_support_report(contribution, requested_profile, delivery_class)
}

#[cfg(test)]
pub fn materialize_workflow_support_report<T>(
    contribution: WorthQueryMaterializationReadyWorkflowContribution<T>,
    requested_profile: FoundationalProfileSet,
    delivery_class: FoundationalDiagnosticDeliveryClass,
) -> Result<
    WorthQueryWorkflowContributionSupportReport<T>,
    WorthQueryDomainCapabilityDescriptiveMaterializationDenial,
>
where
    T: WorthQueryDomainCapabilityTargetBinding,
{
    materialize_domain_capability_support_report(contribution, requested_profile, delivery_class)
}

#[cfg(test)]
fn availability_for<P, T>(
    contribution: &WorthQueryMaterializationReadyDomainCapabilityContribution<P, T>,
) -> FoundationalDiagnosticSurfaceAvailability
where
    P: WorthQueryDomainCapabilityPayload,
    T: WorthQueryDomainCapabilityTargetBinding,
{
    match contribution.payload().target().kind() {
        super::super::WorthQueryDomainCapabilityTargetKind::IntentDeclaration
        | super::super::WorthQueryDomainCapabilityTargetKind::AdmittedIntentPlan => {
            FoundationalDiagnosticSurfaceAvailability::retained_hot()
        }
        super::super::WorthQueryDomainCapabilityTargetKind::LowerRuntimeBoundaryEnvelope => {
            FoundationalDiagnosticSurfaceAvailability::reconstructable()
        }
    }
}

#[cfg(test)]
fn counters_for<P, T>(
    contribution: &WorthQueryMaterializationReadyDomainCapabilityContribution<P, T>,
) -> worth_foundational::FoundationalDiagnosticCounterSnapshot
where
    P: WorthQueryDomainCapabilityPayload,
    T: WorthQueryDomainCapabilityTargetBinding,
{
    match contribution.payload().target().kind() {
        super::super::WorthQueryDomainCapabilityTargetKind::LowerRuntimeBoundaryEnvelope => {
            worth_foundational::FoundationalDiagnosticCounterSnapshot::new(0, 1, 0, 0, 0, 1)
        }
        _ => worth_foundational::FoundationalDiagnosticCounterSnapshot::new(1, 0, 0, 0, 0, 0),
    }
}
