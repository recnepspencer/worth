pub(crate) use worth_query_testkit_host::facade::host::temporal_conditional_operation::contribution_installation::checkpoint_transition::admission::*;
use worth_query_testkit_host::facade::host::checkpoint_capture;
use worth_execution::ExecutionRequest;

#[test]
fn checkpoint_transition_admission_refusal_retains_acknowledged_effects_for_explicit_repair() {
    let (source, predecessor) = source();
    let mut calls = 0;
    let pending = worth_query_testkit_host::facade::host::checkpoint_capture::stopped(|phase| {
        calls = 0;
        application_installation::WorthQueryInterimCheckpointRestore::rostered_program_with_transition_in_advancement(
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
    assert_zero_memory(pending.capture_denial().expect("capture stopped"), Some(0));
    assert_eq!(
        calls, 1,
        "capture refusal follows actual authoring and acknowledgment"
    );
    let zero = worth_query_testkit_host::facade::host::checkpoint_capture::lease(0);
    let pending = pending
        .repair_to_checkpoint_with_request(worth_execution::ExecutionRequest::leased(&zero))
        .unwrap_err();
    assert_zero_memory(
        pending.capture_denial().expect("zero request stopped"),
        None,
    );
    let original = checkpoint_capture::lease(64 * 1024 * 1024);
    let capture_only = original
        .child(checkpoint_capture::LeaseRequest {
            policy: checkpoint_capture::ExecutionRequestPolicy::new(
                original.policy().posture(),
                original.policy().determinism(),
                checkpoint_capture::ExecutionBudget::new(
                    checkpoint_capture::NonZeroUsize::MIN,
                    Some(0),
                    None,
                ),
            ),
            cancellation: checkpoint_capture::CancellationToken::new(),
            deadline: None,
        })
        .unwrap();
    let pending = checkpoint_capture::with_bootstrap_advancement_from_request(
        ExecutionRequest::leased(&original),
        |phase| {
            pending.repair_to_checkpoint(
                &phase,
                application_installation::WorthQueryCheckpointCapturePolicy::Execution(
                    &capture_only,
                ),
            )
        },
    )
    .unwrap()
    .expect_err("the complete capture cannot borrow its original parent when its selected child has zero memory");
    assert_zero_memory(
        pending.capture_denial().expect("zero capture stopped"),
        Some(0),
    );
    assert_eq!(capture_only.charged_memory_bytes(), 0);
    drop(capture_only);
    // Reuse the existing 64MiB original ceiling for the whole capture. The
    // retired frame-only child quote cannot fund an authenticated origin.
    let checkpoint = checkpoint_capture::with_bootstrap_advancement_from_request(
        ExecutionRequest::leased(&original),
        |phase| {
            pending.repair_to_checkpoint(
                &phase,
                application_installation::WorthQueryCheckpointCapturePolicy::Execution(&original),
            )
        },
    )
    .unwrap()
    .expect("a fresh funded request captures the same acknowledged successor");
    let frame_quote = checkpoint.bytes().len() as u64;
    assert_eq!(checkpoint.charged_payload_bytes(), Some(frame_quote));
    assert!(checkpoint.capture_origin().is_some());
    assert!(original.charged_memory_bytes() > frame_quote);
    assert_target_record(checkpoint, 127);
    assert_eq!(original.charged_memory_bytes(), 0);
    assert_eq!(calls, 1, "repair cannot rerun authoring");
}

fn assert_zero_memory(denial: &CaptureDenial, expected_ancestor: Option<u32>) {
    let CaptureDenial::Execution(cause) = denial else {
        panic!("the complete capture retains its original admission cause: {denial:?}");
    };
    let primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::ExecutionDenied {
        kind: primary_graph::WorthQueryProviderSessionDenialKind::ExecutionResource {
            denial: worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial::MemoryLimit {
                requested, admitted: 0,
                level: worth_query_host::facade::application_contribution::WorthQueryMemoryLimitLevel::Policy,
            }, partition_identity: None, policy_ancestor,
        }
    } = cause.kind() else { panic!("typed zero-memory refusal: {cause:?}"); };
    assert_eq!(policy_ancestor, expected_ancestor);
    assert!(requested > 0);
}
