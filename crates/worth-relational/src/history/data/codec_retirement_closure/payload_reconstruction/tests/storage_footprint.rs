//! Finite fixture funding derived from the production phase owners' descriptors.
use super::*;

pub(super) struct Footprint {
    pub(super) peak: u64,
    pub(super) retained: u64,
    pub(super) next: Option<AuthoritativeRecordAspectState>,
}

pub(super) fn quote(
    current: Option<&AuthoritativeRecordAspectState>,
    source: &PublishedAuthoritativePatch,
    plan: &LoweredAspectContractPlan,
    creation: bool,
) -> Footprint {
    // Fixture geometry only: no runtime, effect, publication or recovery occurs.
    // The tested run below still receives its own original finite request.
    let request = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let (input, _) = execution
        .run_native_with_checkpoint(|context| {
            portable_input::quote(source, plan, creation, context).unwrap()
        })
        .unwrap();
    let candidate = portable_input::build(source, plan, creation).unwrap();
    let admission = allocation::admission(&candidate, plan).unwrap();
    let worth_proof::TransitionOutcome::Success(admitted) = readmit_portable_record_aspect_patch(
        candidate,
        if creation {
            PortablePatchReadmissionPurpose::RecordCreation
        } else {
            PortablePatchReadmissionPurpose::RecordMutation
        },
        &|key: &AspectKey| plan.contract_for(key),
    ) else {
        panic!("fixture patch admission")
    };
    let applied = admitted
        .application_retained_storage_bounds(current)
        .unwrap();
    let peak = input
        .retained_bytes
        .checked_add(input.transient_bytes)
        .unwrap()
        .max(
            input
                .retained_bytes
                .checked_add(admission.retained_bytes)
                .unwrap()
                .checked_add(admission.transient_bytes)
                .unwrap(),
        )
        .max(
            input
                .retained_bytes
                .checked_add(admission.retained_bytes)
                .unwrap()
                .checked_add(applied.retained_bytes)
                .unwrap()
                .checked_add(applied.transient_bytes)
                .unwrap(),
        );
    let worth_proof::TransitionOutcome::Success(next) = admitted.apply_to_optional(current) else {
        panic!("fixture payload application")
    };
    Footprint {
        peak: u64::try_from(peak).unwrap(),
        retained: u64::try_from(applied.retained_bytes).unwrap(),
        next: next.map(|state| state.into_parts().into_parts().0),
    }
}
