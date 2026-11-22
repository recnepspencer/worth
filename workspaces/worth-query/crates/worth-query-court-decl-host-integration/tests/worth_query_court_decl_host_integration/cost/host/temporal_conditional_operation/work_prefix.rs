//! Every real charged prefix, one unit short, ends in its typed owning cause.
pub(crate) use worth_query_testkit_host::facade::host::temporal_conditional_operation::counted_read_stops::*;
use std::sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, Arc, Barrier, OnceLock};

type PaidTrace = Arc<(Vec<(usize, u64)>, Vec<(usize, u64)>)>;
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

fn observe_under_ceiling(world: &CourtroomWorld, ceiling: u64) -> ObserveAnswer {
    observe_port_under_ceiling(&mut world.conditional_clock(), ceiling)
}

fn observe_port_under_ceiling(port: &mut ConditionalPort<'_>, ceiling: u64) -> ObserveAnswer {
    let observer = super::test_read_stops::observe_consults_for_test(|_| {});
    let source = worth_execution::CancellationSource::new();
    let host = worth_execution::SerialRequest::from_memory(worth_execution::SerialMemoryBudget::new(None), source.token(), None);
    let request = worth_execution::ExecutionRequest::serial(&host);
    let (answer, report) = request.run(worth_execution::ExecutionWorkCeiling::new(Some(ceiling)), |_| port.observe(&request)).unwrap();
    let (graph::WorthQueryConditionalClockObservationOutcome::Accepted(receipt)
    | graph::WorthQueryConditionalClockObservationOutcome::Duplicate(receipt)) = answer else { panic!("clock answer") };
    let row = receipt.execution_provenance().first().unwrap();
    ObserveAnswer {
        terminal: row.terminal(), cause: row.cause(),
        committed: receipt.committed_operation_count(), retained_due: receipt.retained_due_wake_count(),
        charged_work: report.charged_work(), charges: super::test_read_stops::charged_prefixes_for_test(&observer),
    }
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
                .begin_branch_transaction(
                    &basis,
                    worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
                )
                .unwrap();
            let candidate = rt.prepare_branch_transaction(&request, tx).unwrap();
            drop(candidate);
            let snap = vis.snapshot_for_observation(&obs).unwrap();
            vis.release_snapshot(&snap).unwrap();
        }
    });
}

fn warm_port_under_ceiling(port: &mut ConditionalPort<'_>, ceiling: u64) {
    let source = worth_execution::CancellationSource::new();
    let host = worth_execution::SerialRequest::from_memory(worth_execution::SerialMemoryBudget::new(None), source.token(), None);
    let request = worth_execution::ExecutionRequest::serial(&host);
    let (answer, _) = request.run(worth_execution::ExecutionWorkCeiling::new(Some(ceiling)), |_| port.observe(&request)).unwrap();
    let (graph::WorthQueryConditionalClockObservationOutcome::Accepted(receipt)
    | graph::WorthQueryConditionalClockObservationOutcome::Duplicate(receipt)) = answer else { panic!("clock answer") };
    assert_eq!(receipt.committed_operation_count(), 0);
}

