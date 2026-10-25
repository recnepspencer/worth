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
use std::path::{Path, PathBuf};
use std::time::Instant;

use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};
use worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint as Seam;

use super::durable_run::{apply, open, steps_before};
use super::killed_child::{self, ChildEnd};
use super::workload::{self, Script, Step};

#[path = "process_kill/recovery_court.rs"]
mod recovery_court;
use recovery_court::assert_killed_home_recovers;

pub(crate) const SEED: u64 = 0x5eed_0402;
const COMMITS: usize = 9;
const CHECKPOINT_EVERY: usize = 3;
const ROLE_ENV: &str = "WORTH_RELATIONAL_HOME_CHILD_ROLE";
const LOCATION_ENV: &str = "WORTH_RELATIONAL_HOME_CHILD_LOCATION";
const MARKERS_ENV: &str = "WORTH_RELATIONAL_HOME_CHILD_MARKERS";
const CHILD_TEST: &str = "process_kill::relational_home_child";

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
}

const SEAMS: [Seam; 5] = [
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
            Self::Checkpoint(_) | Self::CheckpointMembers(_) => {
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
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let markers = directory.path().join("markers");
    fs::create_dir_all(&markers).unwrap();
    let role = PathBuf::from(stop.label());
    let ready = markers.join("ready");
    let end = killed_child::run_and_kill(
        CHILD_TEST,
        &[
            (ROLE_ENV, &role),
            (LOCATION_ENV, &location),
            (MARKERS_ENV, &markers),
        ],
        &ready,
        &markers.join("finished"),
    );
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

fn encode(artifacts: &[Vec<u8>]) -> Vec<u8> {
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
fn await_mutation_seam(
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

/// The child: run the plan for its stop, park, announce, and wait to die.
fn child(stop: Stop, location: &Path, markers: &Path) -> ! {
    let deadline = Instant::now() + killed_child::PARK_DEADLINE;
    let script = script();
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let home = open(location, &execution).expect("the child's home opens");
    let (completed, parked) = stop.plan(&script);
    apply(&home, &execution, &script.steps[..completed]);
    let acknowledged = workload::artifacts_of(home.runtime());
    killed_child::announce(&markers.join("acknowledged"), &encode(&acknowledged));
    let transitions = super::branch_lifecycle::artifacts_of_home(&home);
    killed_child::announce(&markers.join("transitions"), &encode(&transitions));
    let seam = match stop {
        Stop::Commit(seam)
        | Stop::BranchTransition(seam)
        | Stop::Archive(seam)
        | Stop::RequestDeletion(seam)
        | Stop::Delete(seam)
        | Stop::Checkpoint(seam)
        | Stop::CheckpointMembers(seam) => seam,
        Stop::BeforeFirstCheckpoint | Stop::AboveCheckpoint => {
            let live = acknowledged.len().to_string();
            killed_child::announce(
                &markers.join("transition-live"),
                transitions.len().to_string().as_bytes(),
            );
            killed_child::announce(&markers.join("ready"), live.as_bytes());
            killed_child::park();
        }
    };
    let target = &script.steps[parked.expect("a parked step")];
    std::thread::scope(|scope| {
        let mut gate = home
            .certification_serving()
            .certification_pause_physical_mutation_at(seam);
        let worker = scope.spawn(|| {
            let request = workload::serial_request();
            let execution = ExecutionRequest::serial(&request);
            match target {
                Step::Checkpoint => home
                    .checkpoint(ExecutionAllocationPolicy::SystemAllocation)
                    .map(|_| ())
                    .map_err(|stop| format!("{stop:?}")),
                step if step.is_lifecycle() => step
                    .lifecycle(home.runtime(), &execution)
                    .map_err(|stop| format!("{stop:?}")),
                _ => target
                    .commit(home.runtime(), &execution)
                    .map(|_| ())
                    .map_err(|stop| format!("{stop:?}")),
            }
        });
        if !await_mutation_seam(&gate, &worker, deadline) {
            // A late arrival must not deadlock scope teardown after a failed
            // assertion. Preserve the actual attempt's result in the failure.
            gate.release();
            panic!("{stop:?}: target did not park: {:?}", worker.join());
        }
        if matches!(stop, Stop::Checkpoint(_)) {
            // This small fixture's new members form one batch. Its durable
            // root does not select the checkpoint; the following anchored
            // append is the decision the existing checkpoint court judges.
            let anchor = home
                .certification_serving()
                .certification_pause_physical_mutation_at(seam);
            gate.release();
            gate = anchor;
            if !await_mutation_seam(&gate, &worker, deadline) {
                gate.release();
                panic!("{stop:?}: anchor did not park: {:?}", worker.join());
            }
        }
        if let Step::Fork { target } = target {
            assert!(
                home.runtime().branch_identity(target).is_err(),
                "{stop:?}: fork installed before durable publication returned"
            );
        }
        // The Store seam may hold its publication locks. Inspect the live
        // catalog here; exporting old artifacts can perform cold Store reads.
        let live = home
            .runtime()
            .history()
            .immutable_commit_count()
            .to_string();
        killed_child::announce(
            &markers.join("transition-live"),
            super::branch_lifecycle::live_count(home.runtime())
                .to_string()
                .as_bytes(),
        );
        killed_child::announce(&markers.join("ready"), live.as_bytes());
        killed_child::park();
    })
}

#[test]
fn relational_home_child() {
    let Some(role) = std::env::var_os(ROLE_ENV) else {
        return;
    };
    let stop = Stop::all()
        .find(|stop| Some(stop.label().as_str()) == role.to_str())
        .expect("a known child role");
    let location = PathBuf::from(std::env::var_os(LOCATION_ENV).expect("child location"));
    let markers = PathBuf::from(std::env::var_os(MARKERS_ENV).expect("child markers"));
    child(stop, &location, &markers);
}
