//! Real processes: a child opens a home, runs a script prefix and is killed
//! idle on either side of the first checkpoint, or while a commit, branch
//! transition, or checkpoint is parked at a Store mutation seam. The parent
//! recovers through production `open` and holds the home to the crash oracle.
//!
//! A commit parked before its WAL append recovers without it. One parked at
//! any later seam recovers with it, although the dead process never
//! installed it: the child observes its own live history while parked, and
//! the commit is not there (durable before live, D2).

use std::fs;
use std::num::NonZeroU64;
use std::path::PathBuf;
use std::time::Instant;

use worth_execution::ExecutionRequest;
use worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint as Seam;

use super::durable_run::{apply, open, steps_before};
use super::killed_child::{self, ChildEnd};
use super::workload::{self, Script, Step};

#[path = "process_kill/child.rs"]
mod child;
#[path = "process_kill/recovery_court.rs"]
mod recovery_court;
use recovery_court::assert_killed_home_recovers;

pub(crate) const SEED: u64 = 0x5eed_0402;
const COMMITS: usize = 9;
const CHECKPOINT_EVERY: usize = 3;
const ROLE_ENV: &str = "WORTH_RELATIONAL_HOME_CHILD_ROLE";
const LOCATION_ENV: &str = "WORTH_RELATIONAL_HOME_CHILD_LOCATION";
const MARKERS_ENV: &str = "WORTH_RELATIONAL_HOME_CHILD_MARKERS";
const CHILD_TEST: &str = "process_kill::child::relational_home_child";
pub(crate) const WAL_SEGMENT_BYTES_ENV: &str = "WORTH_RELATIONAL_HOME_WAL_SEGMENT_BYTES";

pub(crate) fn script() -> Script {
    Script::generate(SEED, COMMITS, CHECKPOINT_EVERY)
}

/// Where the child stopped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Stop {
    /// Idle after two commits, before any checkpoint.
    BeforeFirstCheckpoint,
    /// Idle after five commits, two above the first checkpoint.
    AboveCheckpoint,
    /// The schema-transition commit parked at a seam.
    Commit(Seam),
    BranchTransition(Seam),
    Archive(Seam),
    RequestDeletion(Seam),
    Delete(Seam),
    /// The checkpoint replacing the first one, parked at a seam.
    Checkpoint(Seam),
    /// An unanchored member batch, before the manifest selection.
    CheckpointMembers(Seam),
    /// Repeat a completed capture: all members are Carried.
    CheckpointCarried(Seam),
    /// Publication has acknowledged its locators before physical checkpointing.
    CheckpointAcknowledged(worth_store::physical_runtime::production::PhysicalCheckpointStep),
    /// Idle after the acknowledged checkpoint returns.
    CheckpointReturned,
}

pub(super) const SEAMS: [Seam; 5] = [
    Seam::BeforeWalAppend,
    Seam::AfterWalDurability,
    Seam::AfterDataSettlement,
    Seam::DuringRootPublication,
    Seam::AfterRootReplacement,
];

impl Stop {
    fn label(self) -> String {
        format!("{self:?}")
    }

    fn all() -> impl Iterator<Item = Self> {
        [Self::BeforeFirstCheckpoint, Self::AboveCheckpoint]
            .into_iter()
            .chain(SEAMS.into_iter().map(Self::Commit))
            .chain(SEAMS.into_iter().map(Self::BranchTransition))
            .chain(SEAMS.into_iter().map(Self::Archive))
            .chain(SEAMS.into_iter().map(Self::RequestDeletion))
            .chain(SEAMS.into_iter().map(Self::Delete))
            .chain(SEAMS.into_iter().map(Self::Checkpoint))
            .chain(SEAMS.into_iter().map(Self::CheckpointMembers))
            .chain(SEAMS.into_iter().map(Self::CheckpointCarried))
            .chain(PHYSICAL_STEPS.into_iter().map(Self::CheckpointAcknowledged))
            .chain([Self::CheckpointReturned])
    }

