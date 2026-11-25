//! C4b.1: Genuine filesystem media fault sweeps (`FailBefore` &
//! `IndeterminateAfterEffect` across every observed `(MediaOperationRole, nth)`)
//! for Host checkpoint publication AND recovered custody continuation.
use super::super::{opening, *};
use super::media_fault_sweep::{
    enumerate_admissible_cases, measure_op_media_roles, open_with_one_shot_fault, panic_message,
    MediaCase,
};
use super::oracle_verifier::{
    build_reference, open_dir, prepare_prefix, verify_reopen_and_continue, ReferenceSnapshots,
    TARGET_KEY, TARGET_VAL,
};
use super::seam_kill_sweep::kill_at_seam;
use super::{RestorableHome, TAIL_VARIANCE_BOUND};
use host_observation::{adversarial_verdict::RecoveredPrefix, requests};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
};
use worth_execution::ExecutionRequest;
use worth_query_host::facade::primary_graph::WorthQueryApplicationIdempotencyResolution as Resolution;
use worth_runtime_world::facade::WorldRecoveredContinuationOutcome as Continued;
use worth_store::physical_runtime::certification::MediaOperationRole;

const OP_MEDIA_CHILD: &str =
    "process::application::host_oracle::adversarial::media_fault_ops::host_op_media_fault_child";
const ENV_HOME: &str = "WORTH_ADV_OP_MEDIA_HOME";
const ENV_MARKERS: &str = "WORTH_ADV_OP_MEDIA_MARKERS";
const ENV_SPEC: &str = "WORTH_ADV_OP_MEDIA_SPEC";
const MAX_TAIL_RETRIES: usize = 4;

type RoleCounts = BTreeMap<usize, u64>;

#[test]
fn host_checkpoint_media_fault_sweep_recovers_to_memory_oracle() {
    let req = opening::serial();
    let execution = ExecutionRequest::serial(&req);
    let reference = build_reference(&execution);
    let workers = op_worker_count();
    let calibrated = prepare_calibrated_workers(workers, "checkpoint", |loc, exec| {
        prepare_prefix(loc, exec);
    });
    let counts = verify_and_select_counts("checkpoint", &calibrated, true);
    let cases = enumerate_admissible_cases(&counts);
    let verdicts = run_parallel_op_cases("checkpoint", &calibrated, &cases, &reference, false);
    assert_eq!(verdicts.len(), cases.len());
    for v in verdicts {
        assert_eq!(v, RecoveredPrefix::PreOperation);
    }
}

#[test]
fn host_continuation_media_fault_sweep_recovers_to_memory_oracle() {
    let req = opening::serial();
    let execution = ExecutionRequest::serial(&req);
    let reference = build_reference(&execution);
    let custody_ordinal = discover_custody_ordinal(&execution);
    let workers = op_worker_count();
    let calibrated = prepare_calibrated_workers(workers, "continue", move |loc, exec| {
        prepare_prefix(loc, exec);
        kill_at_seam(loc, &format!("mutate:{custody_ordinal}:after_root"));
    });
    let counts = verify_and_select_counts("continue", &calibrated, false);
    let cases = enumerate_admissible_cases(&counts);
    let verdicts = run_parallel_op_cases("continue", &calibrated, &cases, &reference, true);
    assert_eq!(verdicts.len(), cases.len());
    assert!(
        verdicts
            .iter()
            .any(|v| matches!(v, RecoveredPrefix::UnpublishedCustody { .. })),
        "continuation media faults must observe pre-publish UnpublishedCustody"
    );
    assert!(
        verdicts
            .iter()
            .any(|v| matches!(v, RecoveredPrefix::CommittedInFlight)),
        "continuation media faults must observe post-publish CommittedInFlight"
    );
}

fn op_worker_count() -> usize {
    std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(4)
        .clamp(2, 6)
}

