//! A recorded outcome answers its retry before the caller's lease is opened:
//! every fresh-preparing lane resolves its key in the World's placement first,
//! and only a miss opens the funded call, whose refusal is the original one.
use std::num::NonZeroUsize;
use std::time::{Duration, Instant};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial as Advancement,
        WorthQueryManagedComputationInterruption as Interruption,
    },
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome,
        WorthQueryApplicationProgramMutationPreparation as Preparation,
        WorthQueryApplicationRequestExt,
        WorthQueryApplicationRequestMutationDenial as RequestDenial,
        WorthQueryApplicationRetainedMutationOutcome as RetainedOutcome,
    },
    primary_graph::WorthQueryMutationHandlerWork,
    runtime::{
        CancellationSource, CancellationToken, ExecutionAllocationDenialKind,
        ExecutionAllocationPolicy as Policy, ExecutionResourceLease, LeaseRequest,
    },
};
use worth_relational::facade::history::CommitId;

use crate::document_retention_model::{
    host::{publish_on_first_program, DocumentRetentionRuntime, SEED_RETENTION},
    operator_identity::{authenticate_operator, request_scope},
    programs::RetentionProgramP0,
    readback::{observe_head, read_retention},
    retention_entry::{SetRetentionIntent, DOCUMENT_IDENTITY},
    schema::SetRetentionInput,
};

const RETENTION: u64 = 9;
/// Room for a whole fresh call; no court here reads an owner's edge.
const LEASE_MEMORY: u64 = 16 * 1024 * 1024;

fn lease_request(memory: u64, cancellation: CancellationToken) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
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

/// The commit a recorded answer names, however the record was found.
fn recorded_commit<Denial: std::fmt::Debug, Result: std::fmt::Debug>(
    outcome: Outcome<Denial, Result>,
) -> CommitId {
    match outcome {
        Outcome::AlreadyCommitted(replayed) => replayed.commit_reference().commit_id,
        Outcome::PreviouslyCommitted(replayed) => replayed.commit_id(),
        other => panic!("a recorded key must answer with its commit: {other:?}"),
    }
}

