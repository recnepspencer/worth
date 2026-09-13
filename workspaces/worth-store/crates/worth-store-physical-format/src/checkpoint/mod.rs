mod backup_artifact;
mod backup_artifact_decode;
mod binding_compaction;
mod compaction_cutover_record;
mod dirty_basis;
mod encoded_digest;
mod footer;
mod identity;
#[cfg(test)]
mod inspection;
mod record;
mod selective_aggregate;
mod source;
mod stream;

pub use dirty_basis::{CheckpointDirtyFrameBasis, CHECKPOINT_DIRTY_FRAME_RECORD_BYTES};
pub use encoded_digest::checkpoint_stream_encoded_digest;
pub use footer::{CheckpointStreamFooter, CHECKPOINT_STREAM_FOOTER_RECORD_BYTES};
pub use identity::PhysicalCheckpointIdentity;
#[cfg(test)]
pub(crate) use inspection::{inspect_checkpoint_stream, VerifiedCheckpointStream};
pub use record::CheckpointStreamDecodeDenial;
pub use selective_aggregate::{
    CheckpointSelectiveRecordAggregate, CheckpointSelectiveRecordSummary,
};
pub use source::{
    CheckpointRootBasis, CheckpointWalSourceRange, PhysicalCheckpointSecurityBinding,
    PhysicalCheckpointSource, CHECKPOINT_STREAM_HEADER_RECORD_BYTES,
};
#[cfg(test)]
pub(crate) use stream::{CheckpointBindingCompactionDecoder, CheckpointStreamDecoder};
pub use stream::{CheckpointBindingCompactionEncoder, CheckpointStreamEncoder};

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
