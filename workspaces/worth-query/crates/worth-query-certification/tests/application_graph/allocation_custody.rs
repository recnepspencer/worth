//! Real installed mutation entry, physical custody, refusal, and replay.
use std::num::NonZeroUsize;
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_execution::facade::provider_session::WorthQueryDecisionReadSetDenialKind;
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial as Advancement,
        WorthQueryManagedComputationResourceDenial as Resource, WorthQueryMemoryLimitLevel,
    },
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome,
        WorthQueryApplicationProgramMutationPreparation as Preparation,
        WorthQueryApplicationRequestExt,
        WorthQueryApplicationRequestMutationDenial as RequestDenial,
    },
    primary_graph::{
        MutationHandlerExecutionDenial, WorthQueryApplicationCommitDenialKind as CommitKind,
        WorthQueryApplicationCommitDenialStage as CommitStage, WorthQueryApplicationUncommitted,
        WorthQueryInvariantProjectionDenialKind as InvariantKind, WorthQueryMutationHandlerWork,
        WorthQueryOperationProjectionDenialKind as ProjectionKind,
    },
    runtime::{
        CancellationToken, ExecutionAllocationDenial, ExecutionAllocationDenialKind,
        ExecutionAllocationPolicy as Policy, ExecutionResourceLease, LeaseDenial, LeaseRequest,
    },
};

use crate::document_retention_model::{
    host::{publish_on_first_program, SEED_RETENTION},
    operator_identity::{authenticate_operator, request_scope},
    readback::{observe_head, read_retention},
    retention_entry::{with_decision_hold, SetRetentionIntent, DOCUMENT_IDENTITY},
    schema::SetRetentionInput,
};

/// The parent lease funds the whole ordinary call, not only its payload.
const PAYLOAD_BUDGET: u64 = crate::lease_quote::DECLARED_BUDGET;
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