/// Neither the zero-memory lease nor the stopped one can open a call, so a
/// lane that answers under them answered before opening; neither was charged.
fn assert_uncharged(leases: &[&ExecutionResourceLease<'_>]) {
    for lease in leases {
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
}

#[test]
fn every_fresh_preparing_lane_replays_a_recorded_key_before_the_funded_opening() {
    let authority =
        worth_query_host::facade::primary_graph::query_test_execution_authority_for_test();
    let funded = authority
        .request_lease(lease_request(LEASE_MEMORY, CancellationToken::new()))
        .unwrap();
    // Live but holding no memory: any opening charged to it refuses.
    let zero = authority
        .request_lease(lease_request(0, CancellationToken::new()))
        .unwrap();
    let stopping = CancellationSource::new();
    let stopped = authority
        .request_lease(lease_request(LEASE_MEMORY, stopping.token()))
        .unwrap();
    stopping.cancel();
    assert_eq!(
        Policy::Execution(&stopped).check_live().unwrap_err().kind(),
        ExecutionAllocationDenialKind::Cancelled
    );
    let host = publish_on_first_program();
    let runtime = host.runtime();
    let branch = host.current_world();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);

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

    let key = 0x5e91_0001;
    let (committed, _) = request!(&key)
        .execute_in_program_report(&host, Policy::Execution(&funded))
        .into_parts();
    let Outcome::Committed { receipt, .. } = committed.expect("fresh leased commit") else {
        panic!("fresh lawful retention must commit");
    };
    let commit = receipt.commit_reference().commit_id;
    let head = observe_head(runtime, branch);
    // The commit's recorded evidence stays on the lease that funded it for
    // the idempotency window; replays answer from it and add nothing.
    let kept = funded.charged_memory_bytes();

    for lease in [&zero, &stopped, &funded] {
        let (replay, work) = request!(&key)
            .execute_in_program_report(&host, Policy::Execution(lease))
            .into_parts();
        assert_eq!(
            recorded_commit(replay.expect("report lane replays")),
            commit
        );
        assert_eq!(work, WorthQueryMutationHandlerWork::NotStarted);

        let replay = request!(&key)
            .execute_in_program(&host, Policy::Execution(lease))
            .expect("program lane replays");
        assert_eq!(recorded_commit(replay), commit);

        let mut preparing = request!(&key);
        match preparing
            .prepare_in_program(&host, Policy::Execution(lease))
            .expect("preparation lane replays")
        {
            Preparation::Settled(replay) => assert_eq!(recorded_commit(replay), commit),
            Preparation::Prepared(_) => panic!("a recorded key must settle, not prepare"),
        }

        match request!(&key)
            .execute_retained_in_program(&host, Policy::Execution(lease))
            .expect("retained lane replays")
        {
            RetainedOutcome::Other(replay) => assert_eq!(recorded_commit(replay), commit),
            RetainedOutcome::Committed { .. } => panic!("a recorded key must not commit again"),
        }
    }
    assert_uncharged(&[&zero, &stopped]);
    assert_eq!(funded.charged_memory_bytes(), kept);
    assert_eq!(read_retention(runtime, branch), RETENTION);
    assert_eq!(
        observe_head(runtime, branch).selected_commit(),
        head.selected_commit()
    );
    drop(receipt);
}

#[test]
fn a_miss_under_a_stopped_lease_keeps_the_funded_refusal_and_the_key_fresh() {
    let authority =
        worth_query_host::facade::primary_graph::query_test_execution_authority_for_test();
    let funded = authority
        .request_lease(lease_request(LEASE_MEMORY, CancellationToken::new()))
        .unwrap();
    let stopping = CancellationSource::new();
    let stopped = authority
        .request_lease(lease_request(LEASE_MEMORY, stopping.token()))
        .unwrap();
    stopping.cancel();
    let zero = authority
        .request_lease(lease_request(0, CancellationToken::new()))
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

    let key = 0x5e91_0002;
    let (refused, work) = request!(&key)
        .execute_in_program_report(&host, Policy::Execution(&stopped))
        .into_parts();
    let refused = refused.expect_err("an unseen key opens the stopped lease");
    assert!(
        matches!(
            refused,
            RequestDenial::ExecutionRequest(Advancement::Interrupted(Interruption::Cancelled))
        ),
        "the funded opening's own refusal reaches the caller unchanged: {refused:?}"
    );
    assert_eq!(work, WorthQueryMutationHandlerWork::NotStarted);
    let (refused, _) = request!(&key)
        .execute_in_program_report(&host, Policy::Execution(&zero))
        .into_parts();
    assert!(
        matches!(
            refused,
            Err(RequestDenial::ExecutionRequest(Advancement::Resource(_)))
        ),
        "an unseen key opens the zero lease and refuses its admission: {refused:?}"
    );
    assert_eq!(stopped.charged_memory_bytes(), 0);
    assert_eq!(zero.charged_memory_bytes(), 0);
    assert_eq!(read_retention(runtime, branch), SEED_RETENTION);
    assert_eq!(
        observe_head(runtime, branch).selected_commit(),
        original.selected_commit()
    );

    // Neither refusal recorded anything: the key is still fresh.
    let (committed, _) = request!(&key)
        .execute_in_program_report(&host, Policy::Execution(&funded))
        .into_parts();
    assert!(matches!(
        committed.expect("the refused key commits once funded"),
        Outcome::Committed { .. }
    ));
    assert_eq!(read_retention(runtime, branch), RETENTION);
}

/// The request's own refusal answers under any lease. The World-funded step
/// returns it, so the funded call never opens and never prepares a second
/// time: had it opened, the stopped lease would have answered
/// `Interrupted(Cancelled)` and the zero one `Resource`, losing the refusal.
#[test]
fn the_requests_own_refusal_answers_before_the_funded_opening() {
    let authority =
        worth_query_host::facade::primary_graph::query_test_execution_authority_for_test();
    let zero = authority
        .request_lease(lease_request(0, CancellationToken::new()))
        .unwrap();
    let stopping = CancellationSource::new();
    let stopped = authority
        .request_lease(lease_request(LEASE_MEMORY, stopping.token()))
        .unwrap();
    stopping.cancel();
    let host = publish_on_first_program();
    let runtime = host.runtime();
    let branch = host.current_world();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let original = observe_head(runtime, branch);
    let key = 0x5e91_0003;
    for lease in [&zero, &stopped] {
        // The binding is installed through a program, so the ordinary lane
        // refuses it whatever the caller's lease could fund.
        let (refused, work) = runtime
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
            .idempotency(&key)
            .execute_report(Policy::Execution(lease))
            .into_parts();
        assert!(
            matches!(refused, Err(RequestDenial::ApplicationProgramRequired)),
            "the step's own refusal reaches the caller: {refused:?}"
        );
        assert_eq!(work, WorthQueryMutationHandlerWork::NotStarted);
    }
    assert_uncharged(&[&zero, &stopped]);
    assert_eq!(read_retention(runtime, branch), SEED_RETENTION);
    assert_eq!(
        observe_head(runtime, branch).selected_commit(),
        original.selected_commit()
    );
}

const REPLAY_SAMPLES: usize = 15;

/// A host whose history records `population` committed keys, the first of
/// which the court replays.
fn populated(population: u64) -> DocumentRetentionRuntime<RetentionProgramP0> {
    let host = publish_on_first_program();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    for offset in 0..population {
        let key = 0x5e92_0000 + offset;
        let (committed, _) = host
            .runtime()
            .request(&principal, &scope)
            .on_branch(host.current_world())
            .advanced()
            .mutate(SetRetentionIntent {
                input: SetRetentionInput {
                    identity: DOCUMENT_IDENTITY.to_owned(),
                    // Alternate the value so every key records a real change.
                    retention_days: RETENTION + offset % 2,
                },
            })
            .without_source()
            .idempotency(&key)
            .execute_in_program_report(&host, Policy::SystemAllocation)
            .into_parts();
        assert!(matches!(
            committed.expect("each population key commits"),
            Outcome::Committed { .. }
        ));
    }
    host
}

/// One replay of the oldest recorded key under a stopped lease, timed.
fn timed_replay(
    host: &DocumentRetentionRuntime<RetentionProgramP0>,
    stopped: &ExecutionResourceLease<'_>,
) -> Duration {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(host.current_world())
        .advanced()
        .mutate(SetRetentionIntent {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: RETENTION,
            },
        })
        .without_source();
    let key = 0x5e92_0000;
    let started = Instant::now();
    let (replay, work) = request
        .idempotency(&key)
        .execute_in_program_report(host, Policy::Execution(stopped))
        .into_parts();
    let elapsed = started.elapsed();
    assert!(matches!(
        replay.expect("the oldest recorded key replays"),
        Outcome::AlreadyCommitted(_) | Outcome::PreviouslyCommitted(_)
    ));
    assert_eq!(work, WorthQueryMutationHandlerWork::NotStarted);
    elapsed
}

/// A growth class, not a count: thirty-two times the recorded outcomes must
/// not make the World-side pre-admission step grow with them. Samples of the
/// two hosts alternate, so machine load lands on both, and each side keeps its
/// fastest sample, since noise only adds. A step that walked the records would
/// scale toward 32x; the bound leaves room for noise without admitting that.
#[test]
fn the_pre_admission_step_does_not_grow_with_recorded_outcomes() {
    let authority =
        worth_query_host::facade::primary_graph::query_test_execution_authority_for_test();
    let stopping = CancellationSource::new();
    let stopped = authority
        .request_lease(lease_request(0, stopping.token()))
        .unwrap();
    stopping.cancel();
    let small = populated(4);
    let large = populated(128);
    let (mut small_floor, mut large_floor) = (Duration::MAX, Duration::MAX);
    for _ in 0..REPLAY_SAMPLES {
        small_floor = small_floor.min(timed_replay(&small, &stopped));
        large_floor = large_floor.min(timed_replay(&large, &stopped));
    }
    assert!(
        large_floor <= small_floor.saturating_mul(4),
        "replay floor grew from {small_floor:?} at 4 recorded outcomes to {large_floor:?} at 128"
    );
}
