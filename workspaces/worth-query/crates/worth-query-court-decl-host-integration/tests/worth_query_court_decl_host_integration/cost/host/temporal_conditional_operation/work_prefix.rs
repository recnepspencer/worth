//! Every real charged prefix of one healthy cold trace, one unit short, ends in its typed owning cause.
pub(crate) use worth_query_testkit_host::facade::host::temporal_conditional_operation::counted_read_stops::*;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

type ConditionalPort<'a> = worth_query_host::facade::primary_graph::WorthQueryConditionalClockObservationPort<
    'a,
    worth_query_testkit_host::facade::host::temporal_conditional_operation::schema::TemporalHostSchema,
    worth_query_testkit_host::facade::host::temporal_conditional_operation::contract::TemporalReadyNode,
    worth_query_testkit_host::facade::host::temporal_conditional_operation::adapters::CourtroomClock,
>;

#[derive(Clone, Debug, PartialEq, Eq)]
struct OwnerState {
    native_head: worth_relational::facade::branch::RelationalBranchBasisDescriptor,
    world_installed_commits: usize,
    world_recovery_installed: usize,
    provider_commits: usize,
}

struct ObserveAnswer {
    terminal: Terminal,
    cause: Option<Cause>,
    committed: usize,
    retained_due: usize,
    charged_work: u64,
    charges: Vec<(usize, u64)>,
}

fn install_thread_observation() -> worth_query_execution::facade::primary_graph::WorthQueryExecutionObservation {
    let guard = worth_query_execution::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    worth_query_execution::facade::primary_graph::place_managed_computations_on_this_thread_for_test(
        worth_query_execution::facade::primary_graph::WorthQueryExecutionPlacementForTest::Serial,
    );
    guard
}

fn observe_owner_state(world: &CourtroomWorld) -> OwnerState {
    let core = world.application.runtime_for_test();
    let product = core.product_runtime_for_test();
    let inspection = product.inspection_port_for_test();
    let native_head = product.source_for_test().observe_branch_basis(core.relational_branch_identity_for_test()).unwrap().0;
    OwnerState {
        native_head,
        world_installed_commits: inspection.history_snapshot().unwrap().installed_commits(),
        world_recovery_installed: inspection.recovery_snapshot().unwrap().installed(),
        provider_commits: core.primary_provider_for_test().published_application_commit_count_for_test(),
    }
}

fn extract_answer(
    answer: graph::WorthQueryConditionalClockObservationOutcome<
        worth_query_testkit_host::facade::host::temporal_conditional_operation::adapters::CourtroomClock,
    >,
    charged_work: u64,
    charges: Vec<(usize, u64)>,
) -> ObserveAnswer {
    let (graph::WorthQueryConditionalClockObservationOutcome::Accepted(receipt)
    | graph::WorthQueryConditionalClockObservationOutcome::Duplicate(receipt)) = answer else { panic!("clock answer") };
    let row = receipt.execution_provenance().first().unwrap();
    ObserveAnswer {
        terminal: row.terminal(),
        cause: row.cause(),
        committed: receipt.committed_operation_count(),
        retained_due: receipt.retained_due_wake_count(),
        charged_work,
        charges,
    }
}

fn observe_under_ceiling(world: &CourtroomWorld, ceiling: u64) -> ObserveAnswer {
    observe_port_under_ceiling(&mut world.conditional_clock(), ceiling)
}

fn observe_port_under_ceiling(port: &mut ConditionalPort<'_>, ceiling: u64) -> ObserveAnswer {
    let observer = super::test_read_stops::observe_consults_for_test(|_| {});
    let source = worth_execution::CancellationSource::new();
    let host = worth_execution::SerialRequest::from_memory(worth_execution::SerialMemoryBudget::new(None), source.token(), None);
    let request = worth_execution::ExecutionRequest::serial(&host);
    let (answer, report) = request.run(worth_execution::ExecutionWorkCeiling::new(Some(ceiling)), |_| port.observe(&request)).unwrap();
    extract_answer(answer, report.charged_work(), super::test_read_stops::charged_prefixes_for_test(&observer))
}

