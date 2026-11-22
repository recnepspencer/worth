//! Every real charged prefix, one unit short, ends in its typed owning cause.
pub(crate) use worth_query_testkit_host::facade::host::temporal_conditional_operation::counted_read_stops::*;

#[derive(Clone, Debug, PartialEq, Eq)]
struct OwnerState {
    native_head: worth_relational::facade::branch::RelationalBranchBasisDescriptor,
    world_installed_commits: usize,
    world_recovery_installed: usize,
    provider_commits: usize,
}

fn observe_owner_state(world: &CourtroomWorld) -> OwnerState {
    let core = world.application.runtime_for_test();
    let product = core.product_runtime_for_test();
    let inspection = product.inspection_port_for_test();
    let native_head = product
        .source_for_test()
        .observe_branch_basis(core.relational_branch_identity_for_test())
        .unwrap()
        .0;
    OwnerState {
        native_head,
        world_installed_commits: inspection.history_snapshot().unwrap().installed_commits(),
        world_recovery_installed: inspection.recovery_snapshot().unwrap().installed(),
        provider_commits: core
            .primary_provider_for_test()
            .published_application_commit_count_for_test(),
    }
}

fn observe_under_ceiling(
    world: &CourtroomWorld,
    ceiling: u64,
) -> (Terminal, Option<Cause>, usize, usize, Vec<u64>) {
    let mut port = world.conditional_clock();
    let observer = super::test_read_stops::observe_consults_for_test(|_| {});
    let source = worth_execution::CancellationSource::new();
    let host = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        source.token(),
        None,
    );
    let request = worth_execution::ExecutionRequest::serial(&host);
    let (answer, _) = request
        .run(
            worth_execution::ExecutionWorkCeiling::new(Some(ceiling)),
            |_| port.observe(&request),
        )
        .unwrap();
    let graph::WorthQueryConditionalClockObservationOutcome::Accepted(receipt) = answer else {
        panic!("clock answer")
    };
    let row = receipt.execution_provenance().first().unwrap();
    (
        row.terminal(),
        row.cause(),
        receipt.committed_operation_count(),
        receipt.retained_failed_wake_count(),
        super::test_read_stops::charged_prefixes_for_test(&observer)
            .into_iter()
            .map(|(_, n)| n)
            .collect(),
    )
}

