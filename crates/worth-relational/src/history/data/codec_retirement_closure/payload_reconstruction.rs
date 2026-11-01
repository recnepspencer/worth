//! Fund the actual portable, admitted and applied payload owners separately.
use super::CodecRetirementClosureStop as Stop;
use crate::publication::patch::data::PublishedAuthoritativePatch;
use crate::schema::data::LoweredAspectContractPlan;
use worth_execution::{ExecutionResourceLease, MapKernelContext};
use worth_foundational::facade::*;
mod allocation;
mod portable_input;
#[cfg(test)]
mod tests;

pub(super) struct ReconstructedPayload {
    pub(super) state: Option<AuthoritativeRecordAspectState>,
    // Last: the actual final state dies before its retained allocation bill.
    _funding: worth_execution::ExecutionMemoryReservation,
}

pub(super) fn apply(
    current: Option<&AuthoritativeRecordAspectState>,
    patch: &PublishedAuthoritativePatch,
    plan: &LoweredAspectContractPlan,
    creation: bool,
    lease: Option<&ExecutionResourceLease<'_>>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<ReconstructedPayload, Stop> {
    let input = portable_input::quote(patch, plan, creation, context)?;
    // Reservations precede their owners so every refusal drops data first.
    let _input_memory = allocation::reserve(lease, input.retained_bytes, context)?;
    let input_scratch = allocation::reserve(lease, input.transient_bytes, context)?;
    let candidate = portable_input::build(patch, plan, creation)?;
    drop(input_scratch);
    let admission = allocation::admission(&candidate, plan)?;
    let _admitted_memory = allocation::reserve(lease, admission.retained_bytes, context)?;
    let admission_scratch = allocation::reserve(lease, admission.transient_bytes, context)?;
    let admitted = match readmit_portable_record_aspect_patch(
        candidate,
        if creation {
            PortablePatchReadmissionPurpose::RecordCreation
        } else {
            PortablePatchReadmissionPurpose::RecordMutation
        },
        &|key: &AspectKey| plan.contract_for(key),
    ) {
        worth_proof::TransitionOutcome::Success(patch) => patch,
        worth_proof::TransitionOutcome::Denied(_) => {
            return Err("native output patch readmission refused".into())
        }
    };
    drop(admission_scratch);
    let application = admitted
        .application_allocation_bounds(current)
        .ok_or_else(super::memory_overflow)?;
    let funding = allocation::reserve(lease, application.retained_bytes, context)?;
    let _application_scratch = allocation::reserve(lease, application.transient_bytes, context)?;
    match admitted.apply_to_optional(current) {
        worth_proof::TransitionOutcome::Success(state) => Ok(ReconstructedPayload {
            state: state.map(|artifact| artifact.into_parts().into_parts().0),
            _funding: funding,
        }),
        worth_proof::TransitionOutcome::Denied(_) => {
            Err("native output patch application refused".into())
        }
    }
}
