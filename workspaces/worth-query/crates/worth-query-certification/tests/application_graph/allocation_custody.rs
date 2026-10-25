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
        MutationHandlerExecutionDenial, WorthQueryApplicationCommitDenialKind as CommitKind,
        WorthQueryApplicationCommitDenialStage as CommitStage, WorthQueryApplicationUncommitted,
        WorthQueryInvariantProjectionDenialKind as InvariantKind, WorthQueryMutationHandlerWork,
        WorthQueryOperationProjectionDenialKind as ProjectionKind,
    },
    runtime::{
        CancellationToken, ExecutionAllocationDenial, ExecutionAllocationDenialKind,
        ExecutionAllocationPolicy as Policy, LeaseDenial, LeaseRequest,
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
            ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), Some(memory), Some(1)),
        ),
        deadline: None,
        cancellation,
    }
}

fn assert_memory_refusal(cause: &ExecutionAllocationDenial) {
    assert_eq!(
        cause.kind(),
        ExecutionAllocationDenialKind::Lease(LeaseDenial::MemoryExhausted(
            worth_query_host::facade::runtime::MemoryLimitDenial::Capacity {
                requested: cause.requested_payload_bytes().unwrap(),
                admitted: 0,
                level: worth_query_host::facade::runtime::MemoryLimitLevel::Policy { ancestor: 0 }
            }
        )),
    );
    // Preserve the owner's checked Layout quote. Private completed-fact counts
    // are not exposed here, so this court does not invent a byte/cardinality oracle.
    let quote = cause
        .requested_payload_bytes()
        .expect("checked owner quote survives");
    assert!(quote > 0);
}

#[test]
fn ordinary_mutation_preserves_leased_backing_typed_refusals_and_replay_before_allocation() {
    // Obtain the process authority; the lease owns this proof's payload limit.
    let authority =
        worth_query_host::facade::primary_graph::query_test_execution_authority_for_test();
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

    let foreign = publish_on_first_program();
    let foreign_key = 0xa110_0000;
    foreign.runtime().with_host_advancement(|phase| {
        let mut request = request!(&foreign_key);
        let (answer, work) = request.prepare_in_program_report_in_advancement(&phase, &host, Policy::SystemAllocation).into_parts();
        assert!(matches!(answer, Err(RequestDenial::ExecutionRequest(
            worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial::Resource(
                worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial::ForeignAdvancementPhase
            )
        ))));
        assert_eq!(work, WorthQueryMutationHandlerWork::NotStarted);
    }).unwrap();
    let mut foreign_commit_request = request!(&foreign_key);
    let candidate = match foreign_commit_request
        .prepare_in_program(&host, Policy::SystemAllocation)
        .unwrap()
    {
        Preparation::Prepared(candidate) => candidate,
        Preparation::Settled(_) => panic!("foreign preparation did not claim the key"),
    };
    foreign.runtime().with_host_advancement(|phase| {
        let Outcome::Commit(WorthQueryApplicationUncommitted::Denied(denied)) = candidate.commit_report_in_advancement(&phase, Policy::SystemAllocation).into_outcome() else {
            panic!("foreign phase must not commit the genuine candidate");
        };
        assert!(matches!(denied.kind(), CommitKind::ExecutionResource {
            denial: worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial::ForeignAdvancementPhase, ..
        }));
    }).unwrap();

    let refused_key = 0xa110_0001;
    let (denial, work) = runtime
        .with_application_advancement(&scope, |phase| {
            let zero = crate::test_request_policy::memory_child(
                phase
                    .checked_execution_request_for(runtime.product_runtime())
                    .unwrap(),
                0,
            );
            let mut request = request!(&refused_key);
            let (answer, work) = request
                .prepare_in_program_report_in_advancement(&phase, &host, Policy::Execution(&zero))
                .into_parts();
            let cause = answer
                .err()
                .expect("zero payload budget must refuse scope predicate retention");
            let RequestDenial::Handler(MutationHandlerExecutionDenial::Projection(denial)) = cause
            else {
                panic!("zero payload budget refused at an unexpected owner: {cause:?}");
            };
            (denial, work)
        })
        .unwrap();
    assert_eq!(
        denial.kind(),
        ProjectionKind::InvariantAdmission(InvariantKind::SourceRetentionDenied)
    );
    assert_memory_refusal(denial.allocation_denial().expect("original physical cause"));
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

    let retained_key = 0xa110_0002;
    let mut retained_request = request!(&retained_key);
    let (candidate, mut custody_probe) = runtime
        .with_application_advancement(&scope, |phase| {
            let parent = crate::test_request_policy::memory_child(
                phase
                    .checked_execution_request_for(runtime.product_runtime())
                    .unwrap(),
                PAYLOAD_BUDGET,
            );
            // A nonzero reservation retains this budget's lineage after the phase ends.
            let probe = parent.reserve_memory(1).unwrap();
            let candidate = {
                let child = parent.controlled_child(CancellationToken::new(), None);
                match retained_request
                    .prepare_in_program_report_in_advancement(
                        &phase,
                        &host,
                        Policy::Execution(&child),
                    )
                    .into_outcome()
                    .expect("leased preparation")
                {
                    Preparation::Prepared(candidate) => candidate,
                    Preparation::Settled(_) => panic!("fresh key must prepare a candidate"),
                }
            };
            (candidate, probe)
        })
        .unwrap();
    // Supplied child, parent lease, and attempt-local request control have ended.
    assert_eq!(read_retention(runtime, branch), SEED_RETENTION);
    assert_eq!(
        observe_head(runtime, branch).selected_commit(),
        original.selected_commit()
    );
    assert!(
        matches!(
            custody_probe.resize(PAYLOAD_BUDGET),
            Err(worth_query_host::facade::runtime::MemoryLimitDenial::Capacity {
                requested, admitted,
                level: worth_query_host::facade::runtime::MemoryLimitLevel::Policy { ancestor: 0 }
            }) if requested == PAYLOAD_BUDGET && admitted < PAYLOAD_BUDGET
        ),
        "candidate owns its backing after the preparation phase ends"
    );
    drop(candidate);
    custody_probe
        .resize(PAYLOAD_BUDGET)
        .expect("all candidate backing released");
    drop(custody_probe);

    let commit_key = 0xa110_0003;
    let mut system_request = request!(&commit_key);
    let candidate = match system_request
        .prepare_in_program(&host, Policy::SystemAllocation)
        .expect("explicit System preparation")
    {
        Preparation::Prepared(candidate) => candidate,
        Preparation::Settled(_) => panic!("fresh System key must prepare"),
    };
    let Outcome::Commit(WorthQueryApplicationUncommitted::Denied(denial)) = runtime
        .with_application_advancement(&scope, |phase| {
            let zero = crate::test_request_policy::memory_child(
                phase
                    .checked_execution_request_for(runtime.product_runtime())
                    .unwrap(),
                0,
            );
            candidate
                .commit_report_in_advancement(&phase, Policy::Execution(&zero))
                .into_outcome()
        })
        .unwrap()
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

    let committed_key = 0xa110_0004;
    let (committed, work) = runtime
        .with_application_advancement(&scope, |phase| {
            request!(&committed_key)
                .execute_in_program_report_in_advancement(&phase, &host)
                .into_parts()
        })
        .unwrap();
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
