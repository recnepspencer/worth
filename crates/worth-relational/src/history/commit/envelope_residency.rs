//! Indexed envelopes keep only weak flight custody; readers own admitted bodies.
use super::RelationalCommitIndexEntry;
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact;
use crate::durability::data::RelationalRecoveryStop;
use crate::durability::medium::RelationalDurableColdSource;
use crate::history::data::{CanonicalCommitEnvelope, PositionedCanonicalCommit};
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause,
};
use crate::storage::residency::RelationalResidencyDemandFailure as Failure;
use std::sync::{Arc, Mutex, Weak};

#[derive(Debug)]
pub(crate) struct RelationalCommitEnvelopeResidency {
    backing: EnvelopeBacking,
    flight: Mutex<EnvelopeFlight>,
}
#[derive(Debug)]
enum EnvelopeBacking {
    Memory(Arc<CanonicalCommitEnvelope>),
    Journal {
        entry: RelationalCommitIndexEntry,
        source: Arc<dyn RelationalDurableColdSource>,
    },
}
#[derive(Debug, Default)]
struct EnvelopeFlight {
    envelope: Weak<CanonicalCommitEnvelope>,
    damage: Option<RelationalStorageUnavailable>,
}
impl RelationalCommitEnvelopeResidency {
    pub(crate) fn resident(envelope: Arc<CanonicalCommitEnvelope>) -> Self {
        Self {
            backing: EnvelopeBacking::Memory(envelope),
            flight: Mutex::default(),
        }
    }
    pub(crate) fn cold(
        entry: RelationalCommitIndexEntry,
        source: Arc<dyn RelationalDurableColdSource>,
        admitted: Option<Arc<CanonicalCommitEnvelope>>,
    ) -> Self {
        Self {
            backing: EnvelopeBacking::Journal { entry, source },
            flight: Mutex::new(EnvelopeFlight {
                envelope: admitted.as_ref().map_or_else(Weak::new, Arc::downgrade),
                damage: None,
            }),
        }
    }
    pub(crate) fn get(&self) -> Option<Arc<CanonicalCommitEnvelope>> {
        match &self.backing {
            EnvelopeBacking::Memory(envelope) => Some(envelope.clone()),
            EnvelopeBacking::Journal { .. } => self
                .flight
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .envelope
                .upgrade(),
        }
    }
    pub(crate) fn demand(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<Arc<CanonicalCommitEnvelope>, Failure> {
        crate::execution::check_request(execution).map_err(Failure::Request)?;
        let (entry, source) = match &self.backing {
            EnvelopeBacking::Memory(envelope) => return Ok(envelope.clone()),
            EnvelopeBacking::Journal { entry, source } => (entry, source),
        };
        let mut flight = self
            .flight
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if let Some(failure) = flight.damage {
            return Err(failure.into());
        }
        if let Some(envelope) = flight.envelope.upgrade() {
            return Ok(envelope);
        }
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
                let envelope = commit.canonical_arc().clone();
                flight.envelope = Arc::downgrade(&envelope);
                Ok(envelope)
            }
            Err(stop) => {
                if let Failure::StorageUnavailable(failure) = &stop
                    && failure.cause.is_proven_damage()
                {
                    flight.damage = Some(*failure);
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
