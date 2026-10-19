//! Select once under the request owner; retain its reservation into the candidate.
use super::super::{
    application_contribution::{QueryMemoryReservation, QueryRequestExecution},
    HandlerExecutionDenial, HandlerResult, WorthQueryAdvancementDenial,
    WorthQueryManagedComputationResourceDenial,
};
use super::WorthQueryOperationBodyDenial;
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::{
    application_operation::ApplicationCanonicalInput,
    application_schema::{ApplicationNativeInputAdmissionDenial, ApplicationOperationBodyProgram},
};
use worth_query_installation::facade::WorthQueryInstalledApplicationValueBinding;

pub(in crate::domain_computation::primary_graph) struct PreparedBodyInputs {
    pub(in crate::domain_computation::primary_graph) values: Vec<Option<AspectValue>>,
    _reservation: QueryMemoryReservation,
}

pub(in crate::domain_computation::primary_graph) fn prepare_inputs(
    execution: &QueryRequestExecution<'_>,
    captured: Option<&ApplicationCanonicalInput>,
    body: &ApplicationOperationBodyProgram,
    bindings: &[WorthQueryInstalledApplicationValueBinding],
) -> HandlerResult<Option<PreparedBodyInputs>, WorthQueryOperationBodyDenial> {
    if bindings.is_empty() {
        return HandlerResult::Completed(None);
    }
    let Some(captured) = captured else {
        return HandlerResult::DomainDenied(WorthQueryOperationBodyDenial::InputMismatch {
            member: "uncaptured input".to_owned(),
        });
    };
    let bytes = captured.encoded_byte_length();
    // Each selected member may be retained once. Decoder allocation is admitted
    // separately by the installed scalar binding before its decoder runs.
    let bound = bytes
        .checked_mul(bindings.len())
        .and_then(|bytes| {
            bindings
                .len()
                .checked_mul(size_of::<Option<AspectValue>>())
                .and_then(|slots| slots.checked_mul(2))
                .and_then(|slots| bytes.checked_add(slots))
        })
        .and_then(|bytes| u64::try_from(bytes).ok());
    let Some(bound) = bound else {
        return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(
            WorthQueryAdvancementDenial::Resource(
                WorthQueryManagedComputationResourceDenial::CapacityOverflow,
            ),
        ));
    };
    let reservation = match execution.reserve(bound) {
        Ok(reservation) => reservation,
        Err(cause) => {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(
                WorthQueryAdvancementDenial::Resource(cause),
            ))
        }
    };
    let prepared = execution.decision_stage(|context| {
        let mut values = Vec::with_capacity(bindings.len());
        let scope_inputs = body
            .scope_fields()
            .iter()
            .filter(|step| step.input_member().is_some())
            .map(|step| {
                (
                    step.input_member().expect("input assignment"),
                    step.scalar_family(),
                    step.input_is_optional(),
                )
            });
        let created_inputs = body
            .creation()
            .into_iter()
            .flat_map(|creation| creation.initialization_sites())
            .flat_map(|site| site.fields())
            .filter(|step| step.input_member().is_some())
            .map(|step| {
                (
                    step.input_member().expect("input initializer"),
                    step.scalar_family(),
                    step.input_is_optional(),
                )
            });
        let path_inputs = body
            .path_input_expectations()
            .iter()
            .map(|guard| (guard.member.as_str(), guard.scalar_family, false));
        for ((member, family, optional), binding) in scope_inputs
            .chain(created_inputs)
            .chain(path_inputs)
            .zip(bindings)
        {
            context
                .checkpoint(u64::try_from(bytes).unwrap_or(u64::MAX))
                .map_err(|cause| {
                    super::super::application_contribution::advancement_denial(
                        worth_execution::WorkCeilingDenial::Stopped(cause),
                    )
                })?;
            let selected = match if optional {
                captured.optional_scalar_member(member, family)
            } else {
                captured.scalar_member(member, family).map(Some)
            } {
                Ok(value) => value,
                Err(_) => {
                    return Ok(Err(WorthQueryOperationBodyDenial::InputMismatch {
                        member: member.to_owned(),
                    }))
                }
            };
            let Some(selected) = selected else {
                values.push(None);
                continue;
            };
            let materialization_work = selected
                .owned_allocation_capacity_bytes()
                .checked_mul(2)
                .and_then(|bytes| u64::try_from(bytes).ok())
                .ok_or(WorthQueryAdvancementDenial::Resource(
                    WorthQueryManagedComputationResourceDenial::CapacityOverflow,
                ))?;
            context.checkpoint(materialization_work).map_err(|cause| {
                super::super::application_contribution::advancement_denial(
                    worth_execution::WorkCeilingDenial::Stopped(cause),
                )
            })?;
            let value = selected.into_value();
            let decode = match binding.native_input_resources(&value) {
                Ok(profile) => profile,
                Err(ApplicationNativeInputAdmissionDenial::InvalidValue) => {
                    return Ok(Err(WorthQueryOperationBodyDenial::InputMismatch {
                        member: member.to_owned(),
                    }));
                }
                Err(ApplicationNativeInputAdmissionDenial::CapacityOverflow) => {
                    return Err(WorthQueryAdvancementDenial::Resource(
                        WorthQueryManagedComputationResourceDenial::CapacityOverflow,
                    ));
                }
            };
            if !validate_admitted(execution, context, decode, || {
                binding.validate_encoded(&value).is_ok()
            })? {
                return Ok(Err(WorthQueryOperationBodyDenial::InputMismatch {
                    member: member.to_owned(),
                }));
            }
            values.push(Some(value));
        }
        Ok(Ok(values))
    });
    match prepared {
        Ok(Ok(Ok(values))) => HandlerResult::Completed(Some(PreparedBodyInputs {
            values,
            _reservation: reservation,
        })),
        Ok(Ok(Err(denial))) => HandlerResult::DomainDenied(denial),
        Ok(Err(cause)) | Err(cause) => {
            HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause))
        }
    }
}

pub(super) fn validate_admitted(
    execution: &QueryRequestExecution<'_>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    profile: worth_query_declaration::facade::application_schema::ApplicationNativeInputDecodeResources,
    validate: impl FnOnce() -> bool,
) -> Result<bool, WorthQueryAdvancementDenial> {
    let _decode_memory = execution
        .reserve(profile.peak_memory_bytes())
        .map_err(WorthQueryAdvancementDenial::Resource)?;
    context.checkpoint(profile.work_units()).map_err(|cause| {
        super::super::application_contribution::advancement_denial(
            worth_execution::WorkCeilingDenial::Stopped(cause),
        )
    })?;
    let valid = validate();
    context.checkpoint(0).map_err(|cause| {
        super::super::application_contribution::advancement_denial(
            worth_execution::WorkCeilingDenial::Stopped(cause),
        )
    })?;
    Ok(valid)
}

#[cfg(test)]
#[path = "input/tests.rs"]
mod tests;
