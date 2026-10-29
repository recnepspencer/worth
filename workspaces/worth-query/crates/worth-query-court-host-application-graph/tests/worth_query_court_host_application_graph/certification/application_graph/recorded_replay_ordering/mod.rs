pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::recorded_replay_ordering::*;

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

pub(crate) mod lookup_growth;
