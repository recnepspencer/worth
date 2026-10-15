//! The genuine committed actor footprint must be paid when retained again.
use super::*;
use std::num::NonZeroUsize;
use worth_foundational::{DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy};
use worth_query_host::facade::runtime::{CancellationToken, LeaseRequest, ExecutionAllocationDenialKind,
    ExecutionAllocationDenialClass, LeaseDenial};

fn request(memory: u64) -> LeaseRequest {
    LeaseRequest { policy: ExecutionRequestPolicy::new(ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), memory, 1)),
        deadline: None, cancellation: CancellationToken::new() }
}
#[test]
fn workflow_actor_contract_retained_wire_refuses_unfunded_storage_and_preserves_original() {
    let (application, _, instance, proposal, required, _) = approval_journey("actor-storage", 75_000);
    let performed = match approve_instance(&application, instance, &required, &proposal,
        WorkflowApprovalDecision::Approve, 75_010).unwrap() {
        WorkflowProgressOutcome::Completed(performed) => performed,
        other => panic!("the genuine actor-governed approval must commit: {other:?}"),
    };
    let record = performed.approval().unwrap().entity();
    let original = performed.receipt().committed_changes().clone();
    let authority = worth_query_host::facade::primary_graph::query_test_execution_authority_for_test();
    let zero = authority.request_lease(request(0)).unwrap();
    let execution = worth_execution::ExecutionRequest::leased(&zero);
    let refusal = original.retain_workflow_actor_evidence_for_test(&execution, record).unwrap_err();
    let cause = refusal.allocation_denial().expect("the native storage refusal retains its checked quote");
    assert!(matches!(cause.classify(), ExecutionAllocationDenialClass::Request(_)));
    assert!(matches!(cause.kind(), ExecutionAllocationDenialKind::Lease(LeaseDenial::MemoryExhausted(_))));
    assert!(cause.requested_payload_bytes().is_some_and(|bytes| bytes > 0));
    assert_eq!(refusal.subject(), "workflow actor wire");
    assert!(original == *performed.receipt().committed_changes(), "storage refusal never mutates the original committed proof");
    drop(zero);
    // The existing approval request codec bounds the entire portable request
    // to 64KiB. Its inline native wire payload cannot exceed that bound.
    let retry = authority.request_lease(request(64 * 1024)).unwrap();
    original.retain_workflow_actor_evidence_for_test(&worth_execution::ExecutionRequest::leased(&retry), record)
        .expect("fresh real storage authority retains the unchanged original proof");
    assert!(original == *performed.receipt().committed_changes());
    assert_eq!(read_retention(application.runtime(), application.runtime().current_world()), SEED_RETENTION);
}