fn promote_to_retained_wake(world: &CourtroomWorld, port: &mut ConditionalPort<'_>, cutoff: u64, before: &OwnerState) {
    world.clock_control.push(1, 5);
    let promoted = observe_port_under_ceiling(port, cutoff - 1);
    assert_eq!((promoted.committed, promoted.retained_due), (0, 1));
    assert!(!matches!(promoted.cause, Some(Cause::BridgeConditional(_))));
    assert_eq!(observe_owner_state(world), *before);
    port.reset_retained_wakes_for_replay_for_test();
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
    label: &str,
    world: &CourtroomWorld,
    initial: &OwnerState,
    baseline_charges: &[(usize, u64)],
    baseline_pos: usize,
    required: u64,
    replay: ObserveAnswer,
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

#[test]
fn conditional_operation_work_stop_terminal_matches_its_typed_cause() {
    let _query = install_thread_observation();
    let t0 = std::time::Instant::now();
    let phase1_workers = std::thread::available_parallelism().map_or(1, |n| n.get()).min(28);
    let workers = phase1_workers.min(24);

    // 1. Healthy cold trace: locate the wake-retention boundary, then derive
    // the authoritative cold worklist and cover 100% of pre-retention
    // BridgeConditional charges plus the final cold prefix directly against it.
    let cold_baseline = || {
        let world = CourtroomWorld::publish("ready");
        let before = observe_owner_state(&world);
        world.clock_control.push(1, 5);
        let paid = observe_under_ceiling(&world, 64_000_000);
        validate_paid_commit(&before, &observe_owner_state(&world), &paid);
        let unique = unique_positive_prefixes(&paid.charges);
        assert!(!unique.is_empty(), "the real door charges work");
        (paid.charges, unique)
    };
    let (_, probe_unique) = cold_baseline();
    let probe_count = (usize::BITS - probe_unique.len().leading_zeros()) as usize + 1;
    let mut probe_worlds: Vec<CourtroomWorld> = std::thread::scope(|s| {
        (0..probe_count).map(|_| s.spawn(|| { let _q = install_thread_observation(); CourtroomWorld::publish("ready") }))
            .collect::<Vec<_>>().into_iter().map(|h| h.join().unwrap()).collect()
    });
    let (mut lo, mut hi) = (0usize, probe_unique.len());
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let world = probe_worlds.pop().unwrap();
        world.clock_control.push(1, 5);
        let probe = observe_under_ceiling(&world, probe_unique[mid].1 - 1);
        if probe.retained_due == 1 && !matches!(probe.cause, Some(Cause::BridgeConditional(_))) { hi = mid; } else { lo = mid + 1; }
    }
    assert!(lo < probe_unique.len(), "cold sweep reaches reenter_retained_wakes");
    let admission_len = lo + 1;
    let (cold_charges, cold_unique) = cold_baseline();
    let admission_worklist = &cold_unique[..admission_len];
    let admission_cutoff = admission_worklist[admission_len - 1].1 + 64;
    let admission_covered: Vec<AtomicBool> = (0..admission_len).map(|_| AtomicBool::new(false)).collect();
    let cold_final = *cold_unique.last().expect("cold final prefix");
    let tail_start = admission_len.saturating_sub(8);
    let (cold_charges_ref, adm_list_ref, adm_cov_ref) =
        (&cold_charges, admission_worklist, &admission_covered);
    // Run cold_final and the post-encoding tail first while instance IDs remain
    // in the same digit cohort as cold_baseline, then sweep the earlier prefixes.
    std::thread::scope(|scope| {
        let final_handle = scope.spawn(move || {
            let _query = install_thread_observation();
            let world = CourtroomWorld::publish("ready");
            let initial = observe_owner_state(&world);
            world.clock_control.push(1, 5);
            let replay = observe_under_ceiling(&world, cold_final.1 - 1);
            verify_prefix_replay("cold_final", &world, &initial, cold_charges_ref, cold_final.0, cold_final.1, replay);
        });
        let tail_handles: Vec<_> = (tail_start..admission_len).map(|idx| scope.spawn(move || {
            let _query = install_thread_observation();
            let (pos, required) = adm_list_ref[idx];
            let world = CourtroomWorld::publish("ready");
            let initial = observe_owner_state(&world);
            world.clock_control.push(1, 5);
            let replay = observe_under_ceiling(&world, required - 1);
            verify_prefix_replay("bridge_admission", &world, &initial, cold_charges_ref, pos, required, replay);
            assert!(!adm_cov_ref[idx].swap(true, Ordering::AcqRel));
        })).collect();
        final_handle.join().unwrap();
        for h in tail_handles { h.join().unwrap(); }
    });
    let next_adm = AtomicUsize::new(0);
    let next_adm_ref = &next_adm;
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..phase1_workers).map(|_| scope.spawn(move || {
            let _query = install_thread_observation();
            loop {
                let idx = next_adm_ref.fetch_add(1, Ordering::Relaxed);
                if idx >= tail_start { break; }
                let (pos, required) = adm_list_ref[idx];
                let world = CourtroomWorld::publish("ready");
                let initial = observe_owner_state(&world);
                world.clock_control.push(1, 5);
                let replay = observe_under_ceiling(&world, required - 1);
                verify_prefix_replay("bridge_admission", &world, &initial, cold_charges_ref, pos, required, replay);
                assert!(!adm_cov_ref[idx].swap(true, Ordering::AcqRel));
            }
        })).collect();
        for h in handles { h.join().unwrap(); }
    });
    assert!(admission_covered.iter().all(|c| c.load(Ordering::Acquire)));

    // 2. Healthy paid prepared-wake operation: stabilize process-global world
    // IDs (256..999), per-world transaction IDs (1000..9999), per-world
    // observation/diagnostic retention buffers (past 256 via low-ceiling
    // calls), and global provider-session generations (1000..9999) with zero
    // owner movement. Thread 0 executes ONE healthy paid commit to derive the
    // exact worklist, and all workers replay every single worklist entry
    // directly against that one healthy paid trace.
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..40)
            .map(|_| scope.spawn(|| { let _q = install_thread_observation(); CourtroomWorld::publish("ready") }))
            .collect();
        for h in handles { drop(h.join().unwrap()); }
    });
    const HIGH_WARM: usize = 42;
    let total_threads = workers + 1;
    let primed_barrier = Barrier::new(total_threads);
    let baseline_ready = Barrier::new(total_threads);
    let paid_trace: OnceLock<PaidTrace> = OnceLock::new();
    let covered_cell: OnceLock<Vec<AtomicBool>> = OnceLock::new();
    let next_op = AtomicUsize::new(0);
    let (pb_ref, br_ref, trace_ref, cov_ref, next_op_ref) =
        (&primed_barrier, &baseline_ready, &paid_trace, &covered_cell, &next_op);

    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..total_threads).map(|tid| scope.spawn(move || {
            let _query = install_thread_observation();
            let world = CourtroomWorld::publish("ready");
            let initial = observe_owner_state(&world);
            prime_relational_tx_plateau(&world);
            let mut port = world.conditional_clock();
            port.prime_replay_plateau_for_test();
            promote_to_retained_wake(&world, &mut port, admission_cutoff, &initial);
            for _ in 0..HIGH_WARM {
                world.clock_control.push(1, 5);
                warm_port_under_ceiling(&mut port, 81_500);
                port.reset_retained_wakes_for_replay_for_test();
            }
            assert_eq!(observe_owner_state(&world), initial);
            pb_ref.wait();

            if tid == 0 {
                world.clock_control.push(1, 5);
                let paid = observe_port_under_ceiling(&mut port, 64_000_000);
                validate_paid_commit(&initial, &observe_owner_state(&world), &paid);
                let worklist = unique_positive_prefixes(&paid.charges);
                assert!(!worklist.is_empty(), "healthy paid operation charges work");
                let _ = cov_ref.set((0..worklist.len()).map(|_| AtomicBool::new(false)).collect());
                let _ = trace_ref.set(Arc::new((paid.charges, worklist)));
                br_ref.wait();
                return;
            }
            br_ref.wait();
            let paid = trace_ref.get().expect("healthy paid baseline ready");
            let covered = cov_ref.get().expect("worklist coverage bitmap ready");
            loop {
                let idx = next_op_ref.fetch_add(1, Ordering::Relaxed);
                let Some(&(pos, required)) = paid.1.get(idx) else { break; };
                world.clock_control.push(1, 5);
                let replay = observe_port_under_ceiling(&mut port, required - 1);
                verify_prefix_replay("operation", &world, &initial, &paid.0, pos, required, replay);
                port.reset_retained_wakes_for_replay_for_test();
                assert!(!covered[idx].swap(true, Ordering::AcqRel));
            }
        })).collect();
        for h in handles { h.join().unwrap(); }
    });

    let paid = paid_trace.get().expect("healthy paid baseline");
    let covered = covered_cell.get().expect("coverage bitmap");
    assert_eq!(covered.len(), paid.1.len());
    assert!(covered.iter().all(|c| c.load(Ordering::Acquire)), "every paid prefix must be covered");
    let op_max = paid.1.last().expect("paid worklist").1;
    let total_executed = admission_len + 1 + paid.1.len();
    println!(
        "work_prefix_sweep: operation_prefixes={}/{} (max_required={op_max}) bridge_admission_prefixes={admission_len}/{admission_len} cold_final={} total_executed={total_executed} t_total={:.2?}",
        paid.1.len(), paid.1.len(), cold_final.1, t0.elapsed(),
    );
}
