mod backup_artifact;
mod backup_artifact_decode;
mod binding_compaction;
mod certificate;
mod compaction_cutover_record;
mod dirty_basis;
mod encoded_digest;
mod footer;
mod identity;
#[cfg(test)]
mod inspection;
mod record;
mod release_certificate;
mod selective_aggregate;
mod source;
mod stream;
mod tier_certificate;

pub use certificate::{
    checkpoint_certificate_frame_bytes, decode_checkpoint_certificate,
    encode_checkpoint_certificate, encode_checkpoint_certificate_in_reserved,
    CheckpointCertificateKind, CHECKPOINT_CERTIFICATE_PREFIX_BYTES,
    CHECKPOINT_CERTIFICATE_RECORD_OVERHEAD_BYTES, MAX_CHECKPOINT_CERTIFICATE_BYTES,
    MAX_CHECKPOINT_CERTIFICATE_RECORDS,
};
pub use dirty_basis::{CheckpointDirtyFrameBasis, CHECKPOINT_DIRTY_FRAME_RECORD_BYTES};
pub use encoded_digest::checkpoint_stream_encoded_digest;
pub use footer::{
    CheckpointStreamFooter, CHECKPOINT_CERTIFIED_FOOTER_RECORD_BYTES,
    CHECKPOINT_STREAM_FOOTER_RECORD_BYTES,
};
pub use identity::PhysicalCheckpointIdentity;
#[cfg(test)]
pub(crate) use inspection::inspect_checkpoint_stream;
pub use record::{CheckpointStreamDecodeDenial, CheckpointStreamEncodingDenial};
pub const CHECKPOINT_CERTIFIED_SCHEMA: u8 = record::CERTIFIED_CHECKPOINT_SCHEMA;
pub use release_certificate::{
    release_checkpoint_batch_records_digest_v1, ReleaseCheckpointAccumulatorV1,
    ReleaseCheckpointAccumulatorV2, ReleaseCheckpointBatchV1, ReleaseCheckpointCertificateDenial,
    ReleaseCheckpointCertificateV1, ReleaseCheckpointNoReleaseV1, ReleasedDropCumulativeEvidenceV1,
    ReleasedDropTipProvenanceV1, ReleasedDropWalFateWitnessV1,
    RELEASE_CHECKPOINT_ACCUMULATOR_V2_WIRE_BYTES, RELEASE_CHECKPOINT_ACCUMULATOR_WIRE_BYTES,
    RELEASE_CHECKPOINT_BATCH_WIRE_BYTES, RELEASE_CHECKPOINT_NO_RELEASE_WIRE_BYTES,
    RELEASE_CHECKPOINT_TIP_WIRE_BYTES,
};
pub use selective_aggregate::{
    CheckpointSelectiveRecordAggregate, CheckpointSelectiveRecordSummary,
};
pub use source::{
    CheckpointRootBasis, CheckpointWalSourceRange, PhysicalCheckpointSecurityBinding,
    PhysicalCheckpointSource, CHECKPOINT_STREAM_HEADER_RECORD_BYTES,
};
#[cfg(test)]
pub(crate) use stream::CheckpointStreamDecoder;
pub use stream::{CheckpointBindingCompactionEncoder, CheckpointStreamEncoder};
pub use tier_certificate::{
    TierEpochCheckpointCertificateDenial, TierEpochCheckpointCertificateV1,
    TierEpochWalFrameWitnessV1, TIER_EPOCH_CHECKPOINT_CERTIFICATE_WIRE_BYTES,
};

#[cfg(test)]
mod tests;
pub use backup_artifact::{CheckpointBackupArtifact, CheckpointBackupArtifactInput};
pub use backup_artifact_decode::{
    decode_checkpoint_backup_artifact_from_reader, CheckpointBackupArtifactDecodeDenial,
    CheckpointBackupArtifactDecodeObservation, CheckpointBackupArtifactDecodeRequest,
    DecodedCheckpointBackupArtifact,
};
pub use binding_compaction::{
    decode_checkpoint_binding_record, CheckpointBindingCompactionHeader,
    CheckpointBindingRecordFrameLength, CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES,
    CHECKPOINT_BINDING_RECORD_PREFIX_BYTES, MAX_CHECKPOINT_BINDING_RECORD_BYTES,
    PHYSICAL_MUTATION_BINDING_COMPACTION_RECORD_DOMAIN,
};
#[cfg(test)]
pub(crate) use compaction_cutover_record::PersistedCompactionCutoverRecord;
pub use compaction_cutover_record::PersistedCompactionProductRole;
