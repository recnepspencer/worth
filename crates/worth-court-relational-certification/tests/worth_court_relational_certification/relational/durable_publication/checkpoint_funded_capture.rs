//! The original request controls capture; its surviving output restores editable truth.
use super::branch_history_recovery::current_head;
use std::num::NonZeroUsize;
use worth_execution::{
    CancellationSource, CancellationToken, ExecutionAllocationDenialClass,
    ExecutionAllocationPolicy as Policy, ExecutionAuthority, ExecutionAuthorityConfig,
    ExecutionRequest, LeaseRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{
    durability::RelationalNativeCheckpointCaptureDenial as Denial, history::BranchId,
    indexes::RelationalExecutionDenialCause,
};
use worth_testkit_relational::{
    aspect_storage::publications::release_test_commit_snapshot,
    durable_medium::TestDurableMedium,
    durable_publication::{create_entity_outcome, durable_runtime},
};
fn request(bytes: u64, cancellation: CancellationToken) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), Some(bytes), Some(1_000_000)),
        ),
        deadline: None,
        cancellation,
    }
}
#[test]
fn original_capture_request_refusals_and_escaped_output_preserve_editable_truth() {
    let host=worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let committed = create_entity_outcome(execution, &runtime, "native-custody-source");
    let head = current_head(&runtime, &BranchId("main".into()));
    let system = runtime
        .durability_authority()
        .native_checkpoint(execution, Policy::SystemAllocation)
        .unwrap();
    const FUNDED: u64 = 8 << 20;
    let authority = ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
        max_workers: NonZeroUsize::new(8).unwrap(),
        charged_memory_bytes: Some(512 * 1024 * 1024),
    })
    .unwrap();
    let parent = authority
        .request_lease(request(FUNDED, CancellationToken::new()))
        .unwrap();
    let zero = parent.child(request(0, CancellationToken::new())).unwrap();
    let error = runtime
        .durability_authority()
        .native_checkpoint(&ExecutionRequest::leased(&zero), Policy::Execution(&zero))
        .unwrap_err();
    assert!(
        matches!(error,Denial::Request(RelationalExecutionDenialCause::PolicyMemoryExhausted {requested,admitted:0,ancestor:0}) if requested>0),
        "{error:?}"
    );
    assert_eq!(current_head(&runtime, &BranchId("main".into())), head);
    drop(zero);
    let cancellation = CancellationSource::new();
    let stopped = parent.child(request(FUNDED, cancellation.token())).unwrap();
    cancellation.cancel();
    let Denial::Allocation(error) = runtime
        .durability_authority()
        .native_checkpoint(
            &ExecutionRequest::leased(&stopped),
            Policy::Execution(&stopped),
        )
        .unwrap_err()
    else {
        panic!("caller cancellation must remain typed");
    };
    assert_eq!(
        error.classify(),
        ExecutionAllocationDenialClass::Request(worth_execution::WorkCeilingDenial::Stopped(
            worth_execution::MapKernelStop::Cancelled
        ))
    );
    assert_eq!(error.requested_payload_bytes(), None);
    drop(stopped);
    let retained = {
        let child = parent
            .child(request(FUNDED, CancellationToken::new()))
            .unwrap();
        let captured = runtime
            .durability_authority()
            .native_checkpoint(&ExecutionRequest::leased(&child), Policy::Execution(&child))
            .unwrap();
        assert_eq!(captured.bytes(), system.bytes());
        let clone = captured.clone();
        drop(captured);
        clone
    };
    let mut recovered = durable_runtime(&medium);
    let prior_owner = recovered.runtime_instance_id();
    let restored = recovered
        .durability_recovery()
        .restore_native_checkpoint(execution, &retained)
        .unwrap();
    assert_eq!(restored.latest_commit.as_ref(), Some(&committed.commit));
    assert_ne!(recovered.runtime_instance_id(), prior_owner);
    let later = create_entity_outcome(execution, &recovered, "native-custody-edit");
    assert_eq!(
        current_head(&recovered, &BranchId("main".into())).as_ref(),
        Some(&later.commit)
    );
    release_test_commit_snapshot(&recovered, &later);
}
