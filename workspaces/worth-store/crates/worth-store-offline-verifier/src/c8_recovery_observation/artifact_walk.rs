use std::path::Path;

use super::observer_evidence_accumulation::RecoveryObserverArtifactEvidence;
use super::{
    RecoveryObserverCounters, RecoveryObserverLimits, RecoveryObserverObservationDenial,
    RecoveryObserverObservationFailure,
};

#[path = "artifact_walk/directory_entry_scan.rs"]
mod directory_entry_scan;
#[path = "artifact_walk/entry_admission.rs"]
mod entry_admission;
#[path = "artifact_walk/entry_classification.rs"]
mod entry_classification;
#[path = "artifact_walk/walk_state.rs"]
mod walk_state;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ObservedRecoveryArtifact {
    pub(super) path: Box<str>,
    pub(super) byte_length: u64,
    pub(super) digest: [u8; 32],
    pub(super) evidence: RecoveryObserverArtifactEvidence,
}

#[derive(Debug)]
pub(super) struct RecoveryObserverWalk {
    artifacts: Vec<ObservedRecoveryArtifact>,
    counters: RecoveryObserverCounters,
}

pub(super) fn walk(
    store_root: &Path,
    limits: RecoveryObserverLimits,
) -> Result<RecoveryObserverWalk, RecoveryObserverObservationFailure> {
    let root = store_root.canonicalize().map_err(|error| {
        RecoveryObserverObservationFailure::at_path(
            RecoveryObserverObservationDenial::Media(error.kind()),
            RecoveryObserverCounters::with_root_admitted(),
            store_root,
        )
    })?;
    let mut state = walk_state::WalkState::new(root.clone());
    while let Some(directory) = state.next_directory() {
        let mut entries = directory_entry_scan::admitted(&directory, limits, state.counters_mut())?;
        entries.sort_by_key(std::fs::DirEntry::file_name);
        for entry in entries.into_iter().rev() {
            let classified = entry_classification::classify(entry, state.counters())?;
            let admitted = entry_admission::admit(classified, limits, state.counters_mut())?;
            state.apply(&root, admitted, limits)?;
        }
    }
    Ok(state.finish())
}

impl RecoveryObserverWalk {
    pub(super) fn artifacts(&self) -> &[ObservedRecoveryArtifact] {
        &self.artifacts
    }

    pub(super) const fn counters(&self) -> RecoveryObserverCounters {
        self.counters
    }
}

impl ObservedRecoveryArtifact {
    pub(super) fn path(&self) -> &str {
        &self.path
    }

    pub(super) const fn byte_length(&self) -> u64 {
        self.byte_length
    }

    pub(super) const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    pub(super) const fn evidence(&self) -> RecoveryObserverArtifactEvidence {
        self.evidence
    }
}