fn observe_continuation_under_ceiling(port: &mut ConditionalPort<'_>, bridge_work: u64, ceiling: u64) -> ObserveAnswer {
    let observer = super::test_read_stops::observe_consults_for_test(|_| {});
    let source = worth_execution::CancellationSource::new();
    let host = worth_execution::SerialRequest::from_memory(worth_execution::SerialMemoryBudget::new(None), source.token(), None);
    let request = worth_execution::ExecutionRequest::serial(&host);
    let (answer, report) = request.run(worth_execution::ExecutionWorkCeiling::new(Some(ceiling)), |_| {
        request.run_native_with_checkpoint(|ctx| ctx.checkpoint(bridge_work).unwrap()).unwrap();
        port.reenter_retained_wakes_continuation_for_test(&request)
    }).unwrap();
    extract_answer(answer, report.charged_work(), super::test_read_stops::charged_prefixes_for_test(&observer))
}

fn prime_relational_tx_plateau(world: &CourtroomWorld) {
    let core = world.application.runtime_for_test();
    let branch = core.relational_branch_identity_for_test();
    let graph = core.primary_provider_for_test().graph_for_test();
    let kind_id = graph.layout_for_test().entity_kind_for_test("UnrelatedRecord").unwrap();
    let source = worth_execution::CancellationSource::new();
    let host = worth_execution::SerialRequest::from_memory(worth_execution::SerialMemoryBudget::new(None), source.token(), None);
    let request = worth_execution::ExecutionRequest::serial(&host);
    graph.with_runtime(&request, |rt| {
        let (_, basis) = rt.observe_branch(branch).unwrap();
        let obs = basis.observation();
        let vis = rt.snapshots();
        for b in 0..4u32 {
            let mut batch = worth_relational::facade::mvcc::WorkerIntentBatch::new("prime-lineage");
            for i in 0..250u32 {
                batch = batch.push(worth_relational::facade::transactions::MutationIntent::Create(
                    worth_relational::facade::transactions::CreateIntent::Entity(
                        worth_relational::facade::transactions::EntitySpec {
                            partition_id: worth_relational::facade::identity::PartitionId(60_000),
                            kind_id,
                            client_key: worth_relational::facade::symbols::ClientKey::raw(format!("prime-lineage-{b}-{i}")),
                            fields: worth_relational::facade::transactions::AspectFieldPatch::default(),
                        },
                    ),
                ));
            }
            let mut tx = rt
                .begin_branch_transaction(&basis, worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary())
                .unwrap();
            tx.push_batch(batch, request.allocation_policy()).unwrap();
            drop(rt.prepare_branch_transaction(&request, tx).unwrap());
        }
        for _ in 0..260 {
            let tx = rt
                .begin_branch_transaction(&basis, worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary())
                .unwrap();
            drop(rt.prepare_branch_transaction(&request, tx).unwrap());
            let snap = vis.snapshot_for_observation(&obs).unwrap();
            vis.release_snapshot(&snap).unwrap();
        }
    });
}

fn new_cold_world(prime_rel: bool) -> (CourtroomWorld, OwnerState) {
    let world = CourtroomWorld::publish("ready");
    let initial = observe_owner_state(&world);
    if prime_rel {
        prime_relational_tx_plateau(&world);
        world.conditional_clock().prime_replay_plateau_for_test();
        assert_eq!(observe_owner_state(&world), initial, "uncommitted priming must not move owner state");
    }
    (world, initial)
}

fn promote_to_retained_wake(world: &CourtroomWorld, port: &mut ConditionalPort<'_>, cutoff: u64, before: &OwnerState) -> ObserveAnswer {
    world.clock_control.push(1, 5);
    let promoted = observe_port_under_ceiling(port, cutoff - 1);
    assert_eq!((promoted.committed, promoted.retained_due), (0, 1));
    assert!(!matches!(promoted.cause, Some(Cause::BridgeConditional(_))));
    assert_eq!(observe_owner_state(world), *before);
    port.reset_retained_wakes_for_replay_for_test();
    promoted
}

fn validate_paid_commit(before: &OwnerState, after: &OwnerState, paid: &ObserveAnswer) {
    assert_eq!((paid.terminal, paid.cause.as_ref(), paid.committed), (Terminal::Committed, None, 1));
    assert_ne!(after.native_head, before.native_head);
    assert_eq!(after.world_installed_commits, before.world_installed_commits + 1);
    assert_eq!(after.world_recovery_installed, 0);
    assert_eq!(after.provider_commits, before.provider_commits + 1);
}

fn unique_positive_prefixes(charges: &[(usize, u64)]) -> Vec<(usize, u64)> {
    let mut seen = std::collections::BTreeSet::new();
    charges.iter().enumerate().filter_map(|(pos, &(_, n))| (n > 0 && seen.insert(n)).then_some((pos, n))).collect()
}

