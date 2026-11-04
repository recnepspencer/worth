pub(crate) use worth_query_testkit_host::facade::host::temporal_conditional_operation::contribution_installation::checkpoint_transition::admission::*;
use worth_query_host::facade::runtime::{ExecutionAllocationDenialClass, WorkCeilingDenial};
use worth_query_testkit_host::facade::host::checkpoint_capture;
use worth_execution::{CancellationToken, ExecutionRequest, LeaseRequest};
use worth_foundational::{ExecutionBudget, ExecutionRequestPolicy};
use std::num::NonZeroUsize;

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
        panic!("exact native allocation cause must survive: {pending:?}");
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
    let initial_capture_quote = denied
        .requested_payload_bytes()
        .expect("checked real native payload quote");
    assert!(initial_capture_quote > 0);
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
    let original = checkpoint_capture::lease(64 * 1024 * 1024);
    let child = |bytes| {
        original
            .child(LeaseRequest {
                policy: ExecutionRequestPolicy::new(
                    original.policy().posture(),
                    original.policy().determinism(),
                    ExecutionBudget::new(NonZeroUsize::MIN, Some(bytes), None),
                ),
                cancellation: CancellationToken::new(),
                deadline: None,
            })
            .unwrap()
    };
    let frame_only = child(0);
    let pending = checkpoint_capture::with_bootstrap_advancement_from_request(
        ExecutionRequest::leased(&original),
        |phase| {
            pending.repair_to_checkpoint(
                &phase,
                application_installation::WorthQueryCheckpointCapturePolicy::Execution(
                    &frame_only,
                ),
            )
        },
    )
    .unwrap()
    .expect_err("the final frame cannot allocate from its zero child while Native uses the original request");
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
    assert_eq!(frame_only.charged_memory_bytes(), 0);
    drop(frame_only);
    let repair = child(frame_quote);
    let checkpoint = checkpoint_capture::with_bootstrap_advancement_from_request(
        ExecutionRequest::leased(&original),
        |phase| {
            pending.repair_to_checkpoint(
                &phase,
                application_installation::WorthQueryCheckpointCapturePolicy::Execution(&repair),
            )
        },
    )
    .unwrap()
    .expect("a fresh funded request captures the same acknowledged successor");
    assert_eq!(checkpoint.bytes().len() as u64, frame_quote);
    assert_eq!(checkpoint.charged_payload_bytes(), Some(frame_quote));
    assert_eq!(repair.charged_memory_bytes(), frame_quote);
    assert_target_record(checkpoint, 127);
    assert_eq!(repair.charged_memory_bytes(), 0);
    assert_eq!(calls, 1, "repair cannot rerun authoring");
}
