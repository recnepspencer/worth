#[cfg(test)]
use worth_foundational::FoundationalProfileSet;

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
    WorthQueryAdmissionContributionPayload, WorthQueryExplanationContributionPayload,
    WorthQuerySupportContributionPayload,
};
#[cfg(test)]
use super::super::targets::WorthQueryDomainCapabilityTargetBinding;
#[cfg(test)]
use super::super::{
    WorthQueryMaterializationReadyAdmissionContribution,
    WorthQueryMaterializationReadyDomainCapabilityContribution,
    WorthQueryMaterializationReadyExplanationContribution,
    WorthQueryMaterializationReadySupportContribution,
};
use super::artifacts::WorthQueryDomainCapabilityTraceArtifact;

pub type WorthQueryAdmissionContributionTraceArtifact<T> =
    WorthQueryDomainCapabilityTraceArtifact<WorthQueryAdmissionContributionPayload, T>;
pub type WorthQuerySupportContributionTraceArtifact<T> =
    WorthQueryDomainCapabilityTraceArtifact<WorthQuerySupportContributionPayload, T>;
pub type WorthQueryExplanationContributionTraceArtifact<T> =
    WorthQueryDomainCapabilityTraceArtifact<WorthQueryExplanationContributionPayload, T>;

#[cfg(test)]
pub fn materialize_domain_capability_trace_artifact<P, T>(
    contribution: WorthQueryMaterializationReadyDomainCapabilityContribution<P, T>,
    requested_profile: FoundationalProfileSet,
) -> Result<
    WorthQueryDomainCapabilityTraceArtifact<P, T>,
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
        WorthQueryDomainCapabilityDescriptiveArtifactKind::TraceArtifact,
    )?;
    let provenance = build_provenance(
        &contribution,
        &profile_progression,
        &rows,
        WorthQueryDomainCapabilityDescriptiveArtifactKind::TraceArtifact,
        WorthQueryDomainCapabilityProvenanceFreshnessPolicy::TraceRetention,
    )?;

    Ok(WorthQueryDomainCapabilityTraceArtifact::new(
        contribution,
        profile_progression,
        provenance,
        rows.subject,
        rows.primary_code,
        rows.outcome_kind,
        rows.required_rows,
        rows.standard_rows,
        rows.forensic_rows,
    ))
}

#[cfg(test)]
pub fn materialize_admission_trace_artifact<T>(
    contribution: WorthQueryMaterializationReadyAdmissionContribution<T>,
    requested_profile: FoundationalProfileSet,
) -> Result<
    WorthQueryAdmissionContributionTraceArtifact<T>,
    WorthQueryDomainCapabilityDescriptiveMaterializationDenial,
>
where
    T: WorthQueryDomainCapabilityTargetBinding,
{
    materialize_domain_capability_trace_artifact(contribution, requested_profile)
}

#[cfg(test)]
pub fn materialize_support_traceability_trace_artifact<T>(
    contribution: WorthQueryMaterializationReadySupportContribution<T>,
    requested_profile: FoundationalProfileSet,
) -> Result<
    WorthQuerySupportContributionTraceArtifact<T>,
    WorthQueryDomainCapabilityDescriptiveMaterializationDenial,
>
where
    T: WorthQueryDomainCapabilityTargetBinding,
{
    materialize_domain_capability_trace_artifact(contribution, requested_profile)
}

#[cfg(test)]
pub fn materialize_explanation_trace_artifact<T>(
    contribution: WorthQueryMaterializationReadyExplanationContribution<T>,
    requested_profile: FoundationalProfileSet,
) -> Result<
    WorthQueryExplanationContributionTraceArtifact<T>,
    WorthQueryDomainCapabilityDescriptiveMaterializationDenial,
>
where
    T: WorthQueryDomainCapabilityTargetBinding,
{
    materialize_domain_capability_trace_artifact(contribution, requested_profile)
}