    /// The script steps the child completes before it parks, and the step it
    /// parks, if any.
    fn plan(self, script: &Script) -> (usize, Option<usize>) {
        let position = |wanted: fn(&Step) -> bool, nth: usize| {
            script
                .steps
                .iter()
                .enumerate()
                .filter(|(_, step)| wanted(step))
                .nth(nth)
                .expect("the script has the step")
                .0
        };
        match self {
            Self::BeforeFirstCheckpoint => (steps_before(script, 2).len(), None),
            Self::AboveCheckpoint => (steps_before(script, 5).len(), None),
            Self::Commit(_) => {
                let target = position(|step| matches!(step, Step::Transition { .. }), 0);
                (target, Some(target))
            }
            Self::Checkpoint(_)
            | Self::CheckpointMembers(_)
            | Self::CheckpointCarried(_)
            | Self::CheckpointAcknowledged(_)
            | Self::CheckpointReturned => {
                let target = position(|step| matches!(step, Step::Checkpoint), 1);
                (target, Some(target))
            }
            Self::BranchTransition(_)
            | Self::Archive(_)
            | Self::RequestDeletion(_)
            | Self::Delete(_) => {
                let select: fn(&Step) -> bool = match self {
                    Self::Archive(_) => |step| matches!(step, Step::Archive { .. }),
                    Self::RequestDeletion(_) => |step| matches!(step, Step::RequestDeletion { .. }),
                    Self::Delete(_) => |step| matches!(step, Step::Delete { .. }),
                    _ => |step| matches!(step, Step::Fork { .. }),
                };
                let target = position(select, 0);
                (target, Some(target))
            }
        }
    }
}

/// What a killed child left behind.
pub(crate) struct Killed {
    pub(crate) location: PathBuf,
    /// The artifacts the child had acknowledged when it parked.
    pub(crate) acknowledged: Vec<Vec<u8>>,
    /// How many commits the child's live runtime held while parked.
    pub(crate) live: usize,
    transitions: Vec<Vec<u8>>,
    transition_live: usize,
    _directory: tempfile::TempDir,
}

/// Run a child to `stop` and kill it there.
pub(crate) fn launch(stop: Stop) -> Killed {
    launch_with_wal_segment_bytes(stop, None)
}

pub(crate) fn child_wal_profile(
) -> Option<worth_store_relational::RelationalStoreCertificationProfile> {
    let bytes = std::env::var_os(WAL_SEGMENT_BYTES_ENV)?;
    let bytes = bytes.to_str().unwrap().parse::<NonZeroU64>().unwrap();
    Some(worth_store_relational::RelationalStoreCertificationProfile::with_wal_segment_bytes(bytes))
}

pub(crate) fn launch_with_wal_segment_bytes(stop: Stop, bytes: Option<NonZeroU64>) -> Killed {
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let markers = directory.path().join("markers");
    fs::create_dir_all(&markers).unwrap();
    let role = PathBuf::from(stop.label());
    let ready = markers.join("ready");
    let bytes = bytes.map(|bytes| PathBuf::from(bytes.get().to_string()));
    let mut environment = vec![
        (ROLE_ENV, &role),
        (LOCATION_ENV, &location),
        (MARKERS_ENV, &markers),
    ];
    if let Some(bytes) = &bytes {
        environment.push((WAL_SEGMENT_BYTES_ENV, bytes));
    }
    let end =
        killed_child::run_and_kill(CHILD_TEST, &environment, &ready, &markers.join("finished"));
    assert_eq!(end, ChildEnd::KilledAtPark, "{stop:?}");
    let live = fs::read_to_string(&ready).unwrap().parse().unwrap();
    let acknowledged = decode(&fs::read(markers.join("acknowledged")).unwrap());
    let transitions = decode(&fs::read(markers.join("transitions")).unwrap());
    let transition_live = fs::read_to_string(markers.join("transition-live"))
        .unwrap()
        .parse()
        .unwrap();
    Killed {
        location,
        acknowledged,
        live,
        transitions,
        transition_live,
        _directory: directory,
    }
}

#[test]
fn a_home_killed_on_either_side_of_its_first_checkpoint_recovers() {
    assert_killed_home_recovers(Stop::BeforeFirstCheckpoint);
    assert_killed_home_recovers(Stop::AboveCheckpoint);
}

#[test]
fn a_commit_killed_at_each_mutation_seam_recovers_durable_before_live() {
    for seam in SEAMS {
        assert_killed_home_recovers(Stop::Commit(seam));
    }
}

#[test]
fn a_checkpoint_killed_at_each_mutation_seam_keeps_one_selected() {
    for seam in SEAMS {
        assert_killed_home_recovers(Stop::CheckpointMembers(seam));
        assert_killed_home_recovers(Stop::Checkpoint(seam));
    }
}