fn assert_expected_stop_terminal(required: u64, terminal: Terminal, cause: Option<Cause>) {
    use graph::{
        WorthQueryApplicationAttemptDenialKind as AttemptDenial,
        WorthQueryApplicationCommitDenialKind as CommitDenial,
        WorthQueryApplicationCommitDenialStage as Stage,
        WorthQueryConditionalReentryFailure as ReentryFailure,
        WorthQueryProviderSessionDenialKind as SessionDenial,
    };
    use worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial::WorkExhausted;
    let expected = match cause.expect("a denied charge has its cause") {
        // Native packet identities now span the suspended proposed-state pass.
        // The terminal depends on its typed stage/cause, not a fitted ordinal bound.
        Cause::ApplicationExecutionDenied {
            stage: Stage::InvariantExecution,
            cause: Ok(SessionDenial::ExecutionResource { denial: WorkExhausted, policy_ancestor: None, .. }),
        } | Cause::BridgeConditional(worth_runtime_bridge::facade::BridgeConditionalDenialKind::ExecutionDenied(
            worth_runtime_bridge::facade::BridgeExecutionDenial::WorkCeiling,
        )) | Cause::ApplicationExecutionDenied {
            // The domain write seal charges each realized effect.
            stage: Stage::Idempotency | Stage::DecisionReadSet | Stage::EffectLowering,
            cause: Ok(SessionDenial::ExecutionResource {
                denial: WorkExhausted, partition_identity: None | Some(0) | Some(1), policy_ancestor: None,
            }),
        } => Terminal::Failed,
        Cause::ApplicationCommitDenied(
            CommitDenial::ExecutionResource {
                denial: WorkExhausted, partition_identity: None | Some(0) | Some(1), policy_ancestor: None,
            } | CommitDenial::ExecutionNestedPatternStopped { partition_identity: None | Some(0) | Some(1) },
        ) | Cause::Reentry(ReentryFailure::ApplicationAttempt(AttemptDenial::ExecutionRequest(
            SessionDenial::ExecutionResource {
                denial: WorkExhausted, partition_identity: None | Some(0) | Some(1), policy_ancestor: None,
            },
        ))) => Terminal::Retryable,
        other => panic!("unexpected work-stop cause at a real charge {required}: {other:?}"),
    };
    assert_eq!(terminal, expected, "charge {required}");
}

fn verify_prefix_replay(
    label: &str, world: &CourtroomWorld, initial: &OwnerState,
    baseline_charges: &[(usize, u64)], baseline_pos: usize, required: u64, replay: ObserveAnswer,
) {
    assert_eq!(replay.committed, 0, "{label}: charge {required} committed");
    assert_eq!(observe_owner_state(world), *initial, "{label}: charge {required} mutated owner state");
    assert!(replay.charged_work < required, "{label}: charge {required} billed {} >= {required}", replay.charged_work);
    let stop_pos = replay.charges.iter().position(|&(_, n)| n > required - 1)
        .unwrap_or_else(|| panic!("{label}: charge {required} never crossed {} (len={})", required - 1, replay.charges.len()));
    let prefix_matches = stop_pos == baseline_pos
        && replay.charges[stop_pos].1 == required
        && replay.charges[..=stop_pos].iter().zip(&baseline_charges[..=baseline_pos]).all(|(a, b)| a.1 == b.1);
    if !prefix_matches {
        let first_diff = replay.charges.iter().zip(baseline_charges.iter()).enumerate().find(|(_, (a, b))| a.1 != b.1);
        panic!("{label}: charge {required} stopped at {} (pos {stop_pos}), expected {required} (pos {baseline_pos}); first_diff={first_diff:?}", replay.charges[stop_pos].1);
    }
    assert_expected_stop_terminal(required, replay.terminal, replay.cause);
}

