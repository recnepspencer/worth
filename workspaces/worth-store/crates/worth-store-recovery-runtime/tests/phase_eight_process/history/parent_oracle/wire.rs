#[path = "wire/checkpoint.rs"]
mod checkpoint;
#[path = "wire/wal.rs"]
mod wal;

pub(super) fn observe_checkpoint(bytes: &[u8]) -> Option<super::CheckpointFacts> {
    checkpoint::observe_checkpoint(bytes)
}

pub(super) fn validated_checkpoint_binding_payloads(bytes: &[u8]) -> Option<Vec<&[u8]>> {
    checkpoint::validated_binding_payloads(bytes)
}

pub(super) fn observe_wal(bytes: &[u8]) -> super::ArtifactFacts {
    wal::observe_wal(bytes)
}
