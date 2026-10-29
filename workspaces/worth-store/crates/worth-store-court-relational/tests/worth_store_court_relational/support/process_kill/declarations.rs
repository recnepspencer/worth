//! Real processes: a child opens a home, runs a script prefix and is killed
//! idle on either side of the first checkpoint, or while a commit, branch
//! transition, or checkpoint is parked at a Store mutation seam. The parent
//! recovers through production `open` and holds the home to the crash oracle.
//!
//! A commit parked before its WAL append recovers without it. One parked at
//! any later seam recovers with it, although the dead process never
//! installed it: the child observes its own live history while parked, and
//! the commit is not there (durable before live, D2).

pub(crate) use {std::fs};
pub(crate) use {std::num::NonZeroU64};
pub(crate) use {std::path::PathBuf};
pub(crate) use {std::time::Instant};

pub(crate) use {worth_execution::ExecutionRequest};
pub(crate) use {worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint as Seam};

pub(crate) use {crate::durable_run::apply, crate::durable_run::open, crate::durable_run::steps_before};
pub(crate) use {crate::killed_child, crate::killed_child::ChildEnd};
pub(crate) use {crate::workload, crate::workload::Script, crate::workload::Step};



pub(crate) use {crate::process_kill::recovery_court::assert_killed_home_recovers};

pub(crate) const SEED: u64 = 0x5eed_0402;
pub(crate) const COMMITS: usize = 9;
pub(crate) const CHECKPOINT_EVERY: usize = 3;
pub(crate) const ROLE_ENV: &str = "WORTH_RELATIONAL_HOME_CHILD_ROLE";
pub(crate) const LOCATION_ENV: &str = "WORTH_RELATIONAL_HOME_CHILD_LOCATION";
pub(crate) const MARKERS_ENV: &str = "WORTH_RELATIONAL_HOME_CHILD_MARKERS";
pub(crate) const CHILD_TEST: &str = "process_kill::child::relational_home_child";
pub(crate) const WAL_SEGMENT_BYTES_ENV: &str = "WORTH_RELATIONAL_HOME_WAL_SEGMENT_BYTES";

pub(crate) fn script() -> Script {
    crate::workload::Script::generate(SEED, COMMITS, CHECKPOINT_EVERY)
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

pub(crate) const SEAMS: [Seam; 5] = [
    worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::BeforeWalAppend,
    worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterWalDurability,
    worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterDataSettlement,
    worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::DuringRootPublication,
    worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterRootReplacement,
];

impl Stop {
    pub(crate) fn label(self) -> String {
        format!("{self:?}")
    }

    pub(crate) fn all() -> impl Iterator<Item = Self> {
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
    pub(crate) fn plan(self, script: &Script) -> (usize, Option<usize>) {
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
                let target = position(|step| matches!(step, crate::workload::Step::Transition { .. }), 0);
                (target, Some(target))
            }
            Self::Checkpoint(_)
            | Self::CheckpointMembers(_)
            | Self::CheckpointCarried(_)
            | Self::CheckpointAcknowledged(_)
            | Self::CheckpointReturned => {
                let target = position(|step| matches!(step, crate::workload::Step::Checkpoint), 1);
                (target, Some(target))
            }
            Self::BranchTransition(_)
            | Self::Archive(_)
            | Self::RequestDeletion(_)
            | Self::Delete(_) => {
                let select: fn(&Step) -> bool = match self {
                    Self::Archive(_) => |step| matches!(step, crate::workload::Step::Archive { .. }),
                    Self::RequestDeletion(_) => |step| matches!(step, crate::workload::Step::RequestDeletion { .. }),
                    Self::Delete(_) => |step| matches!(step, crate::workload::Step::Delete { .. }),
                    _ => |step| matches!(step, crate::workload::Step::Fork { .. }),
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
    pub(crate) transitions: Vec<Vec<u8>>,
    pub(crate) transition_live: usize,
    pub(crate) _directory: tempfile::TempDir,
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
    std::fs::create_dir_all(&markers).unwrap();
    let role = std::path::PathBuf::from(stop.label());
    let ready = markers.join("ready");
    let bytes = bytes.map(|bytes| std::path::PathBuf::from(bytes.get().to_string()));
    let mut environment: Vec<(&str, &std::path::Path)> = vec![
        (ROLE_ENV, &role),
        (LOCATION_ENV, &location),
        (MARKERS_ENV, &markers),
    ];
    if let Some(bytes) = &bytes {
        environment.push((WAL_SEGMENT_BYTES_ENV, bytes));
    }
    let end =
        crate::killed_child::run_and_kill(CHILD_TEST, &environment, &ready, &markers.join("finished"));
    assert_eq!(end, crate::killed_child::ChildEnd::KilledAtPark, "{stop:?}");
    let live = std::fs::read_to_string(&ready).unwrap().parse().unwrap();
    let acknowledged = decode(&std::fs::read(markers.join("acknowledged")).unwrap());
    let transitions = decode(&std::fs::read(markers.join("transitions")).unwrap());
    let transition_live = std::fs::read_to_string(markers.join("transition-live"))
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

pub(crate) fn encode(artifacts: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for artifact in artifacts {
        bytes.extend_from_slice(&(artifact.len() as u64).to_le_bytes());
        bytes.extend_from_slice(artifact);
    }
    bytes
}

pub(crate) fn decode(mut bytes: &[u8]) -> Vec<Vec<u8>> {
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
pub(crate) fn await_mutation_seam(
    gate: &worth_store::physical_runtime::certification::CertificationPhysicalMutationPauseGate,
    worker: &std::thread::ScopedJoinHandle<'_, Result<(), String>>,
    deadline: Instant,
) -> bool {
    loop {
        if std::time::Instant::now() >= deadline {
            return false;
        }
        if gate.await_arrival() {
            return std::time::Instant::now() < deadline;
        }
        if worker.is_finished() {
            return false;
        }
    }
}

pub(crate) use {worth_store::physical_runtime::production::PhysicalCheckpointStep};
// Home publications settle their data before this fuzzy capture, so no dirty
// frame is appended. Tier certificates belong to Store's certified-tier route,
// which the home does not use. These are its seven reachable checkpoint cuts.
pub(crate) const PHYSICAL_STEPS: [PhysicalCheckpointStep; 7] = [
    worth_store::physical_runtime::production::PhysicalCheckpointStep::CandidateCreation,
    worth_store::physical_runtime::production::PhysicalCheckpointStep::CandidateBindingCompactionHeader,
    worth_store::physical_runtime::production::PhysicalCheckpointStep::CandidateBindingRecord,
    worth_store::physical_runtime::production::PhysicalCheckpointStep::CandidateFooter,
    worth_store::physical_runtime::production::PhysicalCheckpointStep::CandidateSynchronization,
    worth_store::physical_runtime::production::PhysicalCheckpointStep::CandidatePublication,
    worth_store::physical_runtime::production::PhysicalCheckpointStep::NamespaceSynchronization,
];