#[test]
fn conditional_operation_work_stop_terminal_matches_its_typed_cause() {
    let _query = worth_query_execution::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    worth_query_execution::facade::primary_graph::place_managed_computations_on_this_thread_for_test(worth_query_execution::facade::primary_graph::WorthQueryExecutionPlacementForTest::Serial);

    // Paid baseline: collect every real charged prefix and validate that the
    // independent owner-state observer actually detects a real commit.
    let baseline_world = CourtroomWorld::publish("ready");
    let baseline_before = observe_owner_state(&baseline_world);
    baseline_world.clock_control.push(1, 5);
    let (terminal, cause, committed, _, charges) =
        observe_under_ceiling(&baseline_world, 64_000_000);
    let baseline_after = observe_owner_state(&baseline_world);
    assert_eq!((terminal, cause), (Terminal::Committed, None));
    assert_eq!(committed, 1);
    assert_ne!(baseline_after.native_head, baseline_before.native_head);
    assert_eq!(
        baseline_after.world_installed_commits,
        baseline_before.world_installed_commits + 1
    );
    assert_eq!(baseline_after.world_recovery_installed, 0);
    assert_eq!(
        baseline_after.provider_commits,
        baseline_before.provider_commits + 1
    );
    assert!(!charges.is_empty(), "the real door charges work");
    let prefixes = charges
        .into_iter()
        .filter(|n| *n > 0)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    // Reuse one prepared courtroom per worker so each short replay pays only
    // its prefix work rather than reinstalling the courtroom per prefix.
    let next = std::sync::atomic::AtomicUsize::new(0);
    let workers = std::thread::available_parallelism().map_or(1, |n| n.get());
    let outcomes = std::thread::scope(|scope| {
        let handles = (0..workers)
            .map(|_| {
                scope.spawn(|| {
                    let _query = worth_query_execution::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
                    worth_query_execution::facade::primary_graph::place_managed_computations_on_this_thread_for_test(worth_query_execution::facade::primary_graph::WorthQueryExecutionPlacementForTest::Serial);
                    let mut world = CourtroomWorld::publish("ready");
                    let initial_state = observe_owner_state(&world);
                    let mut seq = 0_u64;
                    let mut done = Vec::new();
                    loop {
                        let index = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let Some(&required) = prefixes.get(index) else {
                            break done;
                        };
                        seq += 1;
                        world.clock_control.push(seq, 5);
                        let (terminal, cause, committed, failed_wakes, attempted) =
                            observe_under_ceiling(&world, required - 1);
                        let after_state = observe_owner_state(&world);
                        assert_eq!(committed, 0, "charge {required} committed unexpectedly");
                        assert_eq!(
                            after_state, initial_state,
                            "charge {required} mutated owner state on a short work ceiling"
                        );
                        if failed_wakes > 0 || terminal == Terminal::Failed {
                            world.reinstall_conditional_runtime().unwrap();
                            assert_eq!(
                                observe_owner_state(&world),
                                initial_state,
                                "reinstalling conditional runtime after charge {required} altered owner state"
                            );
                        }
                        done.push((required, (terminal, cause, attempted)));
                    }
                })
            })
            .collect::<Vec<_>>();
        handles
            .into_iter()
            .flat_map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert_eq!(outcomes.len(), prefixes.len());
    println!(
        "work_prefix_sweep: executed_prefixes={} max_required={}",
        outcomes.len(),
        prefixes.last().copied().unwrap_or(0)
    );
    for (required, (terminal, cause, attempted)) in outcomes {
        // No fitted charge ordinals: pin the terminal contract for each
        // typed cause, and use the dedicated read-ceiling proofs for exact
        // stage and cause values. This cannot prove per-charge attribution.
        let expected_terminal = match cause.expect("a denied charge has its cause") {
            // Native packet identities now span the suspended proposed-state
            // pass. The terminal depends on its typed stage/cause, not a fitted
            // upper bound on the packet ordinal.
            Cause::ApplicationExecutionDenied {
                stage: graph::WorthQueryApplicationCommitDenialStage::InvariantExecution,
                cause: Ok(graph::WorthQueryProviderSessionDenialKind::ExecutionResource {
                    denial: worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial::WorkExhausted,
                    policy_ancestor: None, ..
                }),
            } | Cause::BridgeConditional(worth_runtime_bridge::facade::BridgeConditionalDenialKind::ExecutionDenied(
                worth_runtime_bridge::facade::BridgeExecutionDenial::WorkCeiling,
            )) | Cause::ApplicationExecutionDenied {
                stage: graph::WorthQueryApplicationCommitDenialStage::Idempotency
                    | graph::WorthQueryApplicationCommitDenialStage::DecisionReadSet
                    // The domain write seal charges each realized effect.
                    | graph::WorthQueryApplicationCommitDenialStage::EffectLowering,
                cause: Ok(graph::WorthQueryProviderSessionDenialKind::ExecutionResource {
                    denial: worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial::WorkExhausted,
                    partition_identity: None | Some(0) | Some(1), policy_ancestor: None,
                }),
            } => Terminal::Failed,
            Cause::ApplicationCommitDenied(
                graph::WorthQueryApplicationCommitDenialKind::ExecutionResource {
                    denial: worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial::WorkExhausted,
                    partition_identity: None | Some(0) | Some(1), policy_ancestor: None,
                } | graph::WorthQueryApplicationCommitDenialKind::ExecutionNestedPatternStopped {
                    partition_identity: None | Some(0) | Some(1),
                },
            ) | Cause::Reentry(graph::WorthQueryConditionalReentryFailure::ApplicationAttempt(
                graph::WorthQueryApplicationAttemptDenialKind::ExecutionRequest(
                    graph::WorthQueryProviderSessionDenialKind::ExecutionResource {
                        denial: worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial::WorkExhausted,
                        partition_identity: None | Some(0) | Some(1),
                        policy_ancestor: None,
                    },
                ),
            )) => Terminal::Retryable,
            other => panic!("unexpected work-stop cause at a real charge: {other:?}"),
        };
        assert_eq!(terminal, expected_terminal, "charge {required}");
        assert!(!attempted.is_empty());
    }
}
