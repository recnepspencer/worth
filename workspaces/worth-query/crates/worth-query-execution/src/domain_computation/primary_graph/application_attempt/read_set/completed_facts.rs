//! Admission of original, pre-effect facts into the shared native carrier.
use crate::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::control::StorageControl;
use {crate::domain_computation::primary_graph::vocabulary::application_attempt::attempt_denial::WorthQueryApplicationAttemptDenial, super::WorthQueryApplicationObservedFact};
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
    let (mut builder, control) = begin(count, policy, request, subject, &mut check_authority)?;
    {
        // Conversion follows admission; iterator custody ends before final checks.
        let mut values = values.into_iter();
        append(
            &mut builder,
            &mut values,
            control,
            subject,
            &mut check_authority,
        )?;
    }
    finish(builder, control, subject, &mut check_authority)
}

fn begin<'scope, 'authority>(
    count: usize,
    policy: ExecutionAllocationPolicy<'scope, 'authority>,
    request: &'scope WorthQueryRequestScope,
    subject: &str,
    check_authority: &mut dyn FnMut() -> Result<(), WorthQueryApplicationAttemptDenial>,
) -> Result<
    (NativeFactPayloadBuilder, StorageControl<'scope, 'authority>),
    WorthQueryApplicationAttemptDenial,
> {
    check_authority()?;
    let control = StorageControl::new(policy, Some(request));
    let builder = NativeFactPayloadBuilder::allocate_completed_read(count, control)
        .map_err(|cause| cause.into_attempt_denial(subject))?;
    Ok((builder, control))
}

fn append(
    builder: &mut NativeFactPayloadBuilder,
    values: &mut dyn Iterator<Item = WorthQueryApplicationObservedFact>,
    control: StorageControl<'_, '_>,
    subject: &str,
    check_authority: &mut dyn FnMut() -> Result<(), WorthQueryApplicationAttemptDenial>,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    for value in values {
        check_authority()?;
        control
            .check_live()
            .map_err(|cause| cause.into_attempt_denial(subject))?;
        builder.push(value).map_err(|cause| {
            WorthQueryApplicationAttemptDenial::allocation_denied(subject, cause)
        })?;
    }
    Ok(())
}

fn finish(
    builder: NativeFactPayloadBuilder,
    control: StorageControl<'_, '_>,
    subject: &str,
    check_authority: &mut dyn FnMut() -> Result<(), WorthQueryApplicationAttemptDenial>,
) -> Result<FactPayload, WorthQueryApplicationAttemptDenial> {
    check_authority()?;
    control
        .check_live()
        .map_err(|cause| cause.into_attempt_denial(subject))?;
    builder
        .seal()
        .map_err(|cause| WorthQueryApplicationAttemptDenial::allocation_denied(subject, cause))
}

#[cfg(test)]
mod tests;
