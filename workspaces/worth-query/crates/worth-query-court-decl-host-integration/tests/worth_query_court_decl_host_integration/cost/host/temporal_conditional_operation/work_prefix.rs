//! Every real charged prefix, one unit short, ends in its typed owning cause.
pub(crate) use worth_query_testkit_host::facade::host::temporal_conditional_operation::counted_read_stops::*;
use std::sync::{atomic::{AtomicUsize, Ordering}, Arc, Condvar, Mutex};

type Cohort = Arc<(Vec<(usize, u64)>, Vec<(usize, u64)>)>;
type ConditionalPort<'a> = worth_query_host::facade::primary_graph::WorthQueryConditionalClockObservationPort<
    'a,
    worth_query_testkit_host::facade::host::temporal_conditional_operation::schema::TemporalHostSchema,
    worth_query_testkit_host::facade::host::temporal_conditional_operation::contract::TemporalReadyNode,
    worth_query_testkit_host::facade::host::temporal_conditional_operation::adapters::CourtroomClock,
>;

struct TracePipeline { traces: Vec<Cohort>, final_len: Option<usize>, final_max: u64, aborted: bool }
struct PipelineAbortGuard<'a>(&'a (Mutex<TracePipeline>, Condvar), bool);
impl Drop for PipelineAbortGuard<'_> {
    fn drop(&mut self) {
        if !self.1 { self.0.0.lock().unwrap().aborted = true; self.0.1.notify_all(); }
    }
}

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

fn promote_to_retained_wake(world: &CourtroomWorld, port: &mut ConditionalPort<'_>, cutoff: u64, before: &OwnerState) {
    world.clock_control.push(1, 5);
    let promoted = observe_port_under_ceiling(port, cutoff - 1);
    assert_eq!((promoted.committed, promoted.retained_due), (0, 1));
    assert!(!matches!(promoted.cause, Some(Cause::BridgeConditional(_))));
    assert_eq!(observe_owner_state(world), *before);
    port.reset_retained_wakes_for_replay_for_test();
}

