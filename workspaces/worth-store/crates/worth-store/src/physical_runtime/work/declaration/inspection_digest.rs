use sha2::{Digest, Sha256};
use worth_store_physical_format::{PhysicalArtifactReadRange, PhysicalArtifactReadTarget};

pub(super) fn include(digest: &mut Sha256, range: PhysicalArtifactReadRange) {
    digest.update(b"inspection");
    match range.target() {
        PhysicalArtifactReadTarget::Record(artifact) => {
            digest.update([1]);
            let name = artifact.file_name();
            digest.update((name.len() as u64).to_le_bytes());
            digest.update(name.as_bytes());
        }
        PhysicalArtifactReadTarget::Wal(identity) => {
            digest.update([2]);
            digest.update(identity.segment().get().to_le_bytes());
            digest.update(identity.generation().get().to_le_bytes());
        }
        PhysicalArtifactReadTarget::Checkpoint(identity) => {
            digest.update([3]);
            digest.update(identity.store_identity().bytes());
            digest.update(identity.sequence().get().to_le_bytes());
        }
        PhysicalArtifactReadTarget::PhysicalWork(identity) => {
            digest.update([4]);
            digest.update(identity.runtime().get().to_le_bytes());
            digest.update(identity.generation().get().to_le_bytes());
            digest.update(identity.operation().get().to_le_bytes());
        }
    }
    digest.update(range.offset().to_le_bytes());
    digest.update(range.length().to_le_bytes());
}