fn prepare_calibrated_workers(
    workers: usize,
    op: &'static str,
    build: impl Fn(&Path, &ExecutionRequest<'_, '_>) + Sync,
) -> Vec<(RestorableHome, RoleCounts)> {
    let homes: Vec<RestorableHome> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..workers)
            .map(|_| {
                scope.spawn(|| {
                    let req = opening::serial();
                    let exec = ExecutionRequest::serial(&req);
                    RestorableHome::prepare(|loc| build(loc, &exec))
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let req = opening::serial();
    let exec = ExecutionRequest::serial(&req);
    homes
        .into_iter()
        .map(|home| {
            home.restore();
            let counts = calibrate_in_process(home.location(), op, &exec);
            (home, counts)
        })
        .collect()
}

fn verify_and_select_counts(
    op: &str,
    calibrated: &[(RestorableHome, RoleCounts)],
    require_exact: bool,
) -> Vec<(usize, u64)> {
    assert!(!calibrated.is_empty());
    let first = &calibrated[0].1;
    assert!(!first.is_empty(), "{op} must perform identified media ops");
    let first_roles: Vec<usize> = first.keys().copied().collect();
    let mut mins: BTreeMap<usize, u64> = first.clone();
    let mut maxs: BTreeMap<usize, u64> = first.clone();
    for (_, counts) in &calibrated[1..] {
        let roles: Vec<usize> = counts.keys().copied().collect();
        assert_eq!(
            roles, first_roles,
            "{op} active media roles must agree across homes"
        );
        if require_exact {
            assert_eq!(
                counts, first,
                "{op} media role calibration must agree identically across homes"
            );
        }
        for (&role, &count) in counts {
            mins.entry(role).and_modify(|m| *m = (*m).min(count));
            maxs.entry(role).and_modify(|m| *m = (*m).max(count));
        }
    }
    let max_spread = TAIL_VARIANCE_BOUND.saturating_mul(4);
    for (&role, &min_c) in &mins {
        let max_c = maxs[&role];
        assert!(
            min_c > 0,
            "{op} role {:?} must have positive count on every home",
            MediaOperationRole::ALL[role]
        );
        assert!(
            max_c.saturating_sub(min_c) <= max_spread,
            "{op} role {:?} count spread too wide across homes: min={min_c}, max={max_c}, bound={max_spread}",
            MediaOperationRole::ALL[role]
        );
    }
    maxs.into_iter().collect()
}

fn discover_custody_ordinal(execution: &ExecutionRequest<'_, '_>) -> usize {
    let probe_root = RestorableHome::prepare(|loc| prepare_prefix(loc, execution));
    for ordinal in 1..=8 {
        probe_root.restore();
        kill_at_seam(
            probe_root.location(),
            &format!("mutate:{ordinal}:after_root"),
        );
        let (home, _) = open_dir(
            probe_root.location(),
            None,
            Arc::new(AtomicUsize::new(0)),
            execution,
        );
        let is_custody = matches!(
            requests::resolve_original(home.runtime(), TARGET_KEY, TARGET_VAL).unwrap(),
            Resolution::ProductUnpublished(_)
        );
        drop(home);
        if is_custody {
            return ordinal;
        }
    }
    panic!("mutate must have an after_root cut in ProductUnpublished");
}

fn run_parallel_op_cases(
    op: &'static str,
    calibrated: &[(RestorableHome, RoleCounts)],
    cases: &[MediaCase],
    reference: &ReferenceSnapshots,
    allow_tail_variance: bool,
) -> Vec<RecoveredPrefix> {
    let queue = Mutex::new((0..cases.len()).collect::<Vec<usize>>());
    let aborted = Arc::new(AtomicBool::new(false));
    let results: Mutex<Vec<Option<Result<RecoveredPrefix, String>>>> =
        Mutex::new((0..cases.len()).map(|_| None).collect());
    std::thread::scope(|scope| {
        for (home, worker_counts) in calibrated {
            let (queue, results) = (&queue, &results);
            let aborted = Arc::clone(&aborted);
            scope.spawn(move || {
                let req = opening::serial();
                let exec = ExecutionRequest::serial(&req);
                loop {
                    if aborted.load(Ordering::Acquire) {
                        break;
                    }
                    let idx = {
                        let mut q = queue.lock().unwrap();
                        let Some(pos) = q.iter().position(|&i| {
                            let (role_idx, nth, _) = cases[i];
                            worker_counts.get(&role_idx).copied().unwrap_or(0) >= nth
                        }) else {
                            break;
                        };
                        q.swap_remove(pos)
                    };
                    let (role_idx, _, _) = cases[idx];
                    let worker_total = worker_counts[&role_idx];
                    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        run_one_op_case(
                            op,
                            home,
                            cases[idx],
                            (worker_total, allow_tail_variance),
                            (reference, &exec),
                        )
                    }))
                    .map_err(|payload| panic_message(&*payload));
                    if outcome.is_err() {
                        aborted.store(true, Ordering::Release);
                    }
                    results.lock().unwrap()[idx] = Some(outcome);
                }
            });
        }
    });

    super::parallel::collect_case_results(op, cases, results.into_inner().unwrap())
}

