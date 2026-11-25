//! Coupled-output close-resume and interrupted-retention durability courts.
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

#[test]
fn refused_home_close_resume_preserves_live_coupled_receipts() {
    let host = world::open_directory_host(false);
    {
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.request(&principal, &scope);
        let outcome = request
            .advanced()
            .mutate(Intent {
                key: "a".into(),
                value: 2,
            })
            .without_source()
            .idempotency(&"coupled-receipts-before-preflight".to_owned())
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(
            matches!(outcome, WorthQueryApplicationMutationOutcome::Committed { .. }),
            "{outcome:?}"
        );
    }

    let assert_live_coupled_receipts =
        |runtime: &WorthQueryProgramApplicationRuntime<Schema, program::Program>| {
            let scope = request_scope();
            let principal = authenticate_operator(runtime.installed_schema(), &scope);
            let request = runtime.request(&principal, &scope);
            let mut first_demand = request
                .demand(first::Demand("a".into()))
                .start_in_program::<program::Program, program::FirstTree>(runtime)
                .unwrap();
            let WorthQueryApplicationOutputDemandProgress::Settled(first_settled) =
                first_demand.settle(&request).unwrap()
            else {
                panic!("first coupled output must stay settled");
            };
            assert_eq!(first_settled.producer_contacts_in_this_demand(), 0);
            assert!(
                first_settled.application_commit_receipt().is_some(),
                "first coupled completion receipt must survive refused home close"
            );
            let mut second_demand = request
                .demand(second::Demand("a".into()))
                .start_in_program::<program::Program, program::SecondTree>(runtime)
                .unwrap();
            let WorthQueryApplicationOutputDemandProgress::Settled(second_settled) =
                second_demand.settle(&request).unwrap()
            else {
                panic!("second coupled output must stay settled");
            };
            assert_eq!(second_settled.producer_contacts_in_this_demand(), 0);
            assert!(
                second_settled.application_commit_receipt().is_some(),
                "second coupled completion receipt must survive refused home close"
            );
        };

    assert_live_coupled_receipts(&host);
    let (mut home, directory) = host.into_home();
    // Sweep tight retained-memory ceilings until Query preflight succeeds (detaching
    // live coupled receipts into prepared-close custody) and World/bridge seal refuses
    // in `phase: "world-close"` while the Home is still serving, then verify
    // `resume()` restores every receipt.
    let mut hit_world_close = false;
    let mut budget_bytes = 64u64;
    while budget_bytes <= 65536 {
        let lease = worth_execution::ExecutionResourceLease::serial(
            worth_execution::LeaseRequest {
                policy: worth_foundational::ExecutionRequestPolicy::new(
                    worth_foundational::ExecutionPosture::Serial,
                    worth_foundational::DeterminismContract::CanonicalBitwise,
                    worth_foundational::ExecutionBudget::new(
                        std::num::NonZeroUsize::MIN,
                        Some(budget_bytes),
                        None,
                    ),
                ),
                cancellation: worth_execution::CancellationToken::new(),
                deadline: None,
            },
        )
        .unwrap();
        let exec = worth_execution::ExecutionRequest::leased(&lease);
        match home.close(&exec) {
            Ok(closed) => {
                closed.reclaim();
                panic!("expected phase: \"world-close\" refusal before close succeeded at {budget_bytes} bytes");
            }
            Err(refused) => {
                let is_world_close = refused.phase()
                    == worth_query_host::facade::WorthQueryApplicationHomeClosePhase::WorldClose;
                home = refused
                    .resume()
                    .unwrap_or_else(|r| panic!("serving close refusal must resume: {r:?}"));
                if is_world_close {
                    hit_world_close = true;
                    break;
                }
            }
        }
        budget_bytes += 64;
    }
    assert!(
        hit_world_close,
        "close sweep must reach exact post-preflight phase: \"world-close\" refusal"
    );
    assert_live_coupled_receipts(home.runtime());
    let fresh_lease =
        worth_query_testkit_document_retention::facade::directory_home::directory_lease();
    let fresh_exec = worth_execution::ExecutionRequest::leased(&fresh_lease);
    home.close(&fresh_exec)
        .unwrap_or_else(|r| panic!("healthy close must succeed: {r:?}"))
        .reclaim();
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn interruption_between_coupled_members_keeps_remaining_member_until_flushed_and_reopened() {
    let host = world::open_directory_host(false);
    // Lose the original commit answer after World commits: post-answer flush
    // never runs on this request, so BOTH coupled members must already reside
    // in owner commit-resolution custody before any durable write occurs.
    super::lose_next_commit_answer(&host);
    {
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.request(&principal, &scope);
        let outcome = request
            .advanced()
            .mutate(Intent {
                key: "a".into(),
                value: 2,
            })
            .without_source()
            .idempotency(&"two-coupled-members-interrupted".to_owned())
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(
            matches!(
                outcome,
                WorthQueryApplicationMutationOutcome::Commit(
                    WorthQueryApplicationUncommitted::Indeterminate(_)
                )
            ),
            "lost original commit answer must return Indeterminate after World committed: {outcome:?}"
        );
    }

    // Interrupt the first durable flush between member 1 and member 2 using a
    // 1-unit work ceiling: member 1 persists for the first time and pops, while
    // member 2 stays queued in retryable owner custody.
    let one_member_lease = worth_execution::ExecutionResourceLease::serial(
        worth_execution::LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Serial,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(
                    std::num::NonZeroUsize::MIN,
                    Some(256 << 20),
                    Some(1),
                ),
            ),
            cancellation: worth_execution::CancellationToken::new(),
            deadline: None,
        },
    )
    .unwrap();
    let one_member_exec = worth_execution::ExecutionRequest::leased(&one_member_lease);
    let denial = host
        .flush_pending_coupled_outputs(&one_member_exec)
        .expect_err("1-unit work ceiling must stop between coupled member 1 and member 2");
    assert!(
        matches!(
            denial.kind(),
            WorthQueryOutputDemandDenialKind::ExecutionRequest(_)
        ),
        "{denial:?}"
    );

    // An interrupted request cannot checkpoint or close while member 2 remains pending.
    let cancelled = worth_execution::CancellationSource::new();
    cancelled.cancel();
    let cancelled_lease = worth_execution::ExecutionResourceLease::serial(
        worth_execution::LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Serial,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(
                    std::num::NonZeroUsize::MIN,
                    Some(256 << 20),
                    None,
                ),
            ),
            cancellation: cancelled.token(),
            deadline: None,
        },
    )
    .unwrap();
    let cancelled_exec = worth_execution::ExecutionRequest::leased(&cancelled_lease);
    assert!(
        matches!(
            host.home().checkpoint(&cancelled_exec),
            Err(worth_query_host::facade::WorthQueryApplicationHomeCheckpointDenial::CoupledOutputRetention(_))
        ),
        "checkpoint must refuse while pending coupled output cannot flush"
    );
    let (home, directory) = host.into_home();
    let refused = home
        .close(&cancelled_exec)
        .err()
        .expect("cancelled close must refuse while pending coupled output remains");
    let home = refused
        .resume()
        .unwrap_or_else(|_| panic!("pre-seal close refusal must resume cleanly"));

    // A second 1-unit flush succeeds, proving the first 1-unit flush completed
    // member 1 and stopped strictly between member 1 and member 2.
    let second_member_lease = worth_execution::ExecutionResourceLease::serial(
        worth_execution::LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Serial,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(
                    std::num::NonZeroUsize::MIN,
                    Some(256 << 20),
                    Some(1),
                ),
            ),
            cancellation: worth_execution::CancellationToken::new(),
            deadline: None,
        },
    )
    .unwrap();
    let second_member_exec = worth_execution::ExecutionRequest::leased(&second_member_lease);
    home.runtime()
        .flush_pending_coupled_outputs(&second_member_exec)
        .unwrap_or_else(|denial| {
            panic!("exactly one remaining coupled member must fit in 1 work unit: {denial:?}")
        });

    // A fresh request checkpoints and closes cleanly, and reopen reuses BOTH
    // coupled members with zero producer contacts.
    let fresh_lease =
        worth_query_testkit_document_retention::facade::directory_home::directory_lease();
    let fresh_exec = worth_execution::ExecutionRequest::leased(&fresh_lease);
    home.checkpoint(&fresh_exec)
        .unwrap_or_else(|denial| panic!("fresh checkpoint must succeed after flush: {denial:?}"));
    home.close(&fresh_exec)
        .unwrap_or_else(|denial| panic!("fresh close must succeed after flush: {denial:?}"))
        .reclaim();

    let reopened = world::reopen_same_build(directory, false);
    let scope = request_scope();
    let principal = authenticate_operator(reopened.installed_schema(), &scope);
    let request = reopened.request(&principal, &scope);
    let mut first_demand = request
        .demand(first::Demand("a".into()))
        .start_in_program::<program::Program, program::FirstTree>(&reopened)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(first_settled) =
        first_demand.settle(&request).unwrap()
    else {
        panic!("reopened first coupled output must settle");
    };
    assert_eq!(first_settled.producer_contacts_in_this_demand(), 0);
    let mut second_demand = request
        .demand(second::Demand("a".into()))
        .start_in_program::<program::Program, program::SecondTree>(&reopened)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(second_settled) =
        second_demand.settle(&request).unwrap()
    else {
        panic!("reopened second coupled output must settle");
    };
    assert_eq!(second_settled.producer_contacts_in_this_demand(), 0);
}
