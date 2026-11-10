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
        peak: u64::try_from(peak)
            .unwrap()
            .checked_add(scope_entries())
            .unwrap(),
        retained: if next.is_some() {
            u64::try_from(applied.retained_bytes).unwrap()
        } else {
            0
        },
        next: next.map(|state| state.into_parts().into_parts().0),
    }
}

// The tested serial adapter owns two real metered scopes: in_scope's
// run_serial and its nested run_with_checkpoint. Quote that same stack and
// callback result type through the existing owner, without any payload work.
fn scope_entries() -> u64 {
    let budget = SerialMemoryBudget::new(None);
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let request = ExecutionRequest::serial(&host);
    let mut entry = None;
    let outcome = request
        .in_scope(|lease| {
            ExecutionWorkCeiling::new(request.work_ceiling()).run_with_checkpoint(
                lease,
                None,
                |_| {
                    entry = Some(budget.charged_memory_bytes());
                    // Preserve apply's exact Result type. This empty data error
                    // constructs no reconstructed payload or Original evidence.
                    Err::<ReconstructedPayload, Stop>(Stop::Payload(String::new()))
                },
            )
        })
        .unwrap()
        .unwrap();
    drop(outcome);
    assert_eq!(budget.charged_memory_bytes(), 0);
    entry.expect("both actual reconstruction scopes were entered")
}
