//! One envelope cell owns memory history or a journal-backed publication flight.
mod artifact_export;
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
    Memory(Arc<CanonicalCommitEnvelope>),
    Journal {
        entry: Arc<RelationalCommitIndexEntry>,
        source: Arc<dyn RelationalDurableColdSource>,
        unsettled: Option<Arc<CanonicalCommitEnvelope>>,
    },
}
#[derive(Debug)]
struct EnvelopeState {
    backing: EnvelopeBacking,
    position: Option<RetainedCanonicalPosition>,
    damage: Option<RelationalStorageUnavailable>,
    payload_digest: Option<[u8; 32]>,
    lineage: Option<crate::lineage::data::LineageCheckpointDigestBasis>,
}
impl EnvelopeState {
    fn resident_envelope(&self) -> Option<Arc<CanonicalCommitEnvelope>> {
        match &self.backing {
            EnvelopeBacking::Memory(envelope) => Some(envelope.clone()),
            EnvelopeBacking::Journal {
                unsettled: Some(envelope),
                ..
            } => Some(envelope.clone()),
            EnvelopeBacking::Journal { .. } => self
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
        payload_digest: [u8; 32],
    ) -> Self {
        Self {
            state: Mutex::new(EnvelopeState {
                lineage: Some(lineage_delta(&envelope)),
                backing: EnvelopeBacking::Memory(envelope),
                position: None,
                damage: None,
                payload_digest: Some(payload_digest),
            }),
        }
    }
    pub(crate) fn cold(
        entry: RelationalCommitIndexEntry,
        source: Arc<dyn RelationalDurableColdSource>,
        admitted: Option<PositionedCanonicalCommit>,
        payload_digest: Option<[u8; 32]>,
    ) -> Self {
        Self {
            state: Mutex::new(EnvelopeState {
                backing: EnvelopeBacking::Journal {
                    entry: Arc::new(entry),
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
            }),
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
            EnvelopeBacking::Journal { entry, .. } => entry.position,
            EnvelopeBacking::Memory(_) => state.position.as_ref()?.position(),
        };
        Some((position, state.lineage.clone()?))
    }
    pub(crate) fn journal_index(&self) -> Option<Arc<RelationalCommitIndexEntry>> {
        match &self
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .backing
        {
            EnvelopeBacking::Journal { entry, .. } => Some(entry.clone()),
            EnvelopeBacking::Memory(_) => None,
        }
    }
    /// Called with the actual performed value, before its route becomes visible.
    /// Journal-backed bodies remain pinned until catalog settlement completes.
    pub(crate) fn bind_performed(
        &self,
        positioned: &PositionedCanonicalCommit,
        journal: Option<(
            RelationalCommitIndexEntry,
            Arc<dyn RelationalDurableColdSource>,
        )>,
    ) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state.position = Some(RetainedCanonicalPosition::from_positioned(positioned));
        if let Some((entry, source)) = journal {
            state.backing = EnvelopeBacking::Journal {
                entry: Arc::new(entry),
                source,
                unsettled: Some(positioned.canonical_arc().clone()),
            };
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
    ) -> Result<PositionedCanonicalCommit, Failure> {
        let _resident = self.demand(execution, limit)?;
        Ok(self
            .positioned_if_resident()
            .expect("admitted envelope retains its sealed position"))
    }
    pub(crate) fn demand(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<Arc<CanonicalCommitEnvelope>, Failure> {
        crate::execution::check_request(execution).map_err(Failure::Request)?;
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(failure) = state.damage {
            return Err(failure.into());
        }
        if let Some(envelope) = state.resident_envelope() {
            return Ok(envelope);
        }
        let EnvelopeBacking::Journal { entry, source, .. } = &state.backing else {
            unreachable!("memory history owns its body")
        };
        let result =
            Self::readmit_positioned(entry, source.as_ref(), execution, limit).map_err(|stop| {
                match stop {
                    RelationalRecoveryStop::Request(cause) => Failure::Request(cause),
                    RelationalRecoveryStop::Allocation(_) => RelationalStorageUnavailable {
                        subject: RelationalStorageSubject::Envelope(entry.position),
                        cause: Cause::MemoryExhausted,
                    }
                    .into(),
                    RelationalRecoveryStop::Durability(error) => error
                        .storage_unavailable
                        .unwrap_or(RelationalStorageUnavailable {
                            subject: RelationalStorageSubject::Envelope(entry.position),
                            cause: Cause::Damaged,
                        })
                        .into(),
                }
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
                Ok(envelope)
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
    pub(super) fn readmit_positioned(
        entry: &RelationalCommitIndexEntry,
        source: &dyn RelationalDurableColdSource,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<PositionedCanonicalCommit, RelationalRecoveryStop> {
        let failure = |cause| RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Envelope(entry.position),
            cause,
        };
        let bytes = source
            .read_commit(entry.position, limit)
            .map_err(&failure)?;
        let artifact = RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes)
            .map_err(|_| failure(Cause::Damaged))?;
        if artifact.position() != entry.position
            || artifact.commit_id() != entry.commit.commit_id
            || artifact.payload_digest().as_bytes() != &entry.payload_digest
        {
            return Err(failure(Cause::IntegrityMismatch).into());
        }
        let admitted = RelationalCanonicalCommitArtifact::readmit(artifact.as_bytes(), execution)
            .map_err(|error| match error {
                crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal::Request(cause) => RelationalRecoveryStop::Request(cause),
                crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal::FieldText(cause) => crate::symbols::data::field_text_recovery_stop(cause),
                _ => failure(Cause::Damaged).into(),
            })?.into_positioned();
        if admitted.commit != entry.commit
            || admitted.schema_version != entry.schema_version
            || entry.root.schema_root()
                != &crate::schema::data::schema_authority_snapshot_digest_bytes(
                    &admitted.schema_authority,
                )
        {
            return Err(failure(Cause::IntegrityMismatch).into());
        }
        crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
        Ok(admitted)
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
