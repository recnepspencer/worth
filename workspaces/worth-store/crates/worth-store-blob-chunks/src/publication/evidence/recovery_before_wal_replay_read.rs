use super::{
    BlobPublicationClassification, BlobPublicationCrashBoundaryReport, BlobPublicationCrashEdge,
    BlobPublicationCrashOutcome, BlobPublicationObservationSet, BlobPublicationPersistedBytes,
    BlobPublicationReplayReadArtifact, BlobPublicationReplayReadDenial,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobPublicationBeforeWalReplayRead {
    persisted_bytes: BlobPublicationPersistedBytes,
    classification: BlobPublicationClassification,
    crash_report: BlobPublicationCrashBoundaryReport,
    _seal: BeforeWalReplayReadSeal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BeforeWalReplayReadSeal;

impl BlobPublicationBeforeWalReplayRead {
    pub(crate) fn from_admitted_crash_edge(
        edge: BlobPublicationCrashEdge,
    ) -> Result<Self, BlobPublicationReplayReadDenial> {
        let persisted_bytes = match &edge {
            BlobPublicationCrashEdge::BeforeWalAppend { operation_digest } => {
                BlobPublicationPersistedBytes::before_wal_append(operation_digest)
            }
            BlobPublicationCrashEdge::AfterWalAppendBeforeDurability {
                wal_range,
                operation_digest,
            } => BlobPublicationPersistedBytes::after_wal_append_before_durability(
                *wal_range,
                operation_digest,
            ),
            BlobPublicationCrashEdge::DuringCheckpointCutover { checkpoint_digest } => {
                BlobPublicationPersistedBytes::during_checkpoint_cutover(checkpoint_digest)
            }
            BlobPublicationCrashEdge::AfterDurabilityBeforeAck { .. } => {
                return Err(BlobPublicationReplayReadDenial::NotBeforeWalAppend {
                    actual_operation_digest: None,
                });
            }
        };
        Self::from_admitted_persisted_bytes(persisted_bytes)
    }

    fn from_admitted_persisted_bytes(
        persisted_bytes: BlobPublicationPersistedBytes,
    ) -> Result<Self, BlobPublicationReplayReadDenial> {
        let classification = BlobPublicationClassification::classify_observations(
            BlobPublicationObservationSet::new().with_persisted_bytes(persisted_bytes.clone()),
        );
        let Ok(crash_report) =
            BlobPublicationCrashBoundaryReport::admit_classification(&classification)
        else {
            return Err(not_before_wal_append(&classification));
        };
        if crash_report.outcome() != BlobPublicationCrashOutcome::NoWalAppendObserved
            || classification
                .before_wal_append_operation_digest()
                .is_none()
        {
            return Err(not_before_wal_append(&classification));
        }
        Ok(Self {
            persisted_bytes,
            classification,
            crash_report,
            _seal: BeforeWalReplayReadSeal,
        })
    }

    pub fn into_replay_read_artifact(
        self,
        recovery_entry_digest: impl Into<String>,
    ) -> BlobPublicationReplayReadArtifact {
        BlobPublicationReplayReadArtifact::from_admitted_before_wal_read(
            recovery_entry_digest,
            self.persisted_bytes,
            self.classification,
            self.crash_report,
        )
    }
}

fn not_before_wal_append(
    classification: &BlobPublicationClassification,
) -> BlobPublicationReplayReadDenial {
    BlobPublicationReplayReadDenial::NotBeforeWalAppend {
        actual_operation_digest: classification
            .before_wal_append_operation_digest()
            .map(str::to_owned),
    }
}
