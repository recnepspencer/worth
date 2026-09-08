use std::num::NonZeroU64;

use worth_store_physical_format::{
    store_namespace::{
        ProposedStoreIdentity, StableStoreIdentity, StoreNamespaceIdentityRecord,
        StoreNamespaceVersion,
    },
    CheckpointRootBasis, CheckpointWalSourceRange, DurablePhysicalRootManifest,
    DurableRootSelector, FreeSpaceBlockReference, FreeSpaceKey, PhysicalCheckpointIdentity,
    PhysicalCheckpointSource, PhysicalPageSizeClass, PhysicalRecordFormatDeclaration,
    RecordAllocationClass, RootSelectorIdentity, RootSelectorRole,
};
use worth_store_physical_integrity::{
    validate_root_manifest, PhysicalArtifactScope, PhysicalByteRange,
    RootManifestIntegrityValidation, UntrustedPhysicalArtifact,
};

use super::*;

#[test]
fn exact_older_root_remains_checkpoint_provenance_without_becoming_selected() {
    let selected = candidate(9, 7);
    let source = checkpoint_source(store(1), 4, 7);
    let encoded = manifest(4, 7).encode(format(PhysicalPageSizeClass::KiB16));
    let validated = validate(&encoded, store(1), 4, PhysicalPageSizeClass::KiB16);
    assert_eq!(require_source_root(&selected, source, &validated), Ok(()));
    assert_eq!(selected.manifest().generation(), 9);
}

#[test]
fn valid_bytes_cannot_substitute_store_generation_tree_or_future_root() {
    let selected = candidate(9, 7);
    for (
        source_store,
        source_generation,
        source_tree,
        actual_store,
        actual_generation,
        actual_tree,
        expected,
    ) in [
        (2, 4, 7, 1, 4, 7, PhysicalCheckpointBaseDenial::ForeignStore),
        (1, 4, 7, 2, 4, 7, PhysicalCheckpointBaseDenial::ForeignStore),
        (
            1,
            4,
            7,
            1,
            5,
            7,
            PhysicalCheckpointBaseDenial::RootGenerationMismatch,
        ),
        (
            1,
            10,
            7,
            1,
            10,
            7,
            PhysicalCheckpointBaseDenial::RootGenerationMismatch,
        ),
        (
            1,
            4,
            7,
            1,
            4,
            8,
            PhysicalCheckpointBaseDenial::RootTreeMismatch,
        ),
        (
            1,
            4,
            8,
            1,
            4,
            8,
            PhysicalCheckpointBaseDenial::RootTreeMismatch,
        ),
    ] {
        let encoded =
            manifest(actual_generation, actual_tree).encode(format(PhysicalPageSizeClass::KiB16));
        let validated = validate(
            &encoded,
            store(actual_store),
            actual_generation,
            PhysicalPageSizeClass::KiB16,
        );
        assert_eq!(
            require_source_root(
                &selected,
                checkpoint_source(store(source_store), source_generation, source_tree),
                &validated,
            ),
            Err(expected)
        );
    }
}

#[test]
fn valid_source_root_in_another_record_format_is_not_a_checkpoint_base() {
    let encoded = manifest(4, 7).encode(format(PhysicalPageSizeClass::KiB32));
    let validated = validate(&encoded, store(1), 4, PhysicalPageSizeClass::KiB32);
    assert_eq!(
        require_source_root(
            &candidate(9, 7),
            checkpoint_source(store(1), 4, 7),
            &validated,
        ),
        Err(PhysicalCheckpointBaseDenial::RootFormatMismatch)
    );
}

fn candidate(generation: u64, tree: u64) -> super::super::PhysicalRootSourceCandidate {
    let format = format(PhysicalPageSizeClass::KiB16);
    let selector = DurableRootSelector::new(
        store(1),
        format,
        RootSelectorIdentity::new(1).unwrap(),
        RootSelectorRole::Current,
        generation,
        None,
        None,
    )
    .unwrap();
    super::super::PhysicalRootSourceCandidate::from_structured_observation(
        selector,
        manifest(generation, tree),
        format,
    )
    .unwrap()
}

fn manifest(generation: u64, tree: u64) -> DurablePhysicalRootManifest {
    let key = FreeSpaceKey::new(RecordAllocationClass::Extent, 1).unwrap();
    let free = FreeSpaceBlockReference::new(generation, 1, 0, 17, key, key).unwrap();
    DurablePhysicalRootManifest::builder(generation, tree, 4, 19)
        .free_space_root(Some(free))
        .admit()
        .unwrap()
}

fn validate(
    bytes: &[u8],
    store: StableStoreIdentity,
    generation: u64,
    size: PhysicalPageSizeClass,
) -> IntegrityValidatedRootManifest<'_> {
    let scope = PhysicalArtifactScope::root_manifest(
        store,
        format(size),
        generation,
        PhysicalByteRange::new(0, bytes.len() as u64).unwrap(),
    )
    .unwrap();
    let RootManifestIntegrityValidation::Intact(root) =
        validate_root_manifest(UntrustedPhysicalArtifact::from_bounded_bytes(bytes), scope).0
    else {
        panic!("canonical source root must validate")
    };
    root
}

fn checkpoint_source(
    store: StableStoreIdentity,
    generation: u64,
    tree: u64,
) -> PhysicalCheckpointSource {
    PhysicalCheckpointSource::concurrent(
        PhysicalCheckpointIdentity::new(store, NonZeroU64::new(1).unwrap()),
        CheckpointWalSourceRange::new(10, 20).unwrap(),
        CheckpointRootBasis::new(generation, tree),
        1,
    )
}

fn format(size: PhysicalPageSizeClass) -> PhysicalRecordFormatDeclaration {
    PhysicalRecordFormatDeclaration::builder()
        .page_size(size)
        .admit()
        .unwrap()
}

fn store(marker: u8) -> StableStoreIdentity {
    StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([marker; 16]).unwrap(),
    )
    .published_identity()
}
