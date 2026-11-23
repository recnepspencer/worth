//! Full adversarial court oracle (M3 C4b): real OS kills at durable I/O
//! boundaries and media faults across every supported owner family, verified
//! against MemoryOnly under a single run-wide DurableIdentityCorrespondence.
mod adoption_io_kills;
mod correction_fixture;
mod correction_io_kills;
mod dimensions_matrix;
mod dimensions_signal;
mod dispatch_io_kills;
mod media_fault_ops;
mod media_fault_sweep;
mod oracle_verifier;
mod parallel;
mod recovery_fixtures;
mod recovery_interruption_oracle;
mod seam_kill_sweep;
mod signal_bridge_io_kills;
mod workflow_correction_fixture;
mod workflow_correction_io_kills;

use self::parallel::{run_parallel_barriers, run_parallel_calibrated_barriers, run_parallel_items};

use std::path::Path;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;
use worth_foundational::facade::ApplicationBuildIdentity;
use worth_store::physical_runtime::{
    certification::{
        CertificationMediaFaultActivation, MediaFaultDirective, MediaOperationRole,
    },
    FilesystemAccessPosture, FilesystemMediaAdmission,
};
use worth_store_testkit_application_oracle::facade::host_interruption::opening::AdversarialPlacement;

pub(super) const BUILD_PRIMARY: &str = "11111111-1111-4111-8111-111111111111";
pub(super) const BUILD_SECONDARY: &str = "22222222-2222-4222-8222-222222222222";
pub(super) const BUILD_TERTIARY: &str = "33333333-3333-4333-8333-333333333333";

pub(super) fn build_id(raw: &str) -> ApplicationBuildIdentity {
    ApplicationBuildIdentity::parse(raw).unwrap()
}

pub(super) struct RestorableHome {
    _root: tempfile::TempDir,
    location: std::path::PathBuf,
    image: crate::media_sweep::prepared_sweep_home::DirectoryImage,
}

impl RestorableHome {
    pub(super) fn prepare(build: impl FnOnce(&Path)) -> Self {
        let root = tempfile::tempdir().unwrap();
        let location = root.path().join("home");
        build(&location);
        let image = crate::media_sweep::prepared_sweep_home::DirectoryImage::read(&location);
        Self {
            _root: root,
            location,
            image,
        }
    }

    pub(super) fn location(&self) -> &Path {
        &self.location
    }

    /// Restore saved bytes in place under the original root directory so
    /// Store's WAL filesystem-ID binding stays valid across cases.
    pub(super) fn restore(&self) {
        for entry in std::fs::read_dir(&self.location).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                std::fs::remove_dir_all(entry.path()).unwrap();
            } else {
                std::fs::remove_file(entry.path()).unwrap();
            }
        }
        self.image.write(&self.location);
    }
}

