//! C4b.1: Genuine filesystem media fault sweeps (`FailBefore` &
//! `IndeterminateAfterEffect` across every observed `(MediaOperationRole, nth)`)
//! for Host checkpoint publication AND recovered custody continuation.
use super::super::{opening, *};
use super::media_fault_sweep::{
    decode_directive, enumerate_admissible_cases, panic_message, MediaCase,
};
use super::oracle_verifier::{
    build_reference, open_dir, prepare_prefix, verify_reopen_and_continue, ReferenceSnapshots,
    TARGET_KEY, TARGET_VAL,
};
use super::seam_kill_sweep::kill_at_seam;
use super::RestorableHome;
use host_interruption::opening::AdversarialPlacement;
use host_observation::{adversarial_verdict::RecoveredPrefix, requests};
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use worth_execution::ExecutionRequest;
use worth_query_host::facade::primary_graph::WorthQueryApplicationIdempotencyResolution as Resolution;
use worth_runtime_world::facade::WorldRecoveredContinuationOutcome as Continued;
use worth_store::physical_runtime::{
    certification::MediaOperationRole, FilesystemAccessPosture, FilesystemMediaAdmission,
};

const OP_MEDIA_CHILD: &str =
    "process::application::host_oracle::adversarial::media_fault_ops::host_op_media_fault_child";
const ENV_HOME: &str = "WORTH_ADV_OP_MEDIA_HOME";
const ENV_MARKERS: &str = "WORTH_ADV_OP_MEDIA_MARKERS";
const ENV_SPEC: &str = "WORTH_ADV_OP_MEDIA_SPEC";

#[test]
fn host_checkpoint_media_fault_sweep_recovers_to_memory_oracle() {
    let req = opening::serial();
    let execution = ExecutionRequest::serial(&req);
    let reference = build_reference(&execution);
    let homes: Vec<_> = (0..2)
        .map(|_| RestorableHome::prepare(|loc| prepare_prefix(loc, &execution)))
        .collect();
    let c0 = measure_op_roles_in_child(&homes[0], "checkpoint");
    let c1 = measure_op_roles_in_child(&homes[1], "checkpoint");
    assert_eq!(c0, c1, "checkpoint media role calibration must agree identically across homes");
    assert!(!c0.is_empty(), "Host checkpoint must perform identified media ops");
    let cases = enumerate_admissible_cases(&c0);
    let verdicts = run_parallel_op_cases("checkpoint", &homes, &cases, &reference);
    assert_eq!(verdicts.len(), cases.len(), "every checkpoint media fault must fire");
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
    let homes: Vec<_> = (0..2)
        .map(|_| {
            RestorableHome::prepare(|loc| {
                prepare_prefix(loc, &execution);
                kill_at_seam(loc, &format!("mutate:{custody_ordinal}:after_root"));
            })
        })
        .collect();
    let c0 = measure_op_roles_in_child(&homes[0], "continue");
    let c1 = measure_op_roles_in_child(&homes[1], "continue");
    assert_eq!(c0, c1, "continuation media role calibration must agree identically across homes");
    assert!(!c0.is_empty(), "custody continuation must perform identified media ops");
    let cases = enumerate_admissible_cases(&c0);
    let verdicts = run_parallel_op_cases("continue", &homes, &cases, &reference);
    assert_eq!(verdicts.len(), cases.len(), "every continuation media fault must fire");
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

fn discover_custody_ordinal(execution: &ExecutionRequest<'_, '_>) -> usize {
    let probe_root = RestorableHome::prepare(|loc| prepare_prefix(loc, execution));
    for ordinal in 1..=8 {
        probe_root.restore();
        kill_at_seam(probe_root.location(), &format!("mutate:{ordinal}:after_root"));
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
    homes: &[RestorableHome],
    cases: &[MediaCase],
    reference: &ReferenceSnapshots,
) -> Vec<RecoveredPrefix> {
    let next = AtomicUsize::new(0);
    let aborted = std::sync::atomic::AtomicBool::new(false);
    let results: Mutex<Vec<Option<Result<RecoveredPrefix, String>>>> =
        Mutex::new((0..cases.len()).map(|_| None).collect());
    let (next_ref, abort_ref, results_ref) = (&next, &aborted, &results);
    std::thread::scope(|scope| {
        for home in homes {
            scope.spawn(move || {
                let req = opening::serial();
                let exec = ExecutionRequest::serial(&req);
                let markers = tempfile::tempdir().unwrap();
                loop {
                    if abort_ref.load(Ordering::Acquire) {
                        break;
                    }
                    let idx = next_ref.fetch_add(1, Ordering::Relaxed);
                    let Some(&case) = cases.get(idx) else {
                        break;
                    };
                    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        run_one_op_case(op, (home, markers.path()), case, (reference, &exec))
                    }))
                    .map_err(|payload| panic_message(&*payload));
                    if outcome.is_err() {
                        abort_ref.store(true, Ordering::Release);
                    }
                    results_ref.lock().unwrap()[idx] = Some(outcome);
                }
            });
        }
    });

    let mut verdicts = Vec::with_capacity(cases.len());
    for (idx, slot) in results.into_inner().unwrap().into_iter().enumerate() {
        match slot {
            Some(Ok(v)) => verdicts.push(v),
            Some(Err(msg)) => panic!("{op} media case #{idx} ({:?}) failed: {msg}", cases[idx]),
            None => panic!("{op} media case #{idx} skipped after earlier failure"),
        }
    }
    verdicts
}