#[allow(clippy::too_many_arguments)]
fn verify_continuation_replay(
    label: &str, world: &CourtroomWorld, initial: &OwnerState, cold_charges: &[(usize, u64)],
    admission_pos: usize, bridge_work: u64, baseline_pos: usize, required: u64, replay: ObserveAnswer,
) {
    assert_eq!(replay.committed, 0, "{label}: charge {required} committed");
    assert_eq!(observe_owner_state(world), *initial, "{label}: charge {required} mutated owner state");
    assert!(replay.charged_work >= bridge_work && replay.charged_work < required, "{label}: charge {required} billed {} outside [{bridge_work}, {required})", replay.charged_work);
    assert_eq!(replay.charges.first().map(|c| c.1), Some(bridge_work), "{label}: charge {required} lost bridge_work");
    let tail = &replay.charges[1..];
    let stop_off = tail.iter().position(|&(_, n)| n > required - 1)
        .unwrap_or_else(|| panic!("{label}: charge {required} never crossed {} (tail len={})", required - 1, tail.len()));
    let expected_off = baseline_pos - admission_pos;
    let cold_slice = &cold_charges[admission_pos..=baseline_pos];
    let prefix_matches = stop_off == expected_off
        && tail[stop_off].1 == required
        && tail[..=stop_off].iter().zip(cold_slice).all(|(a, b)| a.1 == b.1);
    if !prefix_matches {
        let first_diff = tail.iter().zip(cold_slice.iter()).enumerate().find(|(_, (a, b))| a.1 != b.1);
        panic!("{label}: charge {required} stopped at {} (pos {}), expected {required} (pos {baseline_pos}); first_diff={first_diff:?}", tail[stop_off].1, admission_pos + stop_off);
    }
    assert_expected_stop_terminal(required, replay.terminal, replay.cause);
}