#[test]
fn a_branch_transition_killed_at_each_mutation_seam_recovers() {
    for seam in SEAMS {
        for stop in [
            Stop::BranchTransition(seam),
            Stop::Archive(seam),
            Stop::RequestDeletion(seam),
            Stop::Delete(seam),
        ] {
            assert_killed_home_recovers(stop);
        }
    }
}

pub(super) fn encode(artifacts: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for artifact in artifacts {
        bytes.extend_from_slice(&(artifact.len() as u64).to_le_bytes());
        bytes.extend_from_slice(artifact);
    }
    bytes
}

fn decode(mut bytes: &[u8]) -> Vec<Vec<u8>> {
    let mut artifacts = Vec::new();
    while let Some((length, rest)) = bytes.split_first_chunk::<8>() {
        let length = u64::from_le_bytes(*length) as usize;
        artifacts.push(rest[..length].to_vec());
        bytes = &rest[length..];
    }
    artifacts
}

/// A Store gate's five-second wait is one poll, not the process's deadline.
/// Completion without arrival still fails; a running operation keeps its gate
/// armed within the original, finite crash-driver budget.
pub(super) fn await_mutation_seam(
    gate: &worth_store::physical_runtime::certification::CertificationPhysicalMutationPauseGate,
    worker: &std::thread::ScopedJoinHandle<'_, Result<(), String>>,
    deadline: Instant,
) -> bool {
    loop {
        if Instant::now() >= deadline {
            return false;
        }
        if gate.await_arrival() {
            return Instant::now() < deadline;
        }
        if worker.is_finished() {
            return false;
        }
    }
}

#[test]
fn a_late_mutation_seam_is_not_mistaken_for_a_finished_child() {
    let deadline = Instant::now() + killed_child::PARK_DEADLINE;
    let directory = tempfile::tempdir().unwrap();
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let home = open(&directory.path().join("home"), &execution).unwrap();
    let script = script();
    let (completed, parked) = Stop::Commit(Seam::AfterRootReplacement).plan(&script);
    apply(&home, &execution, &script.steps[..completed]);
    std::thread::scope(|scope| {
        let gate = home
            .certification_serving()
            .certification_pause_physical_mutation_at(Seam::AfterRootReplacement);
        let worker = scope.spawn(|| {
            // Force an actual arrival beyond the gate's first polling window.
            std::thread::sleep(std::time::Duration::from_secs(6));
            let request = workload::serial_request();
            let execution = ExecutionRequest::serial(&request);
            script.steps[parked.unwrap()]
                .commit(home.runtime(), &execution)
                .map(|_| ())
                .map_err(|stop| format!("{stop:?}"))
        });
        let arrived = await_mutation_seam(&gate, &worker, deadline);
        gate.release();
        let result = worker.join().unwrap();
        assert!(arrived, "late Store seam was missed: {result:?}");
        result.unwrap();
    });
}

use worth_store::physical_runtime::production::PhysicalCheckpointStep;
// Home publications settle their data before this fuzzy capture, so no dirty
// frame is appended. Tier certificates belong to Store's certified-tier route,
// which the home does not use. These are its seven reachable checkpoint cuts.
const PHYSICAL_STEPS: [PhysicalCheckpointStep; 7] = [
    PhysicalCheckpointStep::CandidateCreation,
    PhysicalCheckpointStep::CandidateBindingCompactionHeader,
    PhysicalCheckpointStep::CandidateBindingRecord,
    PhysicalCheckpointStep::CandidateFooter,
    PhysicalCheckpointStep::CandidateSynchronization,
    PhysicalCheckpointStep::CandidatePublication,
    PhysicalCheckpointStep::NamespaceSynchronization,
];

#[test]
fn carried_members_survive_a_kill_at_each_manifest_mutation_seam() {
    for seam in SEAMS {
        assert_killed_home_recovers(Stop::CheckpointCarried(seam));
    }
}

#[test]
fn acknowledged_locators_survive_each_physical_checkpoint_seam_and_return() {
    for step in PHYSICAL_STEPS {
        assert_killed_home_recovers(Stop::CheckpointAcknowledged(step));
    }
    assert_killed_home_recovers(Stop::CheckpointReturned);
}