fn run_one_op_case(
    op: &str,
    paths: (&RestorableHome, &Path),
    case: MediaCase,
    ctx: (&ReferenceSnapshots, &ExecutionRequest<'_, '_>),
) -> RecoveredPrefix {
    let (home, markers_root) = paths;
    let (role_idx, nth, dir_tag) = case;
    let (reference, execution) = ctx;
    home.restore();
    let ready = markers_root.join("ready");
    let finished = markers_root.join("finished");
    let _ = std::fs::remove_file(&ready);
    let _ = std::fs::remove_file(&finished);
    let spec = PathBuf::from(format!("{op}:{role_idx}:{nth}:{dir_tag}"));
    let end = crate::killed_child::run_and_kill(
        OP_MEDIA_CHILD,
        &[
            (ENV_HOME, home.location()),
            (ENV_MARKERS, markers_root),
            (ENV_SPEC, &spec),
        ],
        &ready,
        &finished,
    );
    assert_eq!(
        end,
        crate::killed_child::ChildEnd::KilledAtPark,
        "{op}:{:?}#{nth}:{dir_tag} must fire media fault and park",
        MediaOperationRole::ALL[role_idx]
    );
    let label = format!("{op}:{:?}#{nth}:{dir_tag}", MediaOperationRole::ALL[role_idx]);
    verify_reopen_and_continue(home.location(), reference, execution, &label)
}

fn measure_op_roles_in_child(saved: &RestorableHome, op: &str) -> Vec<(usize, u64)> {
    saved.restore();
    let markers = tempfile::tempdir().unwrap();
    let spec = PathBuf::from(format!("count:{op}"));
    let end = crate::killed_child::run_and_kill(
        OP_MEDIA_CHILD,
        &[
            (ENV_HOME, saved.location()),
            (ENV_MARKERS, markers.path()),
            (ENV_SPEC, &spec),
        ],
        &markers.path().join("ready"),
        &markers.path().join("finished"),
    );
    assert_eq!(end, crate::killed_child::ChildEnd::Finished, "{op} calibration must finish");
    let raw = std::fs::read_to_string(markers.path().join("finished")).unwrap();
    raw.lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let mut parts = line.split(':');
            (parts.next().unwrap().parse().unwrap(), parts.next().unwrap().parse().unwrap())
        })
        .collect()
}

