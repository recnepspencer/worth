//! C4b.1: Adversarial Store physical-mutation seam OS-kill sweep for the full
//! Query Host + Store Application Home, verified against MemoryOnly under a
//! single run-wide DurableIdentityCorrespondence.
use super::super::{opening, *};
use super::RestorableHome;
use super::oracle_verifier::{
    build_reference, open_dir, prepare_prefix, verify_reopen_and_continue, TARGET_KEY, TARGET_VAL,
};
use host_interruption::opening::{AdversarialPlacement, PhysicalPauseHandle};
use host_observation::{adversarial_verdict::RecoveredPrefix, requests, OracleHome};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::{atomic::AtomicUsize, Arc};
use worth_execution::ExecutionRequest;
use worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint as Seam;

const SEAM_CHILD: &str =
    "process::application::host_oracle::adversarial::seam_kill_sweep::host_seam_kill_child";
const ENV_HOME: &str = "WORTH_ADV_HOST_HOME";
const ENV_MARKERS: &str = "WORTH_ADV_HOST_MARKERS";
const ENV_SPEC: &str = "WORTH_ADV_HOST_SPEC";

pub(super) const ALL_SEAMS: [(&str, Seam); 5] = [
    ("before_wal", Seam::BeforeWalAppend),
    ("after_wal", Seam::AfterWalDurability),
    ("after_data", Seam::AfterDataSettlement),
    ("during_root", Seam::DuringRootPublication),
    ("after_root", Seam::AfterRootReplacement),
];

#[test]
fn host_mutation_and_checkpoint_killed_at_every_store_seam_recovers_to_memory_oracle() {
    let req = opening::serial();
    let execution = ExecutionRequest::serial(&req);
    let reference = build_reference(&execution);
    let saved = RestorableHome::prepare(|loc| prepare_prefix(loc, &execution));

    let append_count = measure_appends(&saved, "count_mutate");
    assert!(
        append_count >= 3,
        "Host mutation must issue OpenAttempt, Native, and World appends (got {append_count})"
    );

    let mut seen_verdicts = BTreeSet::new();
    for append_ordinal in 1..=append_count {
        for (seam_name, _) in ALL_SEAMS {
            saved.restore();
            kill_at_seam(saved.location(), &format!("mutate:{append_ordinal}:{seam_name}"));
            let label = format!("mutate#{append_ordinal}@{seam_name}");
            let verdict =
                verify_reopen_and_continue(saved.location(), &reference, &execution, &label);
            let tag = match verdict {
                RecoveredPrefix::PreOperation => "pre",
                RecoveredPrefix::UnpublishedCustody { .. } => "custody",
                RecoveredPrefix::CommittedInFlight => "committed",
            };
            seen_verdicts.insert(tag);
        }
    }
    assert_eq!(
        seen_verdicts,
        BTreeSet::from(["pre", "custody", "committed"]),
        "Store seam kill sweep must exercise all three durable outcomes"
    );

    let checkpoint_appends = measure_appends(&saved, "count_checkpoint");
    assert!(checkpoint_appends >= 1, "checkpoint must issue at least one physical append");
    for append_ordinal in 1..=checkpoint_appends {
        for (seam_name, _) in ALL_SEAMS {
            saved.restore();
            kill_at_seam(saved.location(), &format!("checkpoint:{append_ordinal}:{seam_name}"));
            let verdict = verify_reopen_and_continue(
                saved.location(),
                &reference,
                &execution,
                &format!("checkpoint#{append_ordinal}@{seam_name}"),
            );
            assert_eq!(verdict, RecoveredPrefix::PreOperation);
        }
    }

    let close_appends = measure_appends(&saved, "count_close");
    assert!(close_appends >= 1, "dirty close must issue at least one physical append");
    for append_ordinal in 1..=close_appends {
        for (seam_name, _) in ALL_SEAMS {
            saved.restore();
            kill_at_seam(saved.location(), &format!("close:{append_ordinal}:{seam_name}"));
            let verdict = verify_reopen_and_continue(
                saved.location(),
                &reference,
                &execution,
                &format!("close#{append_ordinal}@{seam_name}"),
            );
            assert_eq!(verdict, RecoveredPrefix::CommittedInFlight);
        }
    }
}

pub(super) fn measure_appends(saved: &RestorableHome, spec: &str) -> usize {
    saved.restore();
    let markers = tempfile::tempdir().unwrap();
    let end = crate::killed_child::run_and_kill(
        SEAM_CHILD,
        &[
            (ENV_HOME, saved.location()),
            (ENV_MARKERS, markers.path()),
            (ENV_SPEC, Path::new(spec)),
        ],
        &markers.path().join("ready"),
        &markers.path().join("finished"),
    );
    assert_eq!(end, crate::killed_child::ChildEnd::Finished);
    std::fs::read_to_string(markers.path().join("finished"))
        .unwrap()
        .trim()
        .parse()
        .unwrap()
}