#[test]
fn conditional_operation_work_stop_terminal_matches_its_typed_cause() {
    let _query = install_thread_observation();
    let t0 = std::time::Instant::now();
    let phase1_workers = std::thread::available_parallelism().map_or(1, |n| n.get()).min(28);
    let workers = phase1_workers;

    // 1. Stabilize process-global world IDs past 256 (uint16 / 3-digit cohort)
    // and execute ONE healthy paid cold commit to derive the single authoritative
    // cold trace and worklist.
    let next_dummy = AtomicUsize::new(0);
    let next_dummy_ref = &next_dummy;
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..phase1_workers)
            .map(|_| scope.spawn(move || {
                let _q = install_thread_observation();
                while next_dummy_ref.fetch_add(1, Ordering::Relaxed) < 260 {
                    drop(CourtroomWorld::publish("ready"));
                }
            }))
            .collect();
        for h in handles { h.join().unwrap(); }
    });
    let (cold_world, cold_before) = new_cold_world(true);
    cold_world.clock_control.push(1, 5);
    let cold_paid = observe_under_ceiling(&cold_world, 64_000_000);
    validate_paid_commit(&cold_before, &observe_owner_state(&cold_world), &cold_paid);
    let cold_unique = unique_positive_prefixes(&cold_paid.charges);
    assert!(!cold_unique.is_empty(), "healthy cold commit charges work");
    let cold_charges = cold_paid.charges;
    let cold_final = *cold_unique.last().expect("cold final prefix");
    let covered: Vec<AtomicBool> = (0..cold_unique.len()).map(|_| AtomicBool::new(false)).collect();

    // 2. Locate the exact wake-retention entry `admission_end` directly in
    // `cold_unique`, then prove charge-for-charge continuation equivalence on
    // a healthy paid commit plus full-cold boundary and final verification.
    let probe_count = (usize::BITS - cold_unique.len().leading_zeros()) as usize + 1;
    let mut probe_worlds: Vec<(CourtroomWorld, OwnerState)> = std::thread::scope(|s| {
        (0..probe_count).map(|_| s.spawn(|| { let _q = install_thread_observation(); new_cold_world(false) }))
            .collect::<Vec<_>>().into_iter().map(|h| h.join().unwrap()).collect()
    });
    let (mut lo, mut hi) = (0usize, cold_unique.len());
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let (world, _) = probe_worlds.pop().unwrap();
        world.clock_control.push(1, 5);
        let probe = observe_under_ceiling(&world, cold_unique[mid].1 - 1);
        if probe.retained_due == 1 && !matches!(probe.cause, Some(Cause::BridgeConditional(_))) { hi = mid; } else { lo = mid + 1; }
    }
    assert!(lo > 0 && lo < cold_unique.len(), "cold sweep reaches reenter_retained_wakes");
    let admission_end = lo;
    let (admission_pos, admission_first_req) = cold_unique[admission_end];
    let bridge_work = cold_charges[admission_pos - 1].1;
    assert!(bridge_work > 0 && bridge_work < admission_first_req);

    // Prove full-cold boundary stop, exact paid continuation equivalence through
    // commit, and full-cold final stop directly against `cold_charges`.
    {
        let (eq_world, eq_before) = new_cold_world(true);
        let mut eq_port = eq_world.conditional_clock();
        let promoted = promote_to_retained_wake(&eq_world, &mut eq_port, admission_first_req, &eq_before);
        verify_prefix_replay("cold_full_boundary", &eq_world, &eq_before, &cold_charges, admission_pos, admission_first_req, promoted);
        let cont_paid = observe_continuation_under_ceiling(&mut eq_port, bridge_work, 64_000_000);
        validate_paid_commit(&eq_before, &observe_owner_state(&eq_world), &cont_paid);
        assert_eq!(cont_paid.charges.first().map(|c| c.1), Some(bridge_work));
        let cont_tail: Vec<u64> = cont_paid.charges[1..].iter().map(|c| c.1).collect();
        let cold_tail: Vec<u64> = cold_charges[admission_pos..].iter().map(|c| c.1).collect();
        assert_eq!(cont_tail, cold_tail, "prepared continuation must preserve exact cold cumulative Work and prefix positions");
    }
    {
        let (final_world, final_before) = new_cold_world(true);
        final_world.clock_control.push(1, 5);
        let replay = observe_under_ceiling(&final_world, cold_final.1 - 1);
        verify_prefix_replay("cold_full_final", &final_world, &final_before, &cold_charges, cold_final.0, cold_final.1, replay);
    }

    // 3. Cover 100% of `cold_unique`:
    //    - `tail_start..admission_end`: full cold `observe` first (same world-ID
    //      and pre-sweep global cohort as `cold_world`)
    //    - `admission_end..cold_unique.len()`: exact `bridge_work`-precharged
    //      continuations preserving exact original cumulative Work & positions
    //    - `0..tail_start`: full cold `observe` on fresh worlds
    let (cold_charges_ref, cold_unique_ref, covered_ref) = (&cold_charges, &cold_unique, &covered);
    let tail_start = admission_end.saturating_sub(8);
    std::thread::scope(|scope| {
        let handles: Vec<_> = (tail_start..admission_end).map(|idx| scope.spawn(move || {
            let _query = install_thread_observation();
            let (pos, required) = cold_unique_ref[idx];
            let (world, initial) = new_cold_world(false);
            world.clock_control.push(1, 5);
            let replay = observe_under_ceiling(&world, required - 1);
            verify_prefix_replay("cold_pre_retention", &world, &initial, cold_charges_ref, pos, required, replay);
            assert!(!covered_ref[idx].swap(true, Ordering::AcqRel));
        })).collect();
        for h in handles { h.join().unwrap(); }
    });

    let next_post = AtomicUsize::new(admission_end);
    let next_post_ref = &next_post;
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..workers).map(|_| scope.spawn(move || {
            let _query = install_thread_observation();
            let (world, initial) = new_cold_world(true);
            let mut port = world.conditional_clock();
            promote_to_retained_wake(&world, &mut port, admission_first_req, &initial);
            loop {
                let idx = next_post_ref.fetch_add(1, Ordering::Relaxed);
                let Some(&(pos, required)) = cold_unique_ref.get(idx) else { break; };
                let replay = observe_continuation_under_ceiling(&mut port, bridge_work, required - 1);
                verify_continuation_replay(
                    "cold_continuation", &world, &initial, cold_charges_ref,
                    admission_pos, bridge_work, pos, required, replay,
                );
                port.reset_retained_wakes_for_replay_for_test();
                assert!(!covered_ref[idx].swap(true, Ordering::AcqRel));
            }
        })).collect();
        for h in handles { h.join().unwrap(); }
    });

    let next_pre = AtomicUsize::new(0);
    let next_pre_ref = &next_pre;
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..phase1_workers).map(|_| scope.spawn(move || {
            let _query = install_thread_observation();
            loop {
                let idx = next_pre_ref.fetch_add(1, Ordering::Relaxed);
                if idx >= tail_start { break; }
                let (pos, required) = cold_unique_ref[idx];
                let (world, initial) = new_cold_world(false);
                world.clock_control.push(1, 5);
                let replay = observe_under_ceiling(&world, required - 1);
                verify_prefix_replay("cold_pre_retention", &world, &initial, cold_charges_ref, pos, required, replay);
                assert!(!covered_ref[idx].swap(true, Ordering::AcqRel));
            }
        })).collect();
        for h in handles { h.join().unwrap(); }
    });

    assert!(covered.iter().all(|c| c.load(Ordering::Acquire)), "every positive charged prefix of the healthy cold trace must be covered");
    let post_len = cold_unique.len() - admission_end;
    println!(
        "work_prefix_sweep: cold_prefixes={}/{} (pre_retention={admission_end}/{admission_end}, continuation={post_len}/{post_len}, full_cold_boundary_and_final=2/2, bridge_work={bridge_work}, cold_max={}) t_total={:.2?}",
        cold_unique.len(), cold_unique.len(), cold_final.1, t0.elapsed(),
    );
}
