use super::*;
use crate::facade::durability::RelationalNativeCheckpointCaptureDenial as Denial;
use std::num::NonZeroUsize;
use worth_execution::{
    CancellationToken, ExecutionAllocationDenialKind, ExecutionAllocationPolicy as Policy,
    LeaseDenial, LeaseRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

fn request(bytes: u64, cancellation: CancellationToken) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), bytes, 1),
        ),
        deadline: None,
        cancellation,
    }
}

#[test]
fn native_checkpoint_admits_exact_backing_retains_clones_and_restores_editable_truth() {
    let runtime = persisted_runtime_with_test_schema();
    let committed = create_entity_outcome(&runtime, "native-custody-source");
    let head = runtime.history().branch_head(&BranchId("main".into()));
    let system = runtime
        .durability_authority()
        .native_checkpoint(Policy::SystemAllocation)
        .unwrap();
    let quote = u64::try_from(system.bytes().len()).unwrap();
    let authority = crate::tests::support::test_execution_authority();
    let parent = authority
        .request_lease(request(quote, CancellationToken::new()))
        .unwrap();
    let zero = parent.child(request(0, CancellationToken::new())).unwrap();
    let Denial::Allocation(error) = runtime
        .durability_authority()
        .native_checkpoint(Policy::Execution(&zero))
        .unwrap_err()
    else {
        panic!("the encoded backing admission is the reached owner");
    };
    assert_eq!(
        error.kind(),
        ExecutionAllocationDenialKind::Lease(LeaseDenial::ResourceExhausted)
    );
    assert_eq!(error.requested_payload_bytes(), Some(quote));
    assert_eq!(
        runtime.history().branch_head(&BranchId("main".into())),
        head
    );
    drop(zero);

    let cancellation = CancellationToken::new();
    let stopped = parent.child(request(quote, cancellation.clone())).unwrap();
    cancellation.cancel();
    let Denial::Allocation(error) = runtime
        .durability_authority()
        .native_checkpoint(Policy::Execution(&stopped))
        .unwrap_err()
    else {
        panic!("caller cancellation must remain typed");
    };
    assert_eq!(error.kind(), ExecutionAllocationDenialKind::Cancelled);
    assert_eq!(error.requested_payload_bytes(), None);
    drop(stopped);

    let retained = {
        let child = parent
            .child(request(quote, CancellationToken::new()))
            .unwrap();
        let captured = runtime
            .durability_authority()
            .native_checkpoint(Policy::Execution(&child))
            .unwrap();
        assert_eq!(captured.bytes(), system.bytes());
        assert_eq!(captured.captured_sections(), system.captured_sections());
        let clone = captured.clone();
        assert_eq!(clone.bytes().as_ptr(), captured.bytes().as_ptr());
        drop(captured);
        clone
    };
    assert_eq!(
        parent.reserve_memory(1).unwrap_err(),
        LeaseDenial::ResourceExhausted
    );
    let mut recovered = persisted_runtime_with_test_schema();
    let prior_owner = recovered.runtime_instance_id();
    let restored = recovered
        .durability_recovery()
        .restore_native_checkpoint(&retained)
        .unwrap();
    assert_eq!(restored.latest_commit.as_ref(), Some(&committed.commit));
    assert_ne!(recovered.runtime_instance_id(), prior_owner);
    let later = create_entity_outcome(&recovered, "native-custody-edit");
    assert_eq!(
        recovered
            .history()
            .branch_head(&BranchId("main".into()))
            .as_ref(),
        Some(&later.commit)
    );
    drop(retained);
    drop(
        parent
            .reserve_memory(quote)
            .expect("last native clone releases the complete backing charge"),
    );
}
