//! Test-owned read of the actual checkpoint-source head tree. The tag-7 V2
//! digest is a comparison target, not a substitute for these rooted bytes.

pub use {std::fs, std::path::Path};

pub use {sha2::Digest, sha2::Sha256};
pub use {
    worth_store_physical_format::DurablePhysicalRootManifest,
    worth_store_physical_format::PhysicalCheckpointSource,
    worth_store_physical_format::RecordArtifactFile,
    worth_store_physical_format::ReleaseCheckpointAccumulatorV2,
    worth_store_physical_format::ReleaseCustodyHeadEntryV1,
    worth_store_physical_format::CHECKPOINT_STREAM_HEADER_RECORD_BYTES,
};
pub use {
    worth_store_physical_integrity::walk_release_custody_head,
    worth_store_physical_integrity::ReleaseCustodyHeadWalkLimitsV1,
};

pub fn selected_heads(
    store_root: &Path,
    accumulator: ReleaseCheckpointAccumulatorV2,
) -> Vec<ReleaseCustodyHeadEntryV1> {
    let checkpoint = std::fs::read(store_root.join("families/checkpoint.current")).unwrap();
    let source =
        worth_store_physical_format::PhysicalCheckpointSource::decode_stream_header_record(
            &checkpoint[..CHECKPOINT_STREAM_HEADER_RECORD_BYTES],
        )
        .unwrap();
    assert_eq!(source.identity(), accumulator.base().checkpoint());
    let records = store_root.join("families/records");
    let root_path = records.join("roots").join(
        worth_store_physical_format::RecordArtifactFile::RootManifest {
            generation: source.root().generation(),
        }
        .file_name(),
    );
    let root_bytes = std::fs::read(root_path).unwrap();
    assert_eq!(
        <[u8; 32]>::from(sha2::Sha256::digest(&root_bytes)),
        accumulator.base().root_sha256(),
    );
    let (manifest, format) =
        worth_store_physical_format::DurablePhysicalRootManifest::decode(&root_bytes, u16::MAX)
            .unwrap();
    assert_eq!(manifest.generation(), source.root().generation());
    assert_eq!(manifest.tree_identity(), source.root().tree_identity());
    let expected = accumulator.head_count();
    assert!(expected <= 1024, "fixture head walk must remain bounded");
    let max_nodes = expected.saturating_mul(2).saturating_add(16);
    let limits = worth_store_physical_integrity::ReleaseCustodyHeadWalkLimitsV1::new(
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
                worth_store_physical_format::RecordArtifactFile::ReleaseCustodyHeadBlock {
                    generation: reference.generation(),
                    block: reference.block(),
                }
                .file_name(),
            );
            let bytes = std::fs::read(path)?;
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