pub(super) fn kill_at_seam(home_dir: &Path, spec: &str) {
    let markers = tempfile::tempdir().unwrap();
    let spec_path = PathBuf::from(spec);
    let end = crate::killed_child::run_and_kill(
        SEAM_CHILD,
        &[
            (ENV_HOME, home_dir),
            (ENV_MARKERS, markers.path()),
            (ENV_SPEC, &spec_path),
        ],
        &markers.path().join("ready"),
        &markers.path().join("finished"),
    );
    assert_eq!(
        end,
        crate::killed_child::ChildEnd::KilledAtPark,
        "spec {spec} must park"
    );
}

#[test]
fn host_seam_kill_child() {
    let Some(location) = std::env::var_os(ENV_HOME).map(PathBuf::from) else {
        return;
    };
    let markers = PathBuf::from(std::env::var_os(ENV_MARKERS).unwrap());
    let spec = std::env::var(ENV_SPEC).unwrap();
    let req = opening::serial();
    let execution = ExecutionRequest::serial(&req);
    let (home, pause) = open_dir(
        &location,
        Some(AdversarialPlacement::new()),
        Arc::new(AtomicUsize::new(0)),
        &execution,
    );
    let pause = pause.unwrap();
    if spec == "count_mutate" || spec == "count_checkpoint" || spec == "count_close" {
        let count = count_op_appends(home, &pause, &spec, &execution);
        crate::killed_child::announce(&markers.join("finished"), count.to_string().as_bytes());
        return;
    }
    let mut parts = spec.split(':');
    let op = parts.next().unwrap();
    let target_ordinal: usize = parts.next().unwrap().parse().unwrap();
    let seam_name = parts.next().unwrap();
    let seam = ALL_SEAMS.iter().find(|(n, _)| *n == seam_name).unwrap().1;
    if op == "close" {
        assert!(requests::set(home.runtime(), TARGET_KEY, TARGET_VAL).receipt().is_some());
    }
    let mut gate = pause.pause_mutation(seam).unwrap();
    let deadline = std::time::Instant::now() + crate::killed_child::PARK_DEADLINE;
    std::thread::scope(|scope| {
        let worker = match op {
            "mutate" => scope.spawn(move || {
                let _ = requests::set(home.runtime(), TARGET_KEY, TARGET_VAL);
                Ok::<(), String>(())
            }),
            "checkpoint" => scope.spawn(move || {
                let req = opening::serial();
                let exec = ExecutionRequest::serial(&req);
                let _ = home.checkpoint(&exec);
                Ok(())
            }),
            "close" => scope.spawn(move || {
                let req = opening::serial();
                let exec = ExecutionRequest::serial(&req);
                let _ = home.close(&exec);
                Ok(())
            }),
            other => panic!("unknown op {other}"),
        };
        for seen in 1..=target_ordinal {
            assert!(
                crate::process_kill::await_mutation_seam(&gate, &worker, deadline),
                "op {op} seam {seam_name} did not reach ordinal {seen}"
            );
            if seen < target_ordinal {
                let next = pause.pause_mutation(seam).unwrap();
                gate.release();
                gate = next;
            }
        }
        crate::killed_child::announce(&markers.join("ready"), spec.as_bytes());
        crate::killed_child::park();
    });
}

fn count_op_appends(
    home: OracleHome,
    pause: &PhysicalPauseHandle,
    spec: &str,
    execution: &ExecutionRequest<'_, '_>,
) -> usize {
    if spec == "count_close" {
        assert!(requests::set(home.runtime(), TARGET_KEY, TARGET_VAL).receipt().is_some());
    }
    let mut gate = pause.pause_mutation(Seam::BeforeWalAppend).unwrap();
    let deadline = std::time::Instant::now() + crate::killed_child::PARK_DEADLINE;
    let mut count = 0;
    let remaining = std::sync::Mutex::new(None);
    std::thread::scope(|scope| {
        let remaining_ref = &remaining;
        let worker = scope.spawn(move || {
            let req = opening::serial();
            let exec = ExecutionRequest::serial(&req);
            match spec {
                "count_checkpoint" => {
                    home.checkpoint(&exec).unwrap();
                    *remaining_ref.lock().unwrap() = Some(home);
                }
                "count_mutate" => {
                    assert!(requests::set(home.runtime(), TARGET_KEY, TARGET_VAL)
                        .receipt()
                        .is_some());
                    *remaining_ref.lock().unwrap() = Some(home);
                }
                "count_close" => home.close(&exec).unwrap().reclaim(),
                other => panic!("unknown count spec {other}"),
            }
            Ok::<(), String>(())
        });
        while crate::process_kill::await_mutation_seam(&gate, &worker, deadline) {
            count += 1;
            let next = pause.pause_mutation(Seam::BeforeWalAppend).unwrap();
            gate.release();
            gate = next;
        }
        gate.release();
        worker.join().unwrap().unwrap();
    });
    if let Some(home) = remaining.into_inner().unwrap() {
        home.close(execution).unwrap().reclaim();
    }
    count
}
