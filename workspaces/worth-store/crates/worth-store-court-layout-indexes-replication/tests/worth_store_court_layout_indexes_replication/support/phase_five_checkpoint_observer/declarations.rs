pub(crate) use std::num::NonZeroU64;

pub(crate) use {
    worth_store_offline_verifier::observe_recovery_artifacts,
    worth_store_offline_verifier::RecoveryObserverLimits,
    worth_store_offline_verifier::RecoveryObserverReport,
};
pub(crate) use {
    worth_store_physical_format::store_namespace::ProposedStoreIdentity,
    worth_store_physical_format::store_namespace::StoreNamespaceIdentityRecord,
    worth_store_physical_format::store_namespace::StoreNamespaceVersion,
};
pub(crate) use {
    worth_store_physical_format::CheckpointBindingCompactionHeader,
    worth_store_physical_format::CheckpointDirtyFrameBasis,
    worth_store_physical_format::CheckpointRootBasis,
    worth_store_physical_format::CheckpointStreamEncoder,
    worth_store_physical_format::CheckpointWalSourceRange,
    worth_store_physical_format::PhysicalCheckpointIdentity,
    worth_store_physical_format::PhysicalCheckpointSource,
    worth_store_physical_format::RecordArtifactFile,
    worth_store_physical_format::RecordFrameCoordinate,
};

pub(crate) fn assert_residue(fixture: CheckpointFixture) {
    let root = write_checkpoint(&fixture.bytes);
    let report = observe(root.path(), fixture.bytes.len());
    assert_eq!(report.artifact_count(), 1);
    assert_eq!(report.bytes_read(), fixture.bytes.len() as u64);
    assert_eq!(report.checkpoint_count(), 0);
    assert_eq!(report.checkpoint_page_count(), 0);
    assert_eq!(report.residue_artifact_count(), 1);
    assert_eq!(report.residue_bytes(), fixture.bytes.len() as u64);
}

pub(crate) fn observe(root: &std::path::Path, artifact_bytes: usize) -> RecoveryObserverReport {
    observe_recovery_artifacts(
        root,
        worth_store_offline_verifier::RecoveryObserverLimits::new(2, 2, 1, artifact_bytes as u64)
            .unwrap(),
    )
    .unwrap()
}

pub(crate) struct CheckpointFixture {
    pub(crate) bytes: Vec<u8>,
    pub(crate) dirty_offset: usize,
    pub(crate) dirty_bytes: usize,
    pub(crate) binding_offset: usize,
    pub(crate) binding_bytes: usize,
}

pub(crate) fn checkpoint_fixture(variant: u8) -> CheckpointFixture {
    let proposed =
        worth_store_physical_format::store_namespace::ProposedStoreIdentity::from_nonzero_bytes(
            [7; 16],
        )
        .unwrap();
    let store = worth_store_physical_format::store_namespace::StoreNamespaceIdentityRecord::new(
        worth_store_physical_format::store_namespace::StoreNamespaceVersion::CURRENT,
        proposed,
    )
    .published_identity();
    let identity = worth_store_physical_format::PhysicalCheckpointIdentity::new(
        store,
        std::num::NonZeroU64::new(3).unwrap(),
    );
    let source = worth_store_physical_format::PhysicalCheckpointSource::concurrent(
        identity,
        worth_store_physical_format::CheckpointWalSourceRange::new(11, 29).unwrap(),
        worth_store_physical_format::CheckpointRootBasis::new(5, 71),
        43,
    );
    let (mut encoder, header) = worth_store_physical_format::CheckpointStreamEncoder::begin(source);
    let coordinate = worth_store_physical_format::RecordFrameCoordinate::new(
        worth_store_physical_format::RecordArtifactFile::Segment {
            segment: 2,
            generation: 9,
        },
        128,
        64,
    )
    .unwrap();
    let dirty =
        encoder.encode_dirty_basis(worth_store_physical_format::CheckpointDirtyFrameBasis::new(
            coordinate,
            17 + u64::from(variant),
        ));
    let (mut compaction, compaction_header) = encoder.begin_binding_compaction(
        worth_store_physical_format::CheckpointBindingCompactionHeader::new(4, 29).unwrap(),
    );
    let mut binding_payload = *b"observer-binding-a";
    binding_payload[17] = b'a' + variant;
    let binding = compaction.encode_binding_record(&binding_payload).unwrap();
    let (_, footer) = compaction.finish();
    let dirty_offset = header.len();
    let binding_offset = dirty_offset + dirty.len() + compaction_header.len();
    let mut bytes = header;
    bytes.extend_from_slice(&dirty);
    bytes.extend_from_slice(&compaction_header);
    bytes.extend_from_slice(&binding);
    bytes.extend_from_slice(&footer);
    CheckpointFixture {
        bytes,
        dirty_offset,
        dirty_bytes: dirty.len(),
        binding_offset,
        binding_bytes: binding.len(),
    }
}

pub(crate) fn write_checkpoint(bytes: &[u8]) -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    let families = root.path().join("families");
    std::fs::create_dir(&families).unwrap();
    std::fs::write(families.join("checkpoint.current"), bytes).unwrap();
    root
}