/// Asserts the owner's own lease refusal and returns the room it was admitted.
fn assert_memory_refusal(cause: &ExecutionAllocationDenial) -> u64 {
    // Preserve the owner's checked Layout quote. Private completed-fact counts
    // are not exposed here, so this court does not invent a byte/cardinality oracle.
    let quote = cause
        .requested_payload_bytes()
        .expect("checked owner quote survives");
    assert!(quote > 0);
    let ExecutionAllocationDenialKind::Lease(LeaseDenial::MemoryExhausted(
        worth_query_host::facade::runtime::MemoryLimitDenial::Capacity {
            requested,
            admitted,
            // The charging worker sits two controlled levels below the
            // child that funds the whole call, three below the lease.
            level: worth_query_host::facade::runtime::MemoryLimitLevel::Policy { ancestor: 3 },
        },
    )) = cause.kind()
    else {
        panic!("the owner's own quote refuses at the lease: {cause:?}")
    };
    assert_eq!(requested, quote);
    assert!(admitted < requested, "{cause:?}");
    admitted
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
    let stopped = worth_query_host::facade::runtime::CancellationSource::new();
    let zero = authority
        .request_lease(lease_request(0, stopped.token()))
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
    let admission = admission_quote(&refused.err().expect("zero lease cannot open the call"));
    assert_eq!(work, WorthQueryMutationHandlerWork::NotStarted);
    // The lease funds the whole call, and principal resolution's native query
    // completion reserves and releases a result bound larger than the scope
    // predicate's retention before the handler. So the lease funds every
    // owner before the handler in full, and the decision holds all of its
    // room but what the projection keeps; each owner refusing before the
    // retention quotes its share of that keep (MENTALITY §7).
    let ((refused, work, admitted_only), _) = crate::lease_quote::learn(
        0,
        |keep| {
            let lease = authority
                .request_lease(lease_request(
                    crate::lease_quote::DECLARED_BUDGET,
                    CancellationToken::new(),
                ))
                .unwrap();
            let (refused, work) =
                with_decision_hold(crate::lease_quote::keep_at_decision(&lease, keep), || {
                    request!(&refused_key)
                        .execute_in_program_report(&host, Policy::Execution(&lease))
                        .into_parts()
                });
            (refused, work, lease)
        },
        |(refused, _, _)| match refused.as_ref().err()? {
            RequestDenial::Handler(MutationHandlerExecutionDenial::Projection(denial))
                if denial.kind()
                    != ProjectionKind::InvariantAdmission(InvariantKind::SourceRetentionDenied) =>
            {
                crate::lease_quote::allocation_shortfall(denial.allocation_denial()?)
            }
            _ => None,
        },
    );
    let denial = match refused {
        Err(RequestDenial::Handler(MutationHandlerExecutionDenial::Projection(denial))) => denial,
        Err(denial) => panic!("zero payload budget refused at an unexpected owner: {denial:?}"),
        Ok(_) => panic!("zero payload budget must refuse scope predicate retention"),
    };
    assert_eq!(
        denial.kind(),
        ProjectionKind::InvariantAdmission(InvariantKind::SourceRetentionDenied)
    );
    // The decision holds all but the learned keep, so the retention has none.
    assert_eq!(
        assert_memory_refusal(denial.allocation_denial().expect("original physical cause")),
        0
    );
    let invariant = denial
        .invariant_denial()
        .expect("full scope retention refusal");
    assert_eq!(invariant.kind(), InvariantKind::SourceRetentionDenied);
    assert_eq!(invariant.allocation_denial(), denial.allocation_denial());
    let WorthQueryMutationHandlerWork::Captured(work) = work else {
        panic!("the real decision projection ran before candidate construction");
    };
    assert!(work.handler_contacted());
    assert_eq!(Some(work.projection_work()), denial.projection_work());
    assert_eq!(work.projection_work().equality_lookups(), 1);
    assert_eq!(work.projection_work().index_candidates_examined(), 1);
    assert_eq!(read_retention(runtime, branch), SEED_RETENTION);
    assert_eq!(
        observe_head(runtime, branch).selected_commit(),
        original.selected_commit()
    );
    assert_all_budget_available(&parent);
    assert_eq!(admitted_only.charged_memory_bytes(), 0);

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
    // The commit likewise charges its admission and provider plan before the
    // read set; exactly those leave the read set no room. The commit runs no
    // handler, so it runs on a tracked child whose owners charge what they
    // keep rather than a share of the declared budget.
    let ((denial, admitted_only), _) = crate::lease_quote::learn(
        admission,
        |funded| {
            let lease = authority
                .request_lease(lease_request(
                    crate::lease_quote::DECLARED_BUDGET,
                    CancellationToken::new(),
                ))
                .unwrap();
            let mut system_retry = request!(&commit_key);
            let candidate = match system_retry
                .prepare_in_program(&host, Policy::SystemAllocation)
                .expect("an unopened commit keeps the key fresh")
            {
                Preparation::Prepared(candidate) => candidate,
                Preparation::Settled(_) => panic!("a refused commit call must not settle the key"),
            };
            let call = crate::lease_quote::tracked(&lease);
            let Outcome::Commit(WorthQueryApplicationUncommitted::Denied(denial)) =
                crate::lease_quote::with_room(&lease, funded, || {
                    candidate.commit(Policy::Execution(&call))
                })
            else {
                panic!("provider backing refusal must remain public Uncommitted::Denied");
            };
            (denial, lease)
        },
        |(denial, _)| crate::lease_quote::commit_shortfall(denial),
    );
    assert_eq!(denial.kind(), CommitKind::ProviderRejected);
    assert_eq!(denial.stage(), CommitStage::DecisionReadSet);
    let failure = denial
        .decision_read_set_failure()
        .expect("full provider failure survives");
    assert_eq!(
        failure.kind(),
        WorthQueryDecisionReadSetDenialKind::AllocationDenied
    );
    // Earlier owners release their transient peaks before the read set, which
    // refuses its own quote with the room they left.
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
    assert_eq!(admitted_only.charged_memory_bytes(), 0);

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
