//! Admission of original, pre-effect facts into the shared native carrier.
use super::super::retained_decision_facts::StorageControl;
use super::{WorthQueryApplicationAttemptDenial, WorthQueryApplicationObservedFact};
use crate::domain_computation::primary_graph::application_checkpoint::facts::{
    FactPayload, NativeFactPayloadBuilder,
};
use worth_execution::ExecutionAllocationPolicy;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

pub(in crate::domain_computation::primary_graph::application_attempt) fn admit_completed_facts(
    count: usize,
    values: impl IntoIterator<Item = WorthQueryApplicationObservedFact>,
    policy: ExecutionAllocationPolicy<'_, '_>,
    request: &WorthQueryRequestScope,
    subject: &str,
    mut check_authority: impl FnMut() -> Result<(), WorthQueryApplicationAttemptDenial>,
) -> Result<FactPayload, WorthQueryApplicationAttemptDenial> {
    check_authority()?;
    let control = StorageControl::new(policy, Some(request));
    let mut builder = NativeFactPayloadBuilder::allocate_completed_read(count, control)
        .map_err(|cause| cause.into_attempt_denial(subject))?;
    for value in values {
        check_authority()?;
        control
            .check_live()
            .map_err(|cause| cause.into_attempt_denial(subject))?;
        builder.push(value).map_err(|cause| {
            WorthQueryApplicationAttemptDenial::allocation_denied(subject, cause)
        })?;
    }
    check_authority()?;
    control
        .check_live()
        .map_err(|cause| cause.into_attempt_denial(subject))?;
    builder
        .seal()
        .map_err(|cause| WorthQueryApplicationAttemptDenial::allocation_denied(subject, cause))
}
