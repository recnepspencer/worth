//! Real installed mutation entry, physical custody, refusal, and replay.
use std::num::NonZeroUsize;
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_execution::facade::provider_session::WorthQueryDecisionReadSetDenialKind;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome,
        WorthQueryApplicationProgramMutationPreparation as Preparation,
        WorthQueryApplicationRequestExt,
        WorthQueryApplicationRequestMutationDenial as RequestDenial,
    },
    primary_graph::{
        MutationHandlerExecutionDenial, WorthQueryApplicationAttemptDenialKind as AttemptKind,
        WorthQueryApplicationCommitDenialKind as CommitKind,
        WorthQueryApplicationCommitDenialStage as CommitStage, WorthQueryApplicationUncommitted,
        WorthQueryMutationHandlerWork,
    },
    runtime::{
        CancellationToken, ExecutionAllocationDenial, ExecutionAllocationDenialKind,
        ExecutionAllocationPolicy as Policy, ExecutionAuthority, ExecutionAuthorityConfig,
        ExecutionResourceLease, LeaseDenial, LeaseRequest,
    },
};

use crate::document_retention_model::{
    host::{publish_on_first_program, SEED_RETENTION},
    operator_identity::{authenticate_operator, request_scope},
    readback::{observe_head, read_retention},
    retention_entry::{SetRetentionIntent, DOCUMENT_IDENTITY},
    schema::SetRetentionInput,
};

const PAYLOAD_BUDGET: u64 = 64 * 1024;
const RETENTION: u64 = 9;

fn lease_request(memory: u64, cancellation: CancellationToken) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), memory, 1),
        ),
        deadline: None,
        cancellation,
    }
}

fn assert_memory_refusal(cause: &ExecutionAllocationDenial) {
    assert_eq!(
        cause.kind(),
        ExecutionAllocationDenialKind::Lease(LeaseDenial::ResourceExhausted),
    );
    // Preserve the owner's checked Layout quote. Private completed-fact counts
    // are not exposed here, so this court does not invent a byte/cardinality oracle.
    let quote = cause
        .requested_payload_bytes()
        .expect("checked owner quote survives");
    assert!(quote > 0);
}

fn assert_all_budget_available(lease: &ExecutionResourceLease<'_>) {
    drop(
        lease
            .reserve_memory(PAYLOAD_BUDGET)
            .expect("all inline payload charge released"),
    );
}

