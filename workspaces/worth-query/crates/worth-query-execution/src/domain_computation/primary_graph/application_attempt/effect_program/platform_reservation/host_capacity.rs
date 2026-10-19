use super::{
    add, overflow, ApplicationCandidateCardinalityCeiling, ApplicationCandidateRequirements,
    ApplicationCandidateResourceCeiling, PlatformEffectDemand, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationRealizedEffect, WorthQueryCandidateReservation,
    WorthQueryResourceDimension, WorthQuerySemanticScaleAxis,
};
use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;

pub(in crate::domain_computation::primary_graph::application_attempt) fn check_combined_host_capacity<
    Schema,
    Operation,
    Input,
    Scope,
>(
    admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    effects: &[WorthQueryApplicationRealizedEffect],
    outputs: &super::super::output_correspondence::WorthQueryApplicationOutputCorrespondenceCandidate,
    carried: Option<ApplicationCandidateRequirements>,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    // Ordinary handlers carry their admitted demand and sidecar additions: O(1).
    let requirements = match carried {
        Some(requirements) => requirements,
        // Native unreserved programs have no handler reservation to carry.
        // Their explicit native boundary measures O(effects + nested values + roles).
        None => {
            let mut demand = PlatformEffectDemand::default();
            for effect in effects {
                demand.observe(effect)?;
            }
            demand.retained_bytes = add(
                demand.retained_bytes,
                outputs
                    .retained_representation_bytes()
                    .ok_or_else(overflow)?,
            )?;
            demand.requirements()
        }
    };
    check_host(admission, requirements)
}

/// Combine already-admitted actual reservations before a sidecar grows backing.
pub(in crate::domain_computation::primary_graph::application_attempt) fn validate_candidate_extension<
    Schema,
    Operation,
    Input,
    Scope,
>(
    admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    original: Option<ApplicationCandidateRequirements>,
    sidecar: ApplicationCandidateRequirements,
) -> Result<Option<ApplicationCandidateRequirements>, WorthQueryApplicationAttemptDenial> {
    let Some(original) = original else {
        return Ok(None);
    };
    let (left, right) = (original.cardinality(), sidecar.cardinality());
    let combined = ApplicationCandidateRequirements::fixed_shape(
        ApplicationCandidateCardinalityCeiling::fixed(
            add(left.maximum_creates(), right.maximum_creates())?,
            add(left.maximum_deletes(), right.maximum_deletes())?,
            add(left.maximum_links(), right.maximum_links())?,
            add(left.maximum_unlinks(), right.maximum_unlinks())?,
            add(left.maximum_writes(), right.maximum_writes())?,
            add(left.maximum_emits(), right.maximum_emits())?,
        ),
        ApplicationCandidateResourceCeiling::representation_bytes(add(
            original.resources().maximum_retained_representation_bytes(),
            sidecar.resources().maximum_retained_representation_bytes(),
        )?),
    );
    check_host(admission, combined)?;
    Ok(Some(combined))
}

fn check_host<Schema, Operation, Input, Scope>(
    admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    requirements: ApplicationCandidateRequirements,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let host = admission
        .graph_work()
        .candidate_host_envelope()
        .ok_or_else(overflow)?;
    WorthQueryCandidateReservation::check_host_capacity(
        requirements,
        host.optional_scale_ceiling(WorthQuerySemanticScaleAxis::CandidateItems),
        host.optional_resource_ceiling(
            WorthQueryResourceDimension::CandidateRetainedRepresentationBytes,
        ),
    )?;
    Ok(())
}
