//! Real installed mutation entry, physical custody, refusal, and replay.
use std::num::NonZeroUsize;
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial as Advancement,
        WorthQueryManagedComputationInterruption as Interruption,
        WorthQueryManagedComputationResourceDenial as Resource, WorthQueryMemoryLimitLevel,
    },
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome,
        WorthQueryApplicationProgramMutationPreparation as Preparation,
        WorthQueryApplicationRequestExt,
        WorthQueryApplicationRequestMutationDenial as RequestDenial,
    },
    primary_graph::{
        WorthQueryApplicationCommitDenialKind as CommitKind,
        WorthQueryApplicationCommitDenialStage as CommitStage, WorthQueryApplicationUncommitted,
        WorthQueryMutationHandlerWork,
    },
    runtime::{
        CancellationSource, CancellationToken, ExecutionAllocationDenialKind,
        ExecutionAllocationPolicy as Policy, ExecutionResourceLease, LeaseRequest,
    },
};

use crate::document_retention_model::{
    host::{publish_on_first_program, SEED_RETENTION},
    operator_identity::{authenticate_operator, request_scope},
    readback::{observe_head, read_retention},
    retention_entry::{SetRetentionIntent, DOCUMENT_IDENTITY},
    schema::SetRetentionInput,
};

/// The parent lease funds the whole ordinary call, not only its payload. Any
/// room that admits the call will do: no court here reads an owner's edge.
const PAYLOAD_BUDGET: u64 = 16 * 1024 * 1024;
const RETENTION: u64 = 9;

fn lease_request(memory: u64, cancellation: CancellationToken) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            // The lease funds the whole call, so its work admits the handler.
            ExecutionBudget::new(
                NonZeroUsize::new(1).unwrap(),
                Some(memory),
                Some(16_000_000),
            ),
        ),
        deadline: None,
        cancellation,
    }
}

