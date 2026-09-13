use std::path::{Path, PathBuf};

use super::super::artifact_observation::observe_file;
use super::super::{
    RecoveryObserverCounters, RecoveryObserverLimits, RecoveryObserverObservationFailure,
};
use super::entry_admission::AdmittedEntry;
use super::ObservedRecoveryArtifact;

pub(super) struct WalkState {
    pending: Vec<PathBuf>,
    artifacts: Vec<ObservedRecoveryArtifact>,
    counters: RecoveryObserverCounters,
}

impl WalkState {
    pub(super) fn new(root: PathBuf) -> Self {
        Self {
            pending: vec![root],
            artifacts: Vec::new(),
            counters: RecoveryObserverCounters::with_root_admitted(),
        }
    }

    pub(super) fn next_directory(&mut self) -> Option<PathBuf> {
        self.pending.pop()
    }

    pub(super) fn counters(&self) -> RecoveryObserverCounters {
        self.counters
    }

    pub(super) fn counters_mut(&mut self) -> &mut RecoveryObserverCounters {
        &mut self.counters
    }

    pub(super) fn apply(
        &mut self,
        root: &Path,
        entry: AdmittedEntry,
        limits: RecoveryObserverLimits,
    ) -> Result<(), RecoveryObserverObservationFailure> {
        match entry {
            AdmittedEntry::Directory(path) => self.pending.push(path),
            AdmittedEntry::IgnoredLock => {}
            AdmittedEntry::Artifact(path) => {
                self.artifacts
                    .push(observe_file(root, path, limits, &mut self.counters)?);
            }
        }
        Ok(())
    }

    pub(super) fn finish(mut self) -> super::RecoveryObserverWalk {
        self.artifacts
            .sort_by(|left, right| left.path.cmp(&right.path));
        super::RecoveryObserverWalk {
            artifacts: self.artifacts,
            counters: self.counters,
        }
    }
}