#[test]
fn ordinary_mutation_preserves_leased_backing_typed_refusals_and_replay_before_allocation() {
    // One actual process authority in this filtered test. The ordinary fixture
    // has no World authority; supplying a process lease is physical admission.
    let authority = ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
        max_workers: NonZeroUsize::new(1).unwrap(),
        charged_memory_bytes: Some(PAYLOAD_BUDGET),
    })
    .expect("one process authority");
    let parent = authority
        .request_lease(lease_request(PAYLOAD_BUDGET, CancellationToken::new()))
        .unwrap();
    let stopped = CancellationToken::new();
    let zero = authority
        .request_lease(lease_request(0, stopped.clone()))
        .unwrap();
    let host = publish_on_first_program();
    let runtime = host.runtime();
    let branch = host.current_world();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let original = observe_head(runtime, branch);

    macro_rules! request {
        ($key:expr) => {
            runtime
                .request(&principal, &scope)
                .on_branch(branch)
                .mutate(SetRetentionIntent {
                    input: SetRetentionInput {
                        identity: DOCUMENT_IDENTITY.to_owned(),
                        retention_days: RETENTION,
                    },
                })
                .without_source()
                .idempotency($key)
        };
    }

    let refused_key = 0xa110_0001;
    let (refused, work) = request!(&refused_key)
        .execute_in_program_report(&host, Policy::Execution(&zero))
        .into_parts();
    let Err(RequestDenial::Handler(MutationHandlerExecutionDenial::Attempt(denial))) = refused
    else {
        panic!("zero payload budget must retain the original attempt refusal");
    };
    assert_eq!(denial.kind(), AttemptKind::AllocationDenied);
    assert_memory_refusal(denial.allocation_denial().expect("original physical cause"));
    let WorthQueryMutationHandlerWork::Captured(work) = work else {
        panic!("the real decision projection ran before completion admission");
    };
    assert!(work.handler_contacted());
    assert_eq!(read_retention(runtime, branch), SEED_RETENTION);
    assert_eq!(
        observe_head(runtime, branch).selected_commit(),
        original.selected_commit()
    );
    assert_all_budget_available(&parent);

    let retained_key = 0xa110_0002;
    let mut retained_request = request!(&retained_key);
    let candidate = {
        let child = parent.controlled_child(CancellationToken::new(), None);
        match retained_request
            .prepare_in_program(&host, Policy::Execution(&child))
            .expect("leased preparation")
        {
            Preparation::Prepared(candidate) => candidate,
            Preparation::Settled(_) => panic!("fresh key must prepare a candidate"),
        }
    }; // Both the supplied child and attempt-local request control have ended.
    assert_eq!(read_retention(runtime, branch), SEED_RETENTION);
    assert_eq!(
        observe_head(runtime, branch).selected_commit(),
        original.selected_commit()
    );
    assert!(
        matches!(
            parent.reserve_memory(PAYLOAD_BUDGET),
            Err(LeaseDenial::ResourceExhausted)
        ),
        "the unpublished candidate owns the backing charge after child drop"
    );
    drop(candidate);
    assert_all_budget_available(&parent);

    let commit_key = 0xa110_0003;
    let mut system_request = request!(&commit_key);
    let candidate = match system_request
        .prepare_in_program(&host, Policy::SystemAllocation)
        .expect("explicit System preparation")
    {
        Preparation::Prepared(candidate) => candidate,
        Preparation::Settled(_) => panic!("fresh System key must prepare"),
    };
    assert_all_budget_available(&parent);
    let Outcome::Commit(WorthQueryApplicationUncommitted::Denied(denial)) =
        candidate.commit(Policy::Execution(&zero))
    else {
        panic!("provider backing refusal must remain public Uncommitted::Denied");
    };
    assert_eq!(denial.kind(), CommitKind::ProviderRejected);
    assert_eq!(denial.stage(), CommitStage::DecisionReadSet);
    let failure = denial
        .decision_read_set_failure()
        .expect("full provider failure survives");
    assert_eq!(
        failure.kind(),
        WorthQueryDecisionReadSetDenialKind::AllocationDenied
    );
    assert_memory_refusal(
        failure
            .allocation_denial()
            .expect("provider's original physical cause"),
    );
    assert_eq!(read_retention(runtime, branch), SEED_RETENTION);
    assert_eq!(
        observe_head(runtime, branch).selected_commit(),
        original.selected_commit()
    );
    assert_all_budget_available(&parent);

    let committed_key = 0xa110_0004;
    let (committed, work) = request!(&committed_key)
        .execute_in_program_report(&host, Policy::Execution(&parent))
        .into_parts();
    let Outcome::Committed { receipt, .. } = committed.expect("ordinary leased execution") else {
        panic!("fresh lawful retention must commit");
    };
    assert!(matches!(work, WorthQueryMutationHandlerWork::Captured(_)));
    assert_eq!(read_retention(runtime, branch), RETENTION);
    let committed_head = observe_head(runtime, branch);
    assert_ne!(committed_head.selected_commit(), original.selected_commit());

    // Stop the physical zero-budget lease while keeping Query authorization live.
    // A recorded retry must not perform fresh allocation or call the handler.
    stopped.cancel();
    assert_eq!(
        Policy::Execution(&zero).check_live().unwrap_err().kind(),
        ExecutionAllocationDenialKind::Cancelled
    );
    let (replay, work) = request!(&committed_key)
        .execute_in_program_report(&host, Policy::Execution(&zero))
        .into_parts();
    let replayed_commit = match replay.expect("recorded retry precedes physical admission") {
        Outcome::AlreadyCommitted(replayed) => replayed.commit_reference().commit_id,
        Outcome::PreviouslyCommitted(replayed) => replayed.commit_id(),
        _ => panic!("retry must report the original recorded commit"),
    };
    assert_eq!(replayed_commit, receipt.commit_reference().commit_id);
    assert_eq!(work, WorthQueryMutationHandlerWork::NotStarted);
    assert_eq!(read_retention(runtime, branch), RETENTION);
    assert_eq!(
        observe_head(runtime, branch).selected_commit(),
        committed_head.selected_commit()
    );
    drop(receipt);
}
