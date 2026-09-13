#[cfg(test)]
use super::BlobPublicationPersistedBytes;
use super::{
    BlobPublicationClassification, BlobPublicationCrashBoundaryReport, BlobPublicationCrashOutcome,
    BlobPublicationReplayCounterSnapshot,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobPublicationReplayReadRecord {
    recovery_entry_digest: String,
    persisted_bytes_digest: String,
    classification: BlobPublicationClassification,
    crash_report: BlobPublicationCrashBoundaryReport,
    _seal: ReplayReadRecordSeal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReplayReadRecordSeal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobPublicationReplayReadArtifact {
    recovery_entry_digest: String,
    persisted_bytes_digest: String,
    classification: BlobPublicationClassification,
    crash_report: BlobPublicationCrashBoundaryReport,
    _seal: ReplayReadArtifactSeal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReplayReadArtifactSeal;

#[derive(Debug, Clone, PartialEq, Eq)]
struct BlobPublicationReplayReadSource {
    recovery_entry_digest: String,
    persisted_bytes_digest: String,
    classification: BlobPublicationClassification,
    crash_report: BlobPublicationCrashBoundaryReport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobPublicationReplayReadWitness {
    replay_read_identity: String,
    source: BlobPublicationReplayReadSource,
    operation_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlobPublicationReplayReadDenial {
    NotBeforeWalAppend {
        actual_operation_digest: Option<String>,
    },
}

impl BlobPublicationReplayReadArtifact {
    #[cfg(test)]
    pub(crate) fn from_admitted_before_wal_read(
        recovery_entry_digest: impl Into<String>,
        bytes: BlobPublicationPersistedBytes,
        classification: BlobPublicationClassification,
        crash_report: BlobPublicationCrashBoundaryReport,
    ) -> Self {
        Self {
            recovery_entry_digest: recovery_entry_digest.into(),
            persisted_bytes_digest: bytes.persisted_bytes_digest(),
            classification,
            crash_report,
            _seal: ReplayReadArtifactSeal,
        }
    }

    pub fn recovery_entry_digest(&self) -> &str {
        &self.recovery_entry_digest
    }

    pub fn persisted_bytes_digest(&self) -> &str {
        &self.persisted_bytes_digest
    }

    pub const fn crash_report(&self) -> &BlobPublicationCrashBoundaryReport {
        &self.crash_report
    }
}

impl BlobPublicationReplayReadRecord {
    pub fn from_replay_read_artifact(artifact: BlobPublicationReplayReadArtifact) -> Self {
        Self {
            recovery_entry_digest: artifact.recovery_entry_digest,
            persisted_bytes_digest: artifact.persisted_bytes_digest,
            classification: artifact.classification,
            crash_report: artifact.crash_report,
            _seal: ReplayReadRecordSeal,
        }
    }

    pub fn recovery_entry_digest(&self) -> &str {
        &self.recovery_entry_digest
    }

    pub fn persisted_bytes_digest(&self) -> &str {
        &self.persisted_bytes_digest
    }

    pub const fn crash_report(&self) -> &BlobPublicationCrashBoundaryReport {
        &self.crash_report
    }

    fn into_source(self) -> BlobPublicationReplayReadSource {
        BlobPublicationReplayReadSource {
            recovery_entry_digest: self.recovery_entry_digest,
            persisted_bytes_digest: self.persisted_bytes_digest,
            classification: self.classification,
            crash_report: self.crash_report,
        }
    }
}

impl BlobPublicationReplayReadWitness {
    pub fn readmitted_before_wal_append(
        record: BlobPublicationReplayReadRecord,
    ) -> Result<Self, BlobPublicationReplayReadDenial> {
        let source = record.into_source();
        if source.crash_report.outcome() != BlobPublicationCrashOutcome::NoWalAppendObserved {
            return Err(BlobPublicationReplayReadDenial::NotBeforeWalAppend {
                actual_operation_digest: source
                    .classification
                    .before_wal_append_operation_digest()
                    .map(str::to_owned),
            });
        }
        let Some(operation_digest) = source.classification.before_wal_append_operation_digest()
        else {
            return Err(BlobPublicationReplayReadDenial::NotBeforeWalAppend {
                actual_operation_digest: None,
            });
        };
        let operation_digest = operation_digest.to_owned();
        let replay_read_identity = format!(
            "partial-publication-replay-read:v1:entry={}:bytes={}:kind=before-wal-append:operation={}",
            source.recovery_entry_digest,
            source.persisted_bytes_digest,
            operation_digest
        );
        Ok(Self {
            replay_read_identity,
            source,
            operation_digest,
        })
    }

    pub fn replay_read_identity(&self) -> &str {
        &self.replay_read_identity
    }

    pub fn recovery_entry_digest(&self) -> &str {
        &self.source.recovery_entry_digest
    }

    pub fn persisted_bytes_digest(&self) -> &str {
        &self.source.persisted_bytes_digest
    }

    pub fn operation_digest(&self) -> &str {
        &self.operation_digest
    }

    pub const fn counters(&self) -> BlobPublicationReplayCounterSnapshot {
        self.source.classification.counters()
    }

    pub(crate) fn classification(&self) -> &BlobPublicationClassification {
        &self.source.classification
    }

    pub const fn crash_report(&self) -> &BlobPublicationCrashBoundaryReport {
        &self.source.crash_report
    }
}