/// Build a certification media admission that counts durable barrier calls
/// (when `nth == u64::MAX`) or pauses after the `nth` call for `role` after arming.
pub(super) fn role_barrier_placement(
    role: MediaOperationRole,
    nth: u64,
    ready_marker: Option<std::path::PathBuf>,
    done: Option<Arc<AtomicBool>>,
) -> (
    AdversarialPlacement,
    CertificationMediaFaultActivation,
    Option<std::thread::JoinHandle<()>>,
) {
    let media =
        FilesystemMediaAdmission::certification(FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = media.fault_schedule_authority();
    let activation = authority.one_shot_activation();
    let (schedule, watcher) = if let (Some(ready), Some(done)) = (ready_marker, done) {
        let gate = authority.pause_gate();
        let rule = authority
            .rule(role, 1, MediaFaultDirective::PauseAfter(gate.clone()))
            .for_nth_identified_operation_after_activation(
                activation.clone(),
                std::num::NonZeroU64::new(nth).unwrap(),
            );
        let schedule = authority.schedule(vec![rule]).unwrap();
        let marker = barrier_marker(role, nth);
        let watcher = std::thread::spawn(move || {
            while !done.load(Ordering::SeqCst) {
                if gate.reached_context().is_some() {
                    crate::killed_child::announce(&ready, marker.as_bytes());
                    crate::killed_child::park();
                }
                std::thread::sleep(Duration::from_millis(1));
            }
        });
        (schedule, Some(watcher))
    } else {
        let rule = authority
            .rule(
                role,
                1,
                MediaFaultDirective::FailBefore {
                    kind: std::io::ErrorKind::Other,
                    raw_os_error: None,
                },
            )
            .for_nth_identified_operation_after_activation(
                activation.clone(),
                std::num::NonZeroU64::MAX,
            );
        (authority.schedule(vec![rule]).unwrap(), None)
    };
    (
        AdversarialPlacement::new().with_media(media.with_fault_schedule(schedule)),
        activation,
        watcher,
    )
}

pub(super) fn sync_barrier_placement(
    nth: u64,
    ready_marker: Option<std::path::PathBuf>,
    done: Option<Arc<AtomicBool>>,
) -> (
    AdversarialPlacement,
    CertificationMediaFaultActivation,
    Option<std::thread::JoinHandle<()>>,
) {
    role_barrier_placement(MediaOperationRole::SynchronizeFileState, nth, ready_marker, done)
}

pub(super) fn barrier_marker(role: MediaOperationRole, nth: u64) -> String {
    format!("{role:?}#{nth}")
}

pub(super) const TAIL_VARIANCE_BOUND: u64 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BarrierKill {
    Killed,
    CompletedAtTail { actual: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ReopenPhase {
    PreOp,
    Custody,
    Committed,
}

pub(super) fn assert_all_reopen_phases_killed(
    label: &str,
    outcomes: &[(u64, BarrierKill, ReopenPhase)],
) {
    let pre = outcomes.iter().any(|(_, k, p)| *k == BarrierKill::Killed && *p == ReopenPhase::PreOp);
    let custody = outcomes.iter().any(|(_, k, p)| *k == BarrierKill::Killed && *p == ReopenPhase::Custody);
    let committed = outcomes.iter().any(|(_, k, p)| *k == BarrierKill::Killed && *p == ReopenPhase::Committed);
    assert!(
        pre && custody && committed,
        "{label}: sweep must genuinely kill pre-op ({pre}), custody ({custody}), and committed ({committed}) reopen states"
    );
}

/// Assert that a barrier kill genuinely parked on the exact requested media
/// gate marker (never accepting a silent fallback completion as a barrier kill).
pub(super) fn assert_barrier_killed(
    end: crate::killed_child::ChildEnd,
    ready_marker: &Path,
    role: MediaOperationRole,
    nth: u64,
    context: &str,
) {
    let finished_note = ready_marker
        .parent()
        .and_then(|dir| std::fs::read_to_string(dir.join("finished")).ok())
        .unwrap_or_default();
    assert_eq!(
        end,
        crate::killed_child::ChildEnd::KilledAtPark,
        "{context}: barrier {role:?}#{nth} must park at media gate (finished={finished_note:?})"
    );
    let actual = std::fs::read_to_string(ready_marker).unwrap_or_default();
    let expected = barrier_marker(role, nth);
    assert_eq!(
        actual, expected,
        "{context}: expected exact barrier marker {expected}, got {actual:?}"
    );
}

/// Classify a barrier run across a multi-owner operation whose exact tail sync
/// count can vary by at most `TAIL_VARIANCE_BOUND` across OS processes.
/// Any early miss or unmarked completion panics immediately.
pub(super) fn classify_barrier_kill(
    end: crate::killed_child::ChildEnd,
    ready_marker: &Path,
    role: MediaOperationRole,
    nth: u64,
    total: u64,
    context: &str,
) -> BarrierKill {
    if end == crate::killed_child::ChildEnd::KilledAtPark {
        assert_barrier_killed(end, ready_marker, role, nth, context);
        return BarrierKill::Killed;
    }
    let finished_note = ready_marker
        .parent()
        .and_then(|dir| std::fs::read_to_string(dir.join("finished")).ok())
        .unwrap_or_default();
    let actual: u64 = finished_note
        .trim()
        .strip_prefix("missed:actual=")
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| {
            panic!("{context}: barrier {role:?}#{nth} finished without missed:actual=<N> (finished={finished_note:?})")
        });
    assert!(
        actual < nth && actual >= total.saturating_sub(TAIL_VARIANCE_BOUND),
        "{context}: barrier {role:?}#{nth} missed unexpectedly (actual={actual}, total={total}, bound={TAIL_VARIANCE_BOUND})"
    );
    BarrierKill::CompletedAtTail { actual }
}

/// Exercise every measured durable barrier ordinal `1..=total` without sampling.
pub(super) fn all_barriers(total: u64) -> std::ops::RangeInclusive<u64> {
    assert!(total > 0, "target operation must execute durable syncs");
    1..=total
}
