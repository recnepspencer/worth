pub(crate) use worth_query_testkit_host::facade::host::temporal_conditional_operation::contribution_installation::checkpoint_transition::admission::*;
use worth_query_host::facade::runtime::{ExecutionAllocationDenialClass, WorkCeilingDenial};

#[test]
fn checkpoint_transition_admission_refusal_retains_acknowledged_effects_for_explicit_repair() {
    let (source, predecessor) = source();
    let mut calls = 0;
    let pending = worth_query_testkit_host::facade::host::checkpoint_capture::stopped(|phase| {
        calls = 0;
        application_installation::in_memory_rostered_program_from_checkpoint_with_transition_in_advancement(
            phase,
            validated_program(),
            application_installation::WorthQueryApplicationProgramRoster::new(),
            TemporalHostSchema::declaration().unwrap(),
            configuration(),
            ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
            checkpoint::checkpoint_limits(),
            source.clone(),
            predecessor.clone(),
            Resources::bounded(512, 32, 4096).unwrap().with_capture_memory_limit(0),
            phase.execution_request().allocation_policy(),
            |writer, _| {
                calls += 1;
                author_record(writer, "allocation-repaired-record", 127)
            },
        )
    });
    let Some(CaptureDenial::Allocation(denied)) = pending.capture_denial() else {
        panic!("exact final-frame allocation cause must survive: {pending:?}");
    };
    assert_eq!(
        denied.classify(),
        ExecutionAllocationDenialClass::Request(WorkCeilingDenial::Admission(
            LeaseDenial::MemoryExhausted(
                worth_query_host::facade::runtime::MemoryLimitDenial::Capacity {
                    requested: denied.requested_payload_bytes().unwrap(),
                    admitted: 0,
                    level: worth_query_host::facade::runtime::MemoryLimitLevel::Policy {
                        ancestor: 0
                    }
                }
            )
        ))
    );
    let frame_quote = denied
        .requested_payload_bytes()
        .expect("checked real final-frame payload quote");
    assert!(frame_quote > 0);
    assert_eq!(
        calls, 1,
        "capture refusal follows actual authoring and acknowledgment"
    );
    let zero = worth_query_testkit_host::facade::host::checkpoint_capture::lease(0);
    let pending = pending
        .repair_to_checkpoint_with_request(worth_execution::ExecutionRequest::leased(&zero))
        .unwrap_err();
    let Some(CaptureDenial::Execution(cause)) = pending.capture_denial() else {
        panic!("zero request must preserve the bootstrap admission refusal: {pending:?}");
    };
    let primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::ExecutionDenied {
        kind: primary_graph::WorthQueryProviderSessionDenialKind::ExecutionResource {
            denial: worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial::MemoryLimit { requested: admission_bytes, admitted: 0, .. }, ..
        }
    } = cause.kind() else { panic!("typed original request memory refusal: {cause:?}"); };
    assert!(admission_bytes > 0);
    // Native preparation spends from the original request; the zero capture
    // policy above narrowed only Query's final output backing. A grant for that
    // frame alone now reaches the earlier Native preparation refusal.
    let frame_only = worth_query_testkit_host::facade::host::checkpoint_capture::lease(
        admission_bytes + frame_quote,
    );
    let pending = pending
        .repair_to_checkpoint_with_request(worth_execution::ExecutionRequest::leased(&frame_only))
        .expect_err("the original request must fund Native preparation before final framing");
    let Some(CaptureDenial::Execution(cause)) = pending.capture_denial() else {
        panic!("exact Native preparation cause must survive: {pending:?}");
    };
    assert!(
        matches!(cause.kind(),
        primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::ExecutionDenied {
            kind: primary_graph::WorthQueryProviderSessionDenialKind::ExecutionResource {
                denial: worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial::MemoryLimit { requested, admitted, level: worth_query_host::facade::application_contribution::WorthQueryMemoryLimitLevel::Policy },
                partition_identity: None, policy_ancestor: Some(0),
            }
        } if requested > admitted && admitted > 0),
        "the same owner retains the precise memory stop: {cause:?}"
    );
    let released = frame_only.reserve_memory(frame_quote).unwrap();
    drop(released);
    drop(frame_only);
    // The same finite grant used by the original stopped fixture funds Native
    // preparation and the final Query frame without re-running authoring.
    let repair =
        worth_query_testkit_host::facade::host::checkpoint_capture::lease(64 * 1024 * 1024);
    let checkpoint = pending
        .repair_to_checkpoint_with_request(worth_execution::ExecutionRequest::leased(&repair))
        .expect("a fresh funded request captures the same acknowledged successor");
    assert_eq!(checkpoint.bytes().len() as u64, frame_quote);
    assert_eq!(checkpoint.charged_payload_bytes(), Some(frame_quote));
    assert_target_record(checkpoint, 127);
    assert_eq!(calls, 1, "repair cannot rerun authoring");
}
