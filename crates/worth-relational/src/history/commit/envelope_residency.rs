//! Readmission of an indexed canonical envelope under the caller's request.
use std::sync::{Arc, Mutex, OnceLock};

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

#[derive(Debug)]
pub(crate) struct RelationalCommitEnvelopeResidency {
    resident: OnceLock<AdmittedEnvelope>,
    cold: Option<(
        RelationalCommitIndexEntry,
        Arc<dyn RelationalDurableColdSource>,
    )>,
    load: Mutex<Option<RelationalStorageUnavailable>>,
}

#[derive(Debug)]
enum AdmittedEnvelope {
    Resident(Arc<CanonicalCommitEnvelope>),
    Readmitted(PositionedCanonicalCommit),
}

impl RelationalCommitEnvelopeResidency {
    pub(crate) fn resident(envelope: Arc<CanonicalCommitEnvelope>) -> Self {
        Self {
            resident: OnceLock::from(AdmittedEnvelope::Resident(envelope)),
            cold: None,
            load: Mutex::new(None),
        }
    }

    pub(crate) fn cold(
        entry: RelationalCommitIndexEntry,
        source: Arc<dyn RelationalDurableColdSource>,
        admitted: Option<Arc<CanonicalCommitEnvelope>>,
    ) -> Self {
        let resident = OnceLock::new();
        if let Some(envelope) = admitted {
            resident
                .set(AdmittedEnvelope::Resident(envelope))
                .expect("new envelope cell");
        }
        Self {
            resident,
            cold: Some((entry, source)),
            load: Mutex::new(None),
        }
    }

    pub(crate) fn get(&self) -> Option<&Arc<CanonicalCommitEnvelope>> {
        self.resident.get().map(|admitted| match admitted {
            AdmittedEnvelope::Resident(envelope) => envelope,
            AdmittedEnvelope::Readmitted(commit) => commit.canonical_arc(),
        })
    }

    pub(super) fn into_positioned(
        self,
    ) -> Result<PositionedCanonicalCommit, RelationalRecoveryStop> {
        match self.resident.into_inner() {
            Some(AdmittedEnvelope::Readmitted(commit)) => Ok(commit),
            _ => Err(crate::durability::data::DurabilityError::new(
                crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
                "head envelope has no canonical position admission",
            )
            .into()),
        }
    }

    pub(crate) fn demand(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<&Arc<CanonicalCommitEnvelope>, Failure> {
        crate::execution::check_request(execution).map_err(Failure::Request)?;
        if let Some(envelope) = self.get() {
            return Ok(envelope);
        }
        let mut damage = self.load.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(envelope) = self.get() {
            return Ok(envelope);
        }
        if let Some(failure) = &*damage {
            return Err((*failure).into());
        }
        let (entry, source) = self.cold.as_ref().expect("unloaded cell has a source");
        let result = self
            .readmit(entry, source.as_ref(), execution, limit)
            .map_err(|stop| match stop {
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
            });
        match result {
            Ok(commit) => {
                self.resident
                    .set(AdmittedEnvelope::Readmitted(commit))
                    .expect("one load owns admission");
                Ok(self.get().expect("admitted envelope"))
            }
            Err(stop) => {
                if let Failure::StorageUnavailable(failure) = &stop
                    && failure.cause.is_proven_damage()
                {
                    *damage = Some(*failure);
                }
                Err(stop)
            }
        }
    }

    fn readmit(
        &self,
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