fn run_one_op_case(
    op: &str,
    home: &RestorableHome,
    case: MediaCase,
    tail: (u64, bool),
    ctx: (&ReferenceSnapshots, &ExecutionRequest<'_, '_>),
) -> RecoveredPrefix {
    let (role_idx, nth, dir_tag) = case;
    let (worker_total, allow_tail_variance) = tail;
    let (reference, execution) = ctx;
    let spec_str = format!("{op}:{role_idx}:{nth}:{dir_tag}");
    let spec = PathBuf::from(&spec_str);
    let label = format!(
        "{op}:{:?}#{nth}:{dir_tag}",
        MediaOperationRole::ALL[role_idx]
    );
    let max_attempts = if allow_tail_variance {
        MAX_TAIL_RETRIES + 1
    } else {
        1
    };
    let mut last_actual = None;
    let mut parked = false;
    for attempt in 1..=max_attempts {
        home.restore();
        let markers = tempfile::tempdir().unwrap();
        let ready = markers.path().join("ready");
        let finished = markers.path().join("finished");
        let end = crate::killed_child::run_and_kill(
            OP_MEDIA_CHILD,
            &[
                (ENV_HOME, home.location()),
                (ENV_MARKERS, markers.path()),
                (ENV_SPEC, &spec),
            ],
            &ready,
            &finished,
        );
        if end == crate::killed_child::ChildEnd::KilledAtPark {
            let marker = std::fs::read_to_string(&ready).unwrap_or_default();
            assert_eq!(
                marker, spec_str,
                "{label}: ready marker must match active spec"
            );
            parked = true;
            break;
        }
        let note = std::fs::read_to_string(&finished).unwrap_or_default();
        let actual: u64 = note
            .trim()
            .strip_prefix("missed:actual=")
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| {
                panic!("{label}: child ended without parking on attempt {attempt} (note={note:?})")
            });
        let retryable_tail_miss = allow_tail_variance
            && actual > 0
            && actual < nth
            && actual >= worker_total.saturating_sub(TAIL_VARIANCE_BOUND);
        assert!(
            retryable_tail_miss,
            "{label}: unexpected miss on attempt {attempt} (actual={actual}, calibrated={worker_total}, bound={TAIL_VARIANCE_BOUND})"
        );
        last_actual = Some(actual);
    }
    assert!(
        parked,
        "{label}: failed to fire and park after {max_attempts} attempts (last_actual={last_actual:?}, calibrated={worker_total}, bound={TAIL_VARIANCE_BOUND})"
    );
    verify_reopen_and_continue(home.location(), reference, execution, &label)
}

fn execute_target_op(
    home: &host_observation::OracleHome,
    op: &str,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<(), String> {
    match op {
        "checkpoint" => home
            .checkpoint(execution)
            .map(|_| ())
            .map_err(|e| format!("{e:?}")),
        "continue" => {
            let resolution = requests::resolve_original(home.runtime(), TARGET_KEY, TARGET_VAL)
                .map_err(|e| format!("resolve_original failed: {e:?}"))?;
            let Resolution::ProductUnpublished(effects) = resolution else {
                return Err(format!(
                    "continuation target must start in ProductUnpublished custody, got {resolution:?}"
                ));
            };
            let recovery = effects.into_recovery();
            for _ in 0..64 {
                match recovery.continue_original_in_request(execution) {
                    Ok(Continued::Pending(c)) => drop(c),
                    Ok(Continued::Published { .. }) => return Ok(()),
                    other => return Err(format!("continuation ended without publish: {other:?}")),
                }
            }
            Err("continuation exceeded 64 Pending rounds".into())
        }
        other => panic!("unknown op {other}"),
    }
}

#[test]
fn host_op_media_fault_child() {
    let Some(location) = std::env::var_os(ENV_HOME).map(PathBuf::from) else {
        return;
    };
    let markers = PathBuf::from(std::env::var_os(ENV_MARKERS).unwrap());
    let spec = std::env::var(ENV_SPEC).unwrap();
    let req = opening::serial();
    let execution = ExecutionRequest::serial(&req);
    let mut parts = spec.split(':');
    let op = parts.next().unwrap();
    let role_idx: usize = parts.next().unwrap().parse().unwrap();
    let nth: u64 = parts.next().unwrap().parse().unwrap();
    let dir_tag = parts.next().unwrap();
    let (home, activation) =
        open_with_one_shot_fault(&location, role_idx, nth, dir_tag, &execution);
    activation.arm().unwrap();
    let completed = execute_target_op(&home, op, &execution);
    if activation.is_consumed() {
        crate::killed_child::announce(&markers.join("ready"), spec.as_bytes());
        crate::killed_child::park();
    }
    let actual = activation.matching_operation_count();
    let note = match completed {
        Ok(()) => format!("missed:actual={actual}"),
        Err(reason) => format!("failed:actual={actual}:{reason}"),
    };
    crate::killed_child::announce(&markers.join("finished"), note.as_bytes());
}

fn calibrate_in_process(
    location: &Path,
    op: &str,
    execution: &ExecutionRequest<'_, '_>,
) -> RoleCounts {
    measure_op_media_roles(location, execution, |home, exec| {
        execute_target_op(home, op, exec)
            .unwrap_or_else(|e| panic!("{op} calibration must succeed: {e}"));
    })
    .into_iter()
    .collect()
}
