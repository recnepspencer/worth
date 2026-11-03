//! One envelope cell owns memory history or a journal-backed publication flight.
mod artifact_export;
mod authored_writes;
mod selected_checkpoint;
pub(crate) use selected_checkpoint::RetiredCanonicalPlacement;
mod demand;
mod source_readmission;
mod source_retirement;
use super::RelationalCommitIndexEntry;
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact;
use crate::durability::data::RelationalRecoveryStop;
use crate::durability::medium::RelationalDurableColdSource;
use crate::history::data::{CanonicalCommitEnvelope, PositionedCanonicalCommit};
use crate::runtime::RetainedCanonicalPosition;
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause,
};
use crate::storage::residency::RelationalResidencyDemandFailure as Failure;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub(crate) struct RelationalCommitEnvelopeResidency {
    state: Mutex<EnvelopeState>,
}
#[derive(Debug)]
enum EnvelopeBacking {
    Memory {
        envelope: Arc<CanonicalCommitEnvelope>,
        original: Option<worth_execution::ExecutionImmutableBytes>,
    },
    SelectedCheckpoint {
        entry: super::RetainedCanonicalIndex,
        directory: crate::durability::data::NativeCheckpointDirectoryEdge,
    },
    Journal {
        entry: super::RetainedCanonicalIndex,
        source: crate::durability::medium::NativeJournalSourceEdge,
        unsettled: Option<Arc<CanonicalCommitEnvelope>>,
    },
}
#[derive(Debug)]
struct EnvelopeState {
    authored_writes: Option<crate::runtime::AuthoredWriteGeneration>,
    backing: EnvelopeBacking,
    position: Option<RetainedCanonicalPosition>,
    damage: Option<RelationalStorageUnavailable>,
    payload_digest: Option<[u8; 32]>,
    canonical_payload_bytes: Option<u64>,
    lineage: Option<crate::lineage::data::LineageCheckpointDigestBasis>,
}
impl EnvelopeState {
    fn resident_envelope(&self) -> Option<Arc<CanonicalCommitEnvelope>> {
        match &self.backing {
            EnvelopeBacking::Memory { envelope, .. } => Some(envelope.clone()),
            EnvelopeBacking::Journal {
                unsettled: Some(envelope),
                ..
            } => Some(envelope.clone()),
            EnvelopeBacking::Journal { .. } | EnvelopeBacking::SelectedCheckpoint { .. } => self
                .position
                .as_ref()
                .and_then(|retained| retained.resident())
                .map(|commit| commit.canonical_arc().clone()),
        }
    }
}
impl RelationalCommitEnvelopeResidency {
    pub(crate) fn resident(
        envelope: Arc<CanonicalCommitEnvelope>,
        canonical_payload: (u64, [u8; 32]),
    ) -> Self {
        let (canonical_payload_bytes, payload_digest) = canonical_payload;
        Self {
            state: Mutex::new(EnvelopeState {
                authored_writes: None,
                lineage: Some(lineage_delta(&envelope)),
                backing: EnvelopeBacking::Memory {
                    envelope,
                    original: None,
                },
                position: None,
                damage: None,
                payload_digest: Some(payload_digest),
                canonical_payload_bytes: Some(canonical_payload_bytes),
            }),
        }
    }
    pub(crate) fn cold(
        entry: super::RetainedCanonicalIndex,
        source: Arc<dyn RelationalDurableColdSource>,
        admitted: Option<PositionedCanonicalCommit>,
        payload_digest: Option<[u8; 32]>,
    ) -> Self {
        Self {
            state: Mutex::new(EnvelopeState {
                authored_writes: None,
                backing: EnvelopeBacking::Journal {
                    entry,
                    source: crate::durability::medium::NativeJournalSourceEdge::new(source),
                    unsettled: None,
                },
                position: admitted
                    .as_ref()
                    .map(RetainedCanonicalPosition::from_positioned),
                lineage: admitted
                    .as_ref()
                    .map(|commit| lineage_delta(commit.envelope())),
                damage: None,
                payload_digest,
                canonical_payload_bytes: None,
            }),
        }
    }
    #[cfg(test)]
    pub(crate) fn resident_payload_bytes(&self) -> Option<u64> {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .canonical_payload_bytes
    }
    pub(crate) fn payload_storage_bytes(&self) -> u64 {
        match &self
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .backing
        {
            EnvelopeBacking::Memory {
                original: Some(bytes),
                ..
            } => bytes.len() as u64,
            _ => 0,
        }
    }
    pub(crate) fn payload_digest(&self) -> Option<[u8; 32]> {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .payload_digest
    }
    pub(crate) fn get(&self) -> Option<Arc<CanonicalCommitEnvelope>> {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .resident_envelope()
    }
    /// Preserve the exact sealed position of an already resident head or reader.
    pub(crate) fn positioned_if_resident(&self) -> Option<PositionedCanonicalCommit> {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .position
            .as_ref()
            .and_then(RetainedCanonicalPosition::resident)
    }
    pub(crate) fn checkpoint_lineage(
        &self,
    ) -> Option<(
        crate::publication::patch::data::PatchStreamPosition,
        crate::lineage::data::LineageCheckpointDigestBasis,
    )> {
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let position = match &state.backing {
            EnvelopeBacking::Journal { entry, .. }
            | EnvelopeBacking::SelectedCheckpoint { entry, .. } => entry.position,
            EnvelopeBacking::Memory { .. } => state.position.as_ref()?.position(),
        };
        Some((position, state.lineage.clone()?))
    }
    pub(crate) fn captured_position(
        &self,
    ) -> Option<crate::publication::patch::data::PatchStreamPosition> {
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        match &state.backing {
            EnvelopeBacking::Journal { entry, .. }
            | EnvelopeBacking::SelectedCheckpoint { entry, .. } => Some(entry.position),
            EnvelopeBacking::Memory { .. } => {
                state.position.as_ref().map(|position| position.position())
            }
        }
    }
    pub(crate) fn journal_index(&self) -> Option<super::RetainedCanonicalIndex> {
        self.source_placement().map(|(entry, _)| entry)
    }
    pub(crate) fn journal_placement(
        &self,
    ) -> Option<(
        super::RetainedCanonicalIndex,
        Arc<dyn RelationalDurableColdSource>,
    )> {
        match &self
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .backing
        {
            EnvelopeBacking::Journal { entry, source, .. } => {
                Some((entry.clone(), Arc::clone(source)))
            }
            EnvelopeBacking::Memory { .. } | EnvelopeBacking::SelectedCheckpoint { .. } => None,
        }
    }
    pub(crate) fn source_placement(
        &self,
    ) -> Option<(
        super::RetainedCanonicalIndex,
        Arc<dyn RelationalDurableColdSource>,
    )> {
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        match &state.backing {
            EnvelopeBacking::Journal { entry, source, .. } => {
                Some((entry.clone(), Arc::clone(source)))
            }
            EnvelopeBacking::SelectedCheckpoint { entry, directory } => {
                Some((entry.clone(), directory.source.clone()))
            }
            EnvelopeBacking::Memory { .. } => None,
        }
    }
    /// Called with the actual performed value, before its route becomes visible.
    /// Journal-backed bodies remain pinned until catalog settlement completes.
    pub(crate) fn bind_performed(
        &self,
        positioned: &PositionedCanonicalCommit,
        journal: Option<(
            super::RetainedCanonicalIndex,
            Arc<dyn RelationalDurableColdSource>,
        )>,
        original: Option<worth_execution::ExecutionImmutableBytes>,
    ) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state.position = Some(RetainedCanonicalPosition::from_positioned(positioned));
        if let Some((entry, source)) = journal {
            state.canonical_payload_bytes = None;
            state.backing = EnvelopeBacking::Journal {
                entry,
                source: crate::durability::medium::NativeJournalSourceEdge::new(source),
                unsettled: Some(positioned.canonical_arc().clone()),
            };
        } else if let Some(original) = original {
            let EnvelopeBacking::Memory {
                original: stored, ..
            } = &mut state.backing
            else {
                unreachable!("only a real Memory publication installs an original Memory frame")
            };
            *stored = Some(original);
        }
    }
    pub(crate) fn settle(&self) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if let EnvelopeBacking::Journal { unsettled, .. } = &mut state.backing {
            unsettled.take();
        }
    }
}

fn lineage_delta(
    envelope: &CanonicalCommitEnvelope,
) -> crate::lineage::data::LineageCheckpointDigestBasis {
    let basis = envelope.lineage_digest_basis();
    crate::lineage::data::LineageCheckpointDigestBasis::new(
        usize::from(envelope.has_lineage_authority()),
        basis.canonical_event_ids().to_vec(),
        basis.lineage_event_count(),
        basis.lineage_decision_count(),
    )
}

/// Both owners are captured while the cell is locked; settlement cannot revoke them.
struct ResidentEnvelopeRead {
    envelope: Arc<CanonicalCommitEnvelope>,
    positioned: Option<PositionedCanonicalCommit>,
}

#[cfg(test)]
mod memory_publication_tests;
