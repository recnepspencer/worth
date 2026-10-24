//! Real processes: a child opens a home, runs a script prefix and is killed
//! idle on either side of the first checkpoint, or while a commit or a
//! checkpoint is parked at a Store mutation seam. The parent recovers through
//! the production `open` and holds the home to the crash oracle.
//!
//! A commit parked before its WAL append recovers without it. One parked at
//! any later seam recovers with it, although the dead process never
//! installed it: the child observes its own live history while parked, and
//! the commit is not there (durable before live, D2).

use std::fs;
use std::path::{Path, PathBuf};

use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};
use worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint as Seam;

use super::crash_oracle::{assert_continues, assert_recovered, InFlight};
use super::durable_run::{apply, open, steps_before};
use super::killed_child::{self, ChildEnd};
use super::selected_checkpoint;
use super::workload::{self, Script, Step};

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
    /// The checkpoint replacing the first one, parked at a seam.
    Checkpoint(Seam),
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
            .chain(SEAMS.into_iter().map(Self::Checkpoint))
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
            Self::Checkpoint(_) => {
                let target = position(|step| matches!(step, Step::Checkpoint), 1);
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
    Killed {
        location,
        acknowledged,
        live,
        _directory: directory,
    }
}

fn assert_killed_home_recovers(stop: Stop) {
    let script = script();
    let killed = launch(stop);
    let case = format!("seed {SEED:#x} {stop:?}");
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let home = open(&killed.location, &execution)
        .unwrap_or_else(|refusal| panic!("{case}: recovery refused: {refusal:?}"));
    let held = killed.acknowledged.len();
    let (in_flight, checkpoint) = match stop {
        Stop::BeforeFirstCheckpoint => (InFlight::Nothing, 0),
        Stop::AboveCheckpoint => (InFlight::Nothing, 1),
        Stop::Commit(Seam::BeforeWalAppend) => (InFlight::NotDurable, 2),
        Stop::Commit(_) => (InFlight::Durable, 2),
        Stop::Checkpoint(Seam::BeforeWalAppend) => (InFlight::Nothing, 1),
        Stop::Checkpoint(_) => (InFlight::Nothing, 2),
    };
    assert_eq!(killed.live, held, "{case}: the parked commit was live");
    assert_eq!(
        selected_checkpoint::generation(&home),
        checkpoint,
        "{case}: selected checkpoint"
    );
    let commits = assert_recovered(&home, &script, &killed.acknowledged, &in_flight, &case);
    assert_continues(&killed.location, home, &script, commits, &execution, &case);
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
        assert_killed_home_recovers(Stop::Checkpoint(seam));
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

/// The child: run the plan for its stop, park, announce, and wait to die.
fn child(stop: Stop, location: &Path, markers: &Path) -> ! {
    let script = script();
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let home = open(location, &execution).expect("the child's home opens");
    let (completed, parked) = stop.plan(&script);
    apply(&home, &execution, &script.steps[..completed]);
    let acknowledged = workload::artifacts_of(home.runtime());
    killed_child::announce(&markers.join("acknowledged"), &encode(&acknowledged));
    let seam = match stop {
        Stop::Commit(seam) | Stop::Checkpoint(seam) => seam,
        Stop::BeforeFirstCheckpoint | Stop::AboveCheckpoint => {
            let live = acknowledged.len().to_string();
            killed_child::announce(&markers.join("ready"), live.as_bytes());
            killed_child::park();
        }
    };
    let target = &script.steps[parked.expect("a parked step")];
    std::thread::scope(|scope| {
        let gate = home
            .certification_serving()
            .certification_pause_physical_mutation_at(seam);
        scope.spawn(|| {
            let request = workload::serial_request();
            let execution = ExecutionRequest::serial(&request);
            match target {
                Step::Checkpoint => {
                    let _ = home.checkpoint(ExecutionAllocationPolicy::SystemAllocation);
                }
                _ => {
                    let _ = target.commit(home.runtime(), &execution);
                }
            }
        });
        assert!(gate.await_arrival(), "{stop:?}: the target did not park");
        let live = workload::artifacts_of(home.runtime()).len().to_string();
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