fn with_prepared_world<R>(cutoff: u64, f: impl FnOnce(&CourtroomWorld, &OwnerState, &mut ConditionalPort<'_>) -> R) -> R {
    let world = CourtroomWorld::publish("ready");
    let initial = observe_owner_state(&world);
    let mut port = world.conditional_clock();
    promote_to_retained_wake(&world, &mut port, cutoff, &initial);
    f(&world, &initial, &mut port)
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
    let (_, cold_unique) = cold_baseline();

    let t0 = std::time::Instant::now();
    let workers = std::thread::available_parallelism().map_or(1, |n| n.get()).min(16);

    // Binary search for the exact Phase-1 / Phase-2 boundary using pre-built worlds.
    let probe_count = (usize::BITS - cold_unique.len().leading_zeros()) as usize + 1;
    let mut probe_worlds: Vec<CourtroomWorld> = std::thread::scope(|s| {
        (0..probe_count).map(|_| s.spawn(|| { let _q = install_thread_observation(); CourtroomWorld::publish("ready") }))
            .collect::<Vec<_>>().into_iter().map(|h| h.join().unwrap()).collect()
    });
    let (mut lo, mut hi) = (0usize, cold_unique.len());
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let world = probe_worlds.pop().unwrap();
        world.clock_control.push(1, 5);
        let probe = observe_under_ceiling(&world, cold_unique[mid].1 - 1);
        if probe.retained_due == 1 && !matches!(probe.cause, Some(Cause::BridgeConditional(_))) { hi = mid; } else { lo = mid + 1; }
    }
    assert!(lo < cold_unique.len(), "cold sweep reaches reenter_retained_wakes");
    let phase1_len = lo + 1;

    // Capture a fresh post-probe cold baseline (2-digit ID band) and replay the
    // small framing-sensitive tail (pos >= 200) immediately before the parallel bulk.
    let (p1_charges, p1_unique) = cold_baseline();
    let phase1_cutoff = p1_unique[phase1_len - 1].1 + 64;
    let tail_start = phase1_len.saturating_sub(8);
    let mut phase1_count = 0usize;
    for &(pos, required) in &p1_unique[tail_start..phase1_len] {
        let world = CourtroomWorld::publish("ready");
        let initial = observe_owner_state(&world);
        world.clock_control.push(1, 5);
        let replay = observe_under_ceiling(&world, required - 1);
        verify_prefix_replay("phase1", &world, &initial, &p1_charges, pos, required, replay);
        phase1_count += 1;
    }
    let next_p1 = AtomicUsize::new(0);
    let (p1_charges_ref, p1_unique_ref, next_p1_ref) = (&p1_charges, &p1_unique, &next_p1);
    phase1_count += std::thread::scope(|scope| {
        (0..workers).map(|_| scope.spawn(move || {
            let _query = install_thread_observation();
            let mut count = 0usize;
            loop {
                let idx = next_p1_ref.fetch_add(1, Ordering::Relaxed);
                if idx >= tail_start { break count; }
                let (pos, required) = p1_unique_ref[idx];
                let world = CourtroomWorld::publish("ready");
                let initial = observe_owner_state(&world);
                world.clock_control.push(1, 5);
                let replay = observe_under_ceiling(&world, required - 1);
                verify_prefix_replay("phase1", &world, &initial, p1_charges_ref, pos, required, replay);
                count += 1;
            }
        })).collect::<Vec<_>>().into_iter().map(|h| h.join().unwrap()).sum::<usize>()
    });
    assert_eq!(phase1_count, phase1_len);

    // Capture step-0 paid baseline, replay Phase 2a, and prime global prepare_commit
    // counters past 256 (into MessagePack's constant u16 plateau) in parallel.
    let (step0_cohort, seed_max) = with_prepared_world(phase1_cutoff, |w0, b0, p0| {
        w0.clock_control.push(1, 5);
        let paid0 = observe_port_under_ceiling(p0, 64_000_000);
        validate_paid_commit(b0, &observe_owner_state(w0), &paid0);
        let u0 = unique_positive_prefixes(&paid0.charges);
        let max = u0.last().expect("warm charges").1;
        (Arc::new((paid0.charges, u0)), max)
    });
    let pre_commit_split = step0_cohort.1.iter().position(|&(_, n)| n >= 79_200).unwrap_or(670);
    let next_p2a = AtomicUsize::new(0);
    let (next_p2a_ref, step0_ref) = (&next_p2a, &step0_cohort);
    let phase2a_count: usize = std::thread::scope(|scope| {
        (0..workers).map(|_| scope.spawn(move || {
            let _query = install_thread_observation();
            with_prepared_world(phase1_cutoff, |world, initial, port| {
                let mut count = 0usize;
                loop {
                    let index = next_p2a_ref.fetch_add(1, Ordering::Relaxed);
                    if index >= pre_commit_split { break; }
                    let (pos, required) = step0_ref.1[index];
                    world.clock_control.push(1, 5);
                    let replay = observe_port_under_ceiling(port, required - 1);
                    verify_prefix_replay("phase2a", world, initial, &step0_ref.0, pos, required, replay);
                    port.reset_retained_wakes_for_replay_for_test();
                    count += 1;
                }
                for _ in 0..18 {
                    world.clock_control.push(1, 5);
                    assert_eq!(observe_port_under_ceiling(port, 85_000).committed, 0);
                    port.reset_retained_wakes_for_replay_for_test();
                }
                assert_eq!(observe_owner_state(world), *initial);
                count
            })
        })).collect::<Vec<_>>().into_iter().map(|h| h.join().unwrap()).sum()
    });
    assert_eq!(phase2a_count, pre_commit_split);

    // Pipeline deterministic per-world step traces (steps 0..=256) concurrently with
    // parallel Phase-2b workers (one prepared courtroom per worker + O(1) wake reset).
    const PLATEAU_STEP: usize = 256;
    let pipeline = (
        Mutex::new(TracePipeline { traces: Vec::with_capacity(PLATEAU_STEP + 1), final_len: None, final_max: 0, aborted: false }),
        Condvar::new(),
    );
    let next_p2b = AtomicUsize::new(pre_commit_split);
    let (pipeline_ref, next_p2b_ref) = (&pipeline, &next_p2b);
    let phase2b_count: usize = std::thread::scope(|scope| {
        let producer = scope.spawn(move || {
            let _query = install_thread_observation();
            let mut guard = PipelineAbortGuard(pipeline_ref, false);
            with_prepared_world(phase1_cutoff, |ref_w, ref_b, ref_p| {
                let step0_last = step0_ref.1.len() - 1;
                for step in 0..PLATEAU_STEP {
                    let cap_idx = (pre_commit_split + (step + 1) * workers + 256).min(step0_last);
                    let cap = (step0_ref.1[cap_idx].1 + 512).min(seed_max - 1);
                    ref_w.clock_control.push(1, 5);
                    let r = observe_port_under_ceiling(ref_p, cap);
                    assert_eq!(r.committed, 0);
                    let u = unique_positive_prefixes(&r.charges);
                    pipeline_ref.0.lock().unwrap().traces.push(Arc::new((r.charges, u)));
                    pipeline_ref.1.notify_all();
                    ref_p.reset_retained_wakes_for_replay_for_test();
                }
                assert_eq!(observe_owner_state(ref_w), *ref_b);
                ref_w.clock_control.push(1, 5);
                let paid_plateau = observe_port_under_ceiling(ref_p, 64_000_000);
                validate_paid_commit(ref_b, &observe_owner_state(ref_w), &paid_plateau);
                let u_plateau = unique_positive_prefixes(&paid_plateau.charges);
                let (plateau_len, plateau_max) = (u_plateau.len(), u_plateau.last().expect("plateau charges").1);
                let mut st = pipeline_ref.0.lock().unwrap();
                st.traces.push(Arc::new((paid_plateau.charges, u_plateau)));
                st.final_len = Some(plateau_len);
                st.final_max = plateau_max;
                drop(st);
                pipeline_ref.1.notify_all();
                guard.1 = true;
            });
        });
        let handles: Vec<_> = (0..workers).map(|_| scope.spawn(move || {
            let _query = install_thread_observation();
            with_prepared_world(phase1_cutoff, |world, initial, port| {
                let (mut step, mut count) = (0usize, 0usize);
                loop {
                    let (mut baseline, mut done_len) = {
                        let mut st = pipeline_ref.0.lock().unwrap();
                        while !st.aborted && st.traces.len() <= step && st.final_len.is_none() { st = pipeline_ref.1.wait(st).unwrap(); }
                        if st.aborted { break count; }
                        (Arc::clone(&st.traces[step.min(st.traces.len() - 1)]), st.final_len)
                    };
                    let index = next_p2b_ref.fetch_add(1, Ordering::Relaxed);
                    if done_len.is_some_and(|limit| index >= limit) { break count; }
                    if index >= baseline.1.len() {
                        baseline = {
                            let mut st = pipeline_ref.0.lock().unwrap();
                            while !st.aborted && st.final_len.is_none() { st = pipeline_ref.1.wait(st).unwrap(); }
                            if st.aborted { break count; }
                            done_len = st.final_len;
                            Arc::clone(st.traces.last().unwrap())
                        };
                        if index >= done_len.unwrap() { break count; }
                        while step < PLATEAU_STEP {
                            world.clock_control.push(1, 5);
                            assert_eq!(observe_port_under_ceiling(port, 85_000).committed, 0);
                            port.reset_retained_wakes_for_replay_for_test();
                            step += 1;
                        }
                    }
                    let (pos, required) = baseline.1[index];
                    world.clock_control.push(1, 5);
                    let replay = observe_port_under_ceiling(port, required - 1);
                    verify_prefix_replay("phase2b", world, initial, &baseline.0, pos, required, replay);
                    port.reset_retained_wakes_for_replay_for_test();
                    step += 1;
                    count += 1;
                }
            })
        })).collect();
        producer.join().unwrap();
        handles.into_iter().map(|h| h.join().unwrap()).sum()
    });
    let (warm_paid_len, warm_max) = { let st = pipeline.0.lock().unwrap(); (st.final_len.unwrap(), st.final_max) };
    let phase2_count = phase2a_count + phase2b_count;
    assert_eq!(phase2_count, warm_paid_len);
    let total_executed = phase1_count + phase2_count;
    assert!(total_executed >= cold_unique.len(), "executed {total_executed} < cold {}", cold_unique.len());
    println!(
        "work_prefix_sweep: executed_prefixes={total_executed} (phase1={phase1_count}, phase2={phase2_count}) cold_unique={} cold_max={} warm_max={warm_max} t_total={:.2?}",
        cold_unique.len(), cold_unique.last().map_or(0, |&(_, n)| n), t0.elapsed(),
    );
}
