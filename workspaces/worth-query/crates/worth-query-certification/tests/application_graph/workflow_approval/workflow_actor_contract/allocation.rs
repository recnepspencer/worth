//! The genuine committed actor footprint must be paid when retained again.
use super::*;
use std::num::NonZeroUsize;
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_host::facade::runtime::{CancellationToken, LeaseRequest};

fn request(memory: u64) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), Some(memory), Some(1)),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    }
}
#[test]
fn workflow_actor_contract_retained_wire_refuses_unfunded_storage_and_preserves_original() {
    let (application, _, instance, proposal, required, _) =
        approval_journey("actor-storage", 75_000);
    let performed = match approve_instance(
        &application,
        instance,
        &required,
        &proposal,
        WorkflowApprovalDecision::Approve,
        75_010,
    )
    .unwrap()
    {
        WorkflowProgressOutcome::Completed(performed) => performed,
        other => panic!("the genuine actor-governed approval must commit: {other:?}"),
    };
    let record = performed.approval().unwrap().entity();
    let original = performed.receipt().committed_changes().clone();
    let authority =
        worth_query_host::facade::primary_graph::query_test_execution_authority_for_test();
    let zero = authority.request_lease(request(0)).unwrap();
    let execution = worth_execution::ExecutionRequest::leased(&zero);
    let refusal = original
        .retain_workflow_actor_evidence_for_test(
            &execution,
            application.runtime(),
            performed.receipt(),
            record,
        )
        .unwrap_err();
    use worth_query_host::facade::{
        application_contribution::{
            WorthQueryManagedComputationResourceDenial as Resource,
            WorthQueryMemoryLimitLevel as Level,
        },
        primary_graph::{
            WorthQueryApplicationAttemptDenialKind as Attempt,
            WorthQueryProviderSessionDenialKind as Session,
        },
    };
    assert!(
        matches!(refusal.kind(),
        Attempt::ExecutionRequest(Session::ExecutionResource {
            denial: Resource::MemoryLimit { requested, admitted: 0, level: Level::Policy },
            partition_identity: None,
            policy_ancestor: Some(0),
        }) if requested > 0),
        "the original native outer-parser admission must refuse before constructing its DTO: {refusal:?}"
    );
    assert_eq!(refusal.subject(), "workflow actor JSON payload");
    assert!(
        original == *performed.receipt().committed_changes(),
        "storage refusal never mutates the original committed proof"
    );
    drop(zero);
    // Wire length does not bound decode peak: outer text/DTO, context and
    // native retained actor wire coexist. Use the existing paid fixture policy.
    let retry = crate::test_request_policy::serial_request();
    original
        .retain_workflow_actor_evidence_for_test(
            &worth_execution::ExecutionRequest::serial(&retry),
            application.runtime(),
            performed.receipt(),
            record,
        )
        .expect("fresh real storage authority retains the unchanged original proof");
    assert!(original == *performed.receipt().committed_changes());
    assert_eq!(
        read_retention(application.runtime(), application.runtime().current_world()),
        SEED_RETENTION
    );
}
