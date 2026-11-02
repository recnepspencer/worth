use super::*;
use crate::facade::durability::RelationalNativeCheckpointCaptureDenial as Denial;
use std::num::NonZeroUsize;
use worth_execution::{
    CancellationSource, CancellationToken, ExecutionAllocationDenialClass,
    ExecutionAllocationPolicy as Policy, LeaseRequest, MemoryLimitDenial, MemoryLimitLevel,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
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
fn native_checkpoint_admits_exact_backing_retains_clones_and_restores_editable_truth() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    let committed = create_entity_outcome(execution, &runtime, "native-custody-source");
    let head = runtime.history().branch_head(&BranchId("main".into()));
    let system = runtime
        .durability_authority()
        .native_checkpoint(execution, Policy::SystemAllocation)
        .unwrap();
    let quote = u64::try_from(system.bytes().len()).unwrap();
    // The first original-controlled evidence/encoding scope enters the SDK before
    // emitting any envelope. Its current framework admission is an independent
    // literal invoice, not the eventual encoded checkpoint length.
    const FIRST_SCOPE_BYTES: u64 = 408;
    const PEAK_BYTES: u64 = 1024 * 1024;
    let authority = crate::tests::support::test_execution_authority();
    let parent = authority
        .request_lease(request(PEAK_BYTES, CancellationToken::new()))
        .unwrap();
    let zero = parent.child(request(0, CancellationToken::new())).unwrap();
    let error = runtime
        .durability_authority()
        .native_checkpoint(
            &worth_execution::ExecutionRequest::leased(&zero),
            Policy::Execution(&zero),
        )
        .unwrap_err();
    assert_eq!(
        error,
        Denial::Request(
            crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted {
                requested: FIRST_SCOPE_BYTES,
                admitted: 0,
                ancestor: 0,
            }
        )
    );
    assert_eq!(
        runtime.history().branch_head(&BranchId("main".into())),
        head
    );
    drop(zero);

    let cancellation = CancellationSource::new();
    let stopped = parent
        .child(request(PEAK_BYTES, cancellation.token()))
        .unwrap();
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
            .child(request(PEAK_BYTES, CancellationToken::new()))
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
    let member_quote = retained
        .members()
        .iter()
        .map(|member| match member {
            crate::durability::data::RelationalCheckpointMember::New { bytes, .. } => {
                match bytes.charged_payload_bytes() {
                    Some(bytes) => bytes,
                    None => 0,
                }
            }
            _ => panic!("embedded capture resolves every member"),
        })
        .sum::<u64>();
    let retained_quote = quote + member_quote;
    eprintln!("checkpoint capture first_scope={FIRST_SCOPE_BYTES} outer={quote} retained_members={member_quote} parent_charged={}", parent.charged_memory_bytes());
    assert_eq!(parent.charged_memory_bytes(), retained_quote);
    assert_eq!(
        parent
            .reserve_memory(PEAK_BYTES - retained_quote + 1)
            .unwrap_err(),
        MemoryLimitDenial::Capacity {
            requested: PEAK_BYTES - retained_quote + 1,
            admitted: PEAK_BYTES - retained_quote,
            level: MemoryLimitLevel::Policy { ancestor: 0 },
        }
    );
    let mut recovered = durable_runtime_with_test_schema();
    bind_durable_medium(&mut recovered, attached_test_medium(&runtime).attachment());
    let prior_owner = recovered.runtime_instance_id();
    let restored = recovered
        .durability_recovery()
        .restore_native_checkpoint(execution, &retained)
        .unwrap();
    assert_eq!(restored.latest_commit.as_ref(), Some(&committed.commit));
    assert_ne!(recovered.runtime_instance_id(), prior_owner);
    let later = create_entity_outcome(execution, &recovered, "native-custody-edit");
    assert_eq!(
        recovered
            .history()
            .branch_head(&BranchId("main".into()))
            .as_ref(),
        Some(&later.commit)
    );
    drop(retained);
    assert_eq!(parent.charged_memory_bytes(), 0);
    drop(
        parent
            .reserve_memory(PEAK_BYTES)
            .expect("last clone releases all original custody"),
    );
}