/// A lease too small to open the call refuses before any read; its quote is
/// what the call's own admission charges to the lease.
fn admission_quote(denial: &RequestDenial) -> u64 {
    let RequestDenial::ExecutionRequest(Advancement::Resource(Resource::MemoryLimit {
        requested,
        admitted: 0,
        level: WorthQueryMemoryLimitLevel::Policy,
    })) = denial
    else {
        panic!("a zero lease must refuse the call's own admission: {denial:?}")
    };
    assert!(*requested > 0);
    *requested
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
    // Obtain the process authority; the lease owns this proof's payload limit.
    let authority =
        worth_query_host::facade::primary_graph::query_test_execution_authority_for_test();
    let parent = authority
        .request_lease(lease_request(PAYLOAD_BUDGET, CancellationToken::new()))
        .unwrap();
    // Live but holding no memory: the call it would fund cannot open.
    let zero = authority
        .request_lease(lease_request(0, CancellationToken::new()))
        .unwrap();
    // Funded but stopped: the call it would fund is interrupted at opening.
    let stopping = CancellationSource::new();
    let stopped = authority
        .request_lease(lease_request(PAYLOAD_BUDGET, stopping.token()))
        .unwrap();
    stopping.cancel();
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
                .advanced()
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
    let admission = admission_quote(&refused.expect_err("zero lease cannot open the call"));
    assert_eq!(work, WorthQueryMutationHandlerWork::NotStarted);
    let (refused, work) = request!(&refused_key)
        .execute_in_program_report(&host, Policy::Execution(&stopped))
        .into_parts();
    assert!(
        matches!(
            refused,
            Err(RequestDenial::ExecutionRequest(Advancement::Interrupted(
                Interruption::Cancelled
            )))
        ),
        "a stopped lease interrupts the call it would fund: {refused:?}"
    );
    assert_eq!(work, WorthQueryMutationHandlerWork::NotStarted);
    // Neither refusal published, charged its lease or recorded the key.
    assert_eq!(read_retention(runtime, branch), SEED_RETENTION);
    assert_eq!(
        observe_head(runtime, branch).selected_commit(),
        original.selected_commit()
    );
    assert_eq!(zero.charged_memory_bytes(), 0);
    assert_eq!(stopped.charged_memory_bytes(), 0);
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
            Err(worth_query_host::facade::runtime::MemoryLimitDenial::Capacity { requested, admitted, level: worth_query_host::facade::runtime::MemoryLimitLevel::Policy { ancestor: 0 } }) if requested == PAYLOAD_BUDGET && admitted < PAYLOAD_BUDGET
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
    // The commit call charges the same admission: a zero lease cannot open it.
    let Outcome::Commit(WorthQueryApplicationUncommitted::Denied(denial)) =
        candidate.commit(Policy::Execution(&zero))
    else {
        panic!("an unfunded commit call must remain public Uncommitted::Denied");
    };
    assert_eq!(denial.stage(), CommitStage::ResourceAdmission);
    assert!(
        matches!(
            denial.kind(),
            CommitKind::ExecutionResource {
                denial: Resource::MemoryLimit { requested, admitted: 0, level: WorthQueryMemoryLimitLevel::Policy },
                ..
            } if requested == admission
        ),
        "{denial:?}"
    );
    let mut stopped_retry = request!(&commit_key);
    let candidate = match stopped_retry
        .prepare_in_program(&host, Policy::SystemAllocation)
        .expect("an unopened commit keeps the key fresh")
    {
        Preparation::Prepared(candidate) => candidate,
        Preparation::Settled(_) => panic!("a refused commit call must not settle the key"),
    };
    let stopped_commit = candidate.commit(Policy::Execution(&stopped));
    assert!(
        matches!(
            stopped_commit,
            Outcome::Commit(WorthQueryApplicationUncommitted::Cancelled)
        ),
        "a stopped lease interrupts the commit call it would fund: {stopped_commit:?}"
    );
    assert_eq!(read_retention(runtime, branch), SEED_RETENTION);
    assert_eq!(
        observe_head(runtime, branch).selected_commit(),
        original.selected_commit()
    );
    assert_eq!(zero.charged_memory_bytes(), 0);
    assert_eq!(stopped.charged_memory_bytes(), 0);
    assert_all_budget_available(&parent);

    let committed_key = 0xa110_0004;
    let (committed, work) = request!(&committed_key)
        .execute_in_program_report(&host, Policy::Execution(&parent))
        .into_parts();
    let committed = committed.expect("ordinary leased execution");
    let Outcome::Committed { receipt, .. } = committed else {
        panic!("fresh lawful retention must commit: {committed:?}");
    };
    assert!(matches!(work, WorthQueryMutationHandlerWork::Captured(_)));
    assert_eq!(read_retention(runtime, branch), RETENTION);
    let committed_head = observe_head(runtime, branch);
    assert_ne!(committed_head.selected_commit(), original.selected_commit());

    // Neither the zero lease nor the stopped one can fund a fresh call, so a
    // recorded retry under them performs no allocation and calls no handler.
    assert_eq!(
        Policy::Execution(&stopped).check_live().unwrap_err().kind(),
        ExecutionAllocationDenialKind::Cancelled
    );
    for lease in [&zero, &stopped] {
        let (replay, work) = request!(&committed_key)
            .execute_in_program_report(&host, Policy::Execution(lease))
            .into_parts();
        let replayed_commit = match replay.expect("recorded retry precedes physical admission") {
            Outcome::AlreadyCommitted(replayed) => replayed.commit_reference().commit_id,
            Outcome::PreviouslyCommitted(replayed) => replayed.commit_id(),
            _ => panic!("retry must report the original recorded commit"),
        };
        assert_eq!(replayed_commit, receipt.commit_reference().commit_id);
        assert_eq!(work, WorthQueryMutationHandlerWork::NotStarted);
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
    assert_eq!(read_retention(runtime, branch), RETENTION);
    assert_eq!(
        observe_head(runtime, branch).selected_commit(),
        committed_head.selected_commit()
    );
    drop(receipt);
}
