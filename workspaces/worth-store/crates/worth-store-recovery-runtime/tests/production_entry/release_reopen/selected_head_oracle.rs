//! Test-owned read of the actual checkpoint-source head tree. The tag-7 V2
//! digest is a comparison target, not a substitute for these rooted bytes.

use std::{fs, path::Path};

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    DurablePhysicalRootManifest, PhysicalCheckpointSource, RecordArtifactFile,
    ReleaseCheckpointAccumulatorV2, ReleaseCustodyHeadEntryV1,
    CHECKPOINT_STREAM_HEADER_RECORD_BYTES,
};
use worth_store_physical_integrity::{walk_release_custody_head, ReleaseCustodyHeadWalkLimitsV1};

pub(crate) fn selected_heads(
    store_root: &Path,
    accumulator: ReleaseCheckpointAccumulatorV2,
) -> Vec<ReleaseCustodyHeadEntryV1> {
    let checkpoint = fs::read(store_root.join("families/checkpoint.current")).unwrap();
    let source = PhysicalCheckpointSource::decode_stream_header_record(
        &checkpoint[..CHECKPOINT_STREAM_HEADER_RECORD_BYTES],
    )
    .unwrap();
    assert_eq!(source.identity(), accumulator.base().checkpoint());
    let records = store_root.join("families/records");
    let root_path = records.join("roots").join(
        RecordArtifactFile::RootManifest {
            generation: source.root().generation(),
        }
        .file_name(),
    );
    let root_bytes = fs::read(root_path).unwrap();
    assert_eq!(
        <[u8; 32]>::from(Sha256::digest(&root_bytes)),
        accumulator.base().root_sha256(),
    );
    let (manifest, format) = DurablePhysicalRootManifest::decode(&root_bytes, u16::MAX).unwrap();
    assert_eq!(manifest.generation(), source.root().generation());
    assert_eq!(manifest.tree_identity(), source.root().tree_identity());
    let expected = accumulator.head_count();
    assert!(expected <= 1024, "fixture head walk must remain bounded");
    let max_nodes = expected.saturating_mul(2).saturating_add(16);
    let limits = ReleaseCustodyHeadWalkLimitsV1::new(
        max_nodes,
        expected,
        max_nodes * u64::from(format.page_size().bytes()),
        64 << 20,
        64,
    )
    .unwrap();
    let mut entries = Vec::new();
    let walk = walk_release_custody_head(
        &manifest,
        format,
        limits,
        |reference, remaining| {
            let path = records.join("roots").join(
                RecordArtifactFile::ReleaseCustodyHeadBlock {
                    generation: reference.generation(),
                    block: reference.block(),
                }
                .file_name(),
            );
            let bytes = fs::read(path)?;
            assert!(bytes.len() as u64 <= remaining);
            Ok::<_, std::io::Error>(bytes)
        },
        |entry| {
            entries.push(entry);
            Ok::<_, ()>(())
        },
    )
    .unwrap();
    assert_eq!(walk.entry_count(), expected);
    assert_eq!(walk.roster_digest(), accumulator.head_roster_digest());
    assert_eq!(entries.len() as u64, expected);
    entries
}