fn execute_target_op(
    home: &host_observation::OracleHome,
    op: &str,
    must_succeed: bool,
    execution: &ExecutionRequest<'_, '_>,
) {
    match op {
        "checkpoint" => {
            let outcome = home.checkpoint(execution);
            if must_succeed {
                outcome.unwrap_or_else(|e| panic!("checkpoint calibration must succeed: {e:?}"));
            }
        }
        "continue" => {
            let Resolution::ProductUnpublished(effects) =
                requests::resolve_original(home.runtime(), TARGET_KEY, TARGET_VAL).unwrap()
            else {
                panic!("continuation target must start in ProductUnpublished custody");
            };
            let recovery = effects.into_recovery();
            for _ in 0..64 {
                match recovery.continue_original_in_request(execution) {
                    Ok(Continued::Pending(c)) => drop(c),
                    Ok(Continued::Published { .. }) => return,
                    Ok(_) if !must_succeed => return,
                    Err(_) if !must_succeed => return,
                    other => panic!("continuation calibration must publish, got {other:?}"),
                }
            }
            if must_succeed {
                panic!("continuation calibration exceeded 64 Pending rounds");
            }
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
    if let Some(op) = spec.strip_prefix("count:") {
        let counts = calibrate_in_child(&location, op, &execution);
        let body: String = counts.iter().map(|(r, c)| format!("{r}:{c}\n")).collect();
        crate::killed_child::announce(&markers.join("finished"), body.as_bytes());
        return;
    }
    let mut parts = spec.split(':');
    let op = parts.next().unwrap();
    let role_idx: usize = parts.next().unwrap().parse().unwrap();
    let nth: u64 = parts.next().unwrap().parse().unwrap();
    let dir_tag = parts.next().unwrap();
    let role = MediaOperationRole::ALL[role_idx];
    let media =
        FilesystemMediaAdmission::certification(FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = media.fault_schedule_authority();
    let activation = authority.one_shot_activation();
    let rule = authority
        .rule(role, 1, decode_directive(dir_tag))
        .for_nth_identified_operation_after_activation(
            activation.clone(),
            NonZeroU64::new(nth).unwrap(),
        );
    let schedule = authority.schedule(vec![rule]).unwrap();
    let (home, _) = open_dir(
        &location,
        Some(AdversarialPlacement::new().with_media(media.with_fault_schedule(schedule))),
        Arc::new(AtomicUsize::new(0)),
        &execution,
    );
    activation.arm().unwrap();
    execute_target_op(&home, op, false, &execution);
    assert!(activation.is_consumed(), "media fault {spec} did not fire");
    crate::killed_child::announce(&markers.join("ready"), spec.as_bytes());
    crate::killed_child::park();
}

fn calibrate_in_child(
    location: &Path,
    op: &str,
    execution: &ExecutionRequest<'_, '_>,
) -> Vec<(usize, u64)> {
    let media =
        FilesystemMediaAdmission::certification(FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = media.fault_schedule_authority();
    let activations: Vec<_> = MediaOperationRole::ALL
        .iter()
        .map(|_| authority.one_shot_activation())
        .collect();
    let rules = MediaOperationRole::ALL
        .iter()
        .zip(&activations)
        .map(|(role, act)| {
            authority
                .rule(*role, 1, decode_directive("fail_before"))
                .for_nth_identified_operation_after_activation(act.clone(), NonZeroU64::MAX)
        })
        .collect();
    let schedule = authority.schedule(rules).unwrap();
    let (home, _) = open_dir(
        location,
        Some(AdversarialPlacement::new().with_media(media.with_fault_schedule(schedule))),
        Arc::new(AtomicUsize::new(0)),
        execution,
    );
    for act in &activations {
        act.arm().unwrap();
    }
    execute_target_op(&home, op, true, execution);
    let counts: Vec<_> = activations
        .iter()
        .enumerate()
        .map(|(idx, act)| (idx, act.matching_operation_count()))
        .filter(|(_, c)| *c > 0)
        .collect();
    home.close(execution).unwrap().reclaim();
    counts
}
