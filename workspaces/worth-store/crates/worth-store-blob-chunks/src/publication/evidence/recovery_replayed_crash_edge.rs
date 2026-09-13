use super::{
    BlobPublicationCrashOutcome, BlobPublicationReplayCounterSnapshot,
    BlobPublicationReplayReadArtifact, BlobPublicationReplayReadDenial,
    BlobPublicationReplayReadRecord, BlobPublicationReplayReadWitness,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobPublicationReplayedCrashEdge {
    witness: BlobPublicationReplayReadWitness,
    replay_read_recovery_entry_digest: String,
    replay_read_persisted_bytes_digest: String,
    replay_read_identity: String,
}

impl BlobPublicationReplayedCrashEdge {
    pub fn from_replay_read_artifact(
        artifact: BlobPublicationReplayReadArtifact,
    ) -> Result<Self, BlobPublicationReplayReadDenial> {
        let record = BlobPublicationReplayReadRecord::from_replay_read_artifact(artifact);
        let witness = BlobPublicationReplayReadWitness::readmitted_before_wal_append(record)?;
        Ok(Self::from_replay_read_witness(witness))
    }

    pub fn from_replay_read_witness(witness: BlobPublicationReplayReadWitness) -> Self {
        Self {
            replay_read_recovery_entry_digest: witness.recovery_entry_digest().to_owned(),
            replay_read_persisted_bytes_digest: witness.persisted_bytes_digest().to_owned(),
            replay_read_identity: witness.replay_read_identity().to_owned(),
            witness,
        }
    }

    pub fn outcome(&self) -> BlobPublicationCrashOutcome {
        self.witness.crash_report().outcome()
    }

    pub fn classification_digest(&self) -> &str {
        self.witness.crash_report().classification_digest()
    }

    pub fn before_wal_append_operation_digest(&self) -> Option<&str> {
        self.witness
            .classification()
            .before_wal_append_operation_digest()
    }

    pub fn counters(&self) -> BlobPublicationReplayCounterSnapshot {
        self.witness.counters()
    }

    pub fn replay_read_recovery_entry_digest(&self) -> &str {
        &self.replay_read_recovery_entry_digest
    }

    pub fn replay_read_persisted_bytes_digest(&self) -> &str {
        &self.replay_read_persisted_bytes_digest
    }

    pub fn replay_read_identity(&self) -> &str {
        &self.replay_read_identity
    }
}
