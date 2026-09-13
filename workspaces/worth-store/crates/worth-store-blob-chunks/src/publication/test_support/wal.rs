use worth_store_physical_backend::{
    BackendDurabilityProfile, SimulatedStrictDurableProfile, WalAppendObservationScope,
    WalAppendReceipt,
};
use worth_store_wal::{
    LogSequenceNumber, PublicationDeclaration, WalFramePublicationScope, WalLsnRange,
    WalSegmentGeneration, WalSegmentId,
};

use crate::publication::evidence::identity::BlobPublicationRecoveryOperationDigest;
use crate::publication::evidence::BlobPublicationCrashEdge;
use crate::publication::{
    BlobPublicationBeforeWalReplayRead, BlobPublicationCrashBoundaryReport,
    BlobPublicationReplayReadArtifact, BlobPublicationReplayReadDenial,
    BlobPublicationReplayedCrashEdge,
};
use crate::BlobPublicationPreWalReplayEvidence;

pub(crate) fn durable_wal_publication(frame_digest: &str) -> PublicationDeclaration {
    let scope = WalFramePublicationScope::new(
        WalSegmentId::new(7).unwrap(),
        WalSegmentGeneration::new(1).unwrap(),
        WalLsnRange::new(LogSequenceNumber::new(10), LogSequenceNumber::new(11)).unwrap(),
        frame_digest,
        64,
    )
    .expect("wal frame publication scope should admit");
    PublicationDeclaration::wal_frame(scope)
}

pub(crate) fn replayable_wal_classification(
    frame_digest: &str,
) -> BlobPublicationCrashBoundaryReport {
    let scope = WalAppendObservationScope::new(
        WalSegmentId::new(7).unwrap(),
        WalSegmentGeneration::new(1).unwrap(),
        WalLsnRange::new(LogSequenceNumber::new(10), LogSequenceNumber::new(11)).unwrap(),
        frame_digest,
        64,
    )
    .expect("WAL observation scope should admit");
    let receipt = WalAppendReceipt::<SimulatedStrictDurableProfile>::from_certification_observation(
        scope,
        64,
        SimulatedStrictDurableProfile::REQUIRED_BARRIERS,
        None,
    );
    BlobPublicationCrashBoundaryReport::admit_crash_edge(
        BlobPublicationCrashEdge::after_durability_before_ack(receipt),
    )
    .expect("phase-22 crash report should admit replayable WAL evidence")
}

pub(crate) fn pre_wal_replay_edge(
    operation_digest: &BlobPublicationRecoveryOperationDigest,
) -> BlobPublicationReplayedCrashEdge {
    with_recovery_replay_entry(operation_digest.as_str(), |replay_entry| {
        let artifact = replay_entry
            .read_blob_publication_before_wal_append()
            .expect("test recovery entry carries protected before-WAL replay bytes");
        BlobPublicationReplayedCrashEdge::from_replay_read_artifact(artifact)
            .expect("test pre-wal replay witness should admit through production readmission")
    })
}

pub(crate) struct RecoveryReplayReadProbe {
    entry_digest: String,
    replay_read: Option<BlobPublicationBeforeWalReplayRead>,
}

pub(crate) fn with_recovery_replay_entry<R>(
    operation_digest: &str,
    run: impl FnOnce(RecoveryReplayReadProbe) -> R,
) -> R {
    let replay_read = BlobPublicationBeforeWalReplayRead::from_admitted_crash_edge(
        BlobPublicationCrashEdge::before_wal_append(operation_digest),
    )
    .expect("test recovery entry carries admitted before-WAL replay bytes");
    run(RecoveryReplayReadProbe {
        entry_digest: operation_digest.to_owned(),
        replay_read: Some(replay_read),
    })
}

pub(crate) fn with_generic_recovery_replay_entry<R>(
    operation_digest: &str,
    run: impl FnOnce(RecoveryReplayReadProbe) -> R,
) -> R {
    run(RecoveryReplayReadProbe {
        entry_digest: operation_digest.to_owned(),
        replay_read: None,
    })
}

impl RecoveryReplayReadProbe {
    pub(crate) fn read_blob_publication_before_wal_append(
        &self,
    ) -> Result<BlobPublicationReplayReadArtifact, BlobPublicationReplayReadDenial> {
        let Some(replay_read) = self.replay_read.clone() else {
            return Err(BlobPublicationReplayReadDenial::NotBeforeWalAppend {
                actual_operation_digest: None,
            });
        };
        Ok(replay_read.into_replay_read_artifact(self.entry_digest.clone()))
    }

    pub(crate) fn read_blob_publication_checkpoint_cutover(
        &self,
        _operation_digest: &str,
    ) -> Result<BlobPublicationReplayReadArtifact, BlobPublicationReplayReadDenial> {
        Err(BlobPublicationReplayReadDenial::NotBeforeWalAppend {
            actual_operation_digest: None,
        })
    }
}

pub(crate) fn chunk_write_replay_evidence(
    digest: &crate::LogicalContentDigest,
) -> BlobPublicationPreWalReplayEvidence {
    let replay = pre_wal_replay_edge(
        &BlobPublicationPreWalReplayEvidence::chunk_write_recovery_operation_digest(digest),
    );
    BlobPublicationPreWalReplayEvidence::from_chunk_write_replay(digest, &replay)
        .expect("chunk-write replay evidence should admit")
}
