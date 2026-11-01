//! One envelope cell owns memory history or a journal-backed publication flight.
mod artifact_export;
mod authored_writes;
mod selected_checkpoint;
pub(crate) use selected_checkpoint::RetiredCanonicalPlacement;
mod source_readmission;
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
        directory: Arc<crate::durability::data::RelationalCheckpointDirectory>,
    },
    Journal {
        entry: super::RetainedCanonicalIndex,
        source: Arc<dyn RelationalDurableColdSource>,
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
                    source,
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
            EnvelopeBacking::Journal { entry, source, .. } => Some((entry.clone(), source.clone())),
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
            EnvelopeBacking::Journal { entry, source, .. } => Some((entry.clone(), source.clone())),
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
                source,
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
    pub(crate) fn demand_positioned(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<PositionedCanonicalCommit, RelationalRecoveryStop> {
        self.demand_read(execution, limit)?
            .positioned
            .ok_or_else(|| {
                crate::durability::data::DurabilityError::new(
                    crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
                    "checkpoint envelope has no performed position",
                )
                .into()
            })
    }
    pub(crate) fn demand(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<Arc<CanonicalCommitEnvelope>, Failure> {
        self.demand_read(execution, limit).map(|read| read.envelope)
    }
    fn demand_read(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<ResidentEnvelopeRead, Failure> {
        crate::execution::check_request(execution).map_err(Failure::Request)?;
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(failure) = state.damage {
            return Err(failure.into());
        }
        if let Some(envelope) = state.resident_envelope() {
            let positioned = state
                .position
                .as_ref()
                .and_then(RetainedCanonicalPosition::resident);
            return Ok(ResidentEnvelopeRead {
                envelope,
                positioned,
            });
        }
        let position = state.position.as_ref().map_or(
            crate::publication::patch::data::PatchStreamPosition(0),
            |position| position.position(),
        );
        let result = match &state.backing {
            EnvelopeBacking::Memory { envelope, .. } => {
                return Ok(ResidentEnvelopeRead {
                    envelope: envelope.clone(),
                    positioned: state
                        .position
                        .as_ref()
                        .and_then(RetainedCanonicalPosition::resident),
                })
            }
            EnvelopeBacking::Journal { entry, source, .. } => {
                Self::readmit_positioned(entry, source.as_ref(), execution, limit)
            }
            EnvelopeBacking::SelectedCheckpoint { entry, directory } => {
                Self::readmit_selected(entry, directory, execution, limit)
            }
        }
        .map_err(|stop| match stop {
            RelationalRecoveryStop::Request(cause) => Failure::Request(cause),
            RelationalRecoveryStop::Allocation(_) => RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Envelope(position),
                cause: Cause::MemoryExhausted,
            }
            .into(),
            RelationalRecoveryStop::Durability(error) => error
                .storage_unavailable
                .unwrap_or(RelationalStorageUnavailable {
                    subject: RelationalStorageSubject::Envelope(position),
                    cause: Cause::Damaged,
                })
                .into(),
        });
        match result {
            Ok(commit) => {
                let digest = commit.authoritative_payload_digest().map_err(|_| {
                    let failure = RelationalStorageUnavailable {
                        subject: RelationalStorageSubject::Envelope(commit.position()),
                        cause: Cause::Damaged,
                    };
                    state.damage = Some(failure);
                    Failure::StorageUnavailable(failure)
                })?;
                state.payload_digest = Some(digest);
                let envelope = commit.canonical_arc().clone();
                state.position = Some(RetainedCanonicalPosition::from_positioned(&commit));
                state.lineage = Some(lineage_delta(commit.envelope()));
                Ok(ResidentEnvelopeRead {
                    envelope,
                    positioned: Some(commit),
                })
            }
            Err(stop) => {
                if let Failure::StorageUnavailable(failure) = &stop
                    && failure.cause.is_proven_damage()
                {
                    state.damage = Some(*failure);
                }
                Err(stop)
            }
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
