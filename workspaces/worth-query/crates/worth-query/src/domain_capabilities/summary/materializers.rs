use worth_foundational::FoundationalProfileSet;

use super::super::foundational_integration::{
    build_provenance, build_rows, materialize_profile_progression,
    WorthQueryDomainCapabilityProvenanceFreshnessPolicy,
};
use super::super::materialization::{
    WorthQueryDomainCapabilityDescriptiveArtifactKind,
    WorthQueryDomainCapabilityDescriptiveMaterializationDenial,
};
use super::super::payloads::WorthQueryDomainCapabilityPayload;
use super::super::payloads::{
    WorthQueryAdmissionContributionPayload, WorthQueryContinuityContributionPayload,
};
use super::super::targets::WorthQueryDomainCapabilityTargetBinding;
use super::super::WorthQueryMaterializationReadyDomainCapabilityContribution;
#[cfg(test)]
use super::super::{
    WorthQueryMaterializationReadyAdmissionContribution,
    WorthQueryMaterializationReadyContinuityContribution,
};
use super::artifacts::WorthQueryDomainCapabilityDescriptiveSummary;

pub type WorthQueryAdmissionContributionSummary<T> =
    WorthQueryDomainCapabilityDescriptiveSummary<WorthQueryAdmissionContributionPayload, T>;
pub type WorthQueryContinuityContributionSummary<T> =
    WorthQueryDomainCapabilityDescriptiveSummary<WorthQueryContinuityContributionPayload, T>;

pub fn materialize_domain_capability_summary<P, T>(
    contribution: WorthQueryMaterializationReadyDomainCapabilityContribution<P, T>,
    requested_profile: FoundationalProfileSet,
) -> Result<
    WorthQueryDomainCapabilityDescriptiveSummary<P, T>,
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
        WorthQueryDomainCapabilityDescriptiveArtifactKind::Summary,
    )?;
    let provenance = build_provenance(
        &contribution,
        &profile_progression,
        &rows,
        WorthQueryDomainCapabilityDescriptiveArtifactKind::Summary,
        WorthQueryDomainCapabilityProvenanceFreshnessPolicy::SummaryReduction,
    )?;

    Ok(WorthQueryDomainCapabilityDescriptiveSummary::new(
        contribution,
        profile_progression,
        provenance,
        rows.subject,
        rows.primary_code,
        rows.outcome_kind,
        rows.required_rows.len(),
        rows.standard_rows.len(),
        rows.forensic_rows.len(),
    ))
}

#[cfg(test)]
pub fn materialize_admission_summary<T>(
    contribution: WorthQueryMaterializationReadyAdmissionContribution<T>,
    requested_profile: FoundationalProfileSet,
) -> Result<
    WorthQueryAdmissionContributionSummary<T>,
    WorthQueryDomainCapabilityDescriptiveMaterializationDenial,
>
where
    T: WorthQueryDomainCapabilityTargetBinding,
{
    materialize_domain_capability_summary(contribution, requested_profile)
}

#[cfg(test)]
pub fn materialize_continuity_summary<T>(
    contribution: WorthQueryMaterializationReadyContinuityContribution<T>,
    requested_profile: FoundationalProfileSet,
) -> Result<
    WorthQueryContinuityContributionSummary<T>,
    WorthQueryDomainCapabilityDescriptiveMaterializationDenial,
>
where
    T: WorthQueryDomainCapabilityTargetBinding,
{
    materialize_domain_capability_summary(contribution, requested_profile)
}
