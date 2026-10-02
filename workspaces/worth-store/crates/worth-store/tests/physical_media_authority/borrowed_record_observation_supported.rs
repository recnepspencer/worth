use worth_store_physical_backend::{ArtifactTreeReadAllocator, QualifiedFilesystemMedia};
use worth_store_physical_format::RecordArtifactFile;

fn addressed_read<S: ArtifactTreeReadAllocator>(media: &QualifiedFilesystemMedia, storage: &mut S) {
    let mut observation = media.bounded_record_observation(4, 4096).unwrap();
    assert_eq!(observation.store_identity(), media.store_identity());
    let address = RecordArtifactFile::ReleaseCustodyHeadBlock {
        generation: 7,
        block: 2,
    };
    let _ = observation.read_record_artifact_with_storage(address, 4096, storage);
    let _ = observation.read_record_artifact_range_with_storage(address, 0, 32, 4096, storage);
}

fn main() {}
