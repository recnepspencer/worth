//! A durable capture retains every admitted capture owner until its last alias.
//! Restored from f9a53a5d17:crates/worth-relational/src/tests/durability/contracts/native_checkpoint/allocation.rs.
//! Fixture adaptations (C2.3): the source and recovered runtimes are bound to
//! the testkit's `TestDurableMedium` (the original used the retired in-crate
//! medium), so the capture binds its medium publication owner as before. The
//! private `history().branch_head` reads become the public recorded current
//! component and the main branch's selected commit.
use super::*;
use std::num::NonZeroUsize;
use worth_execution::{
    CancellationSource, CancellationToken, ExecutionAllocationDenialClass,
    ExecutionAllocationPolicy as Policy, LeaseRequest, MemoryLimitDenial, MemoryLimitLevel,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{
    durability::RelationalNativeCheckpointCaptureDenial as Denial,
    transactions::RelationalExecutionDenialCause,
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

fn head_commit(runtime: &worth_relational::facade::runtime::RelationalRuntime) -> u64 {
    let (descriptor, _) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let worth_foundational::FoundationalBranchTarget::Basis(target) =
        descriptor.reference().target()
    else {
        panic!("main selects a real commit");
    };
    target.selected_commit_id()
}

#[test]
fn native_checkpoint_retains_all_admitted_capture_owners_and_restores_editable_truth() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let committed = create_entity_outcome(execution, &runtime, "native-custody-source");
    let head = current(execution, &runtime);
    let system = runtime
        .durability_authority()
        .native_checkpoint(execution, Policy::SystemAllocation)
        .unwrap();
    let quote = u64::try_from(system.bytes().len()).unwrap();
    const FUNDED: u64 = 8 << 20;
    let authority = crate::leased_preparation::ordinary_context::authority();
    let parent = authority
        .request_lease(request(FUNDED, CancellationToken::new()))
        .unwrap();
    let zero = parent.child(request(0, CancellationToken::new())).unwrap();
    let error = runtime
        .durability_authority()
        .native_checkpoint(
            &worth_execution::ExecutionRequest::leased(&zero),
            Policy::Execution(&zero),
        )
        .unwrap_err();
    assert!(
        matches!(error, Denial::Request(RelationalExecutionDenialCause::PolicyMemoryExhausted { requested, admitted: 0, ancestor: 0 }) if requested > 0),
        "{error:?}"
    );
    assert_eq!(parent.charged_memory_bytes(), 0);
    assert_eq!(current(execution, &runtime), head);
    drop(zero);

    let cancellation = CancellationSource::new();
    let stopped = parent.child(request(FUNDED, cancellation.token())).unwrap();
    cancellation.cancel();
    let Denial::Allocation(error) = runtime
        .durability_authority()
        .native_checkpoint(
            &worth_execution::ExecutionRequest::leased(&stopped),
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
            .native_checkpoint(
                &worth_execution::ExecutionRequest::leased(&child),
                Policy::Execution(&child),
            )
            .unwrap();
        assert_eq!(captured.bytes(), system.bytes());
        assert_eq!(captured.captured_sections(), system.captured_sections());
        let clone = captured.clone();
        assert_eq!(clone.bytes().as_ptr(), captured.bytes().as_ptr());
        drop(captured);
        clone
    };
    let retained_bytes = parent.charged_memory_bytes();
    assert!(
        retained_bytes >= quote,
        "the checkpoint retains its frame and all original capture owners"
    );
    let remaining = parent.reserve_memory(FUNDED - retained_bytes).unwrap();
    assert_eq!(
        parent.reserve_memory(1).unwrap_err(),
        MemoryLimitDenial::Capacity {
            requested: 1,
            admitted: 0,
            level: MemoryLimitLevel::Policy { ancestor: 0 },
        }
    );
    let mut recovered = durable_runtime(&medium);
    let prior_owner = recovered.runtime_instance_id();
    let restored = recovered
        .durability_recovery()
        .restore_native_checkpoint(execution, &retained)
        .unwrap();
    assert_eq!(restored.latest_commit.as_ref(), Some(&committed.commit));
    assert_ne!(recovered.runtime_instance_id(), prior_owner);
    let later = create_entity_outcome(execution, &recovered, "native-custody-edit");
    assert_eq!(head_commit(&recovered), later.commit.commit_id.0);
    release_test_commit_snapshot(&recovered, &later);
    drop(later);
    drop(recovered);
    drop(retained);
    assert_eq!(parent.charged_memory_bytes(), FUNDED - retained_bytes);
    drop(
        parent
            .reserve_memory(retained_bytes)
            .expect("last actual captured/readmitted owner releases every original grant"),
    );
    drop(remaining);
    assert_eq!(parent.charged_memory_bytes(), 0);
}
