use std::path::{Path, PathBuf};

use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StableStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
use worth_store_physical_format::{
    durable_artifact_checksum, DurablePhysicalRootManifest, DurableRootSelector,
    PhysicalFreeSpaceMembershipBlock, PhysicalRecordFormatDeclaration, RecordAllocationClass,
    RecordArtifactFile, RecordFreeSpaceManifestEntry, RootSelectorIdentity, RootSelectorRole,
};

use super::clean_artifact_manifest::RootArtifactManifestDeclaration;
use super::{
    CleanRootArtifactManifest, RootArtifactIdentity, RootArtifactManifestDenial, RootArtifactRole,
};

const CURRENT_ROOT_GENERATION: u64 = 8;
const PREVIOUS_ROOT_GENERATION: u64 = 7;
const CURRENT_SELECTOR_IDENTITY: u64 = 19;
const PREVIOUS_SELECTOR_IDENTITY: u64 = 17;

#[derive(Debug)]
pub(crate) enum RootFixtureMaterializationDenial {
    Write,
    Manifest(RootArtifactManifestDenial),
}

impl std::fmt::Display for RootFixtureMaterializationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Write => formatter.write_str("could not write canonical root baseline"),
            Self::Manifest(denial) => write!(formatter, "clean root manifest rejected: {denial:?}"),
        }
    }
}

pub(crate) fn materialize_canonical_root_fixture(
    store_root: &Path,
) -> Result<CleanRootArtifactManifest, RootFixtureMaterializationDenial> {
    let format = PhysicalRecordFormatDeclaration::builder()
        .admit()
        .expect("the current default record format is admitted");
    let store = stable_store([7; 16]);
    let donor_store = stable_store([9; 16]);
    let current_selector = selector(
        store,
        format,
        CURRENT_SELECTOR_IDENTITY,
        RootSelectorRole::Current,
        CURRENT_ROOT_GENERATION,
        Some((PREVIOUS_SELECTOR_IDENTITY, PREVIOUS_ROOT_GENERATION)),
    );
    let previous_selector = selector(
        store,
        format,
        PREVIOUS_SELECTOR_IDENTITY,
        RootSelectorRole::Previous,
        PREVIOUS_ROOT_GENERATION,
        None,
    );
    let donor_current_selector = selector(
        donor_store,
        format,
        CURRENT_SELECTOR_IDENTITY + 10,
        RootSelectorRole::Current,
        CURRENT_ROOT_GENERATION,
        Some((PREVIOUS_SELECTOR_IDENTITY + 10, PREVIOUS_ROOT_GENERATION)),
    );
    let donor_previous_selector = selector(
        donor_store,
        format,
        PREVIOUS_SELECTOR_IDENTITY + 10,
        RootSelectorRole::Previous,
        PREVIOUS_ROOT_GENERATION,
        None,
    );
    let (current_root, current_free_space) = root_world(CURRENT_ROOT_GENERATION, format);
    let (previous_root, previous_free_space) = root_world(PREVIOUS_ROOT_GENERATION, format);

    let current_path = record_path(RecordArtifactFile::CurrentRootSelector);
    let previous_path = record_path(RecordArtifactFile::PreviousRootSelector);
    let current_root_path = root_path(CURRENT_ROOT_GENERATION);
    let previous_root_path = root_path(PREVIOUS_ROOT_GENERATION);
    let current_free_space_path = free_space_path(CURRENT_ROOT_GENERATION);
    let previous_free_space_path = free_space_path(PREVIOUS_ROOT_GENERATION);
    let donor_current_path = PathBuf::from("substitution-sources/root-current.selector");
    let donor_previous_path = PathBuf::from("substitution-sources/root-previous.selector");

    write(store_root, &current_path, &current_selector.encode())?;
    write(store_root, &previous_path, &previous_selector.encode())?;
    write(
        store_root,
        &donor_current_path,
        &donor_current_selector.encode(),
    )?;
    write(
        store_root,
        &donor_previous_path,
        &donor_previous_selector.encode(),
    )?;
    write(store_root, &current_root_path, &current_root.encode(format))?;
    write(
        store_root,
        &previous_root_path,
        &previous_root.encode(format),
    )?;
    write(store_root, &current_free_space_path, &current_free_space)?;
    write(store_root, &previous_free_space_path, &previous_free_space)?;

    let declarations = vec![
        RootArtifactManifestDeclaration {
            identity: RootArtifactIdentity::new(
                RootArtifactRole::CurrentSelector,
                store.bytes(),
                CURRENT_SELECTOR_IDENTITY,
                CURRENT_ROOT_GENERATION,
            ),
            relative_path: current_path,
            substitution_source_path: donor_current_path,
            substitution_source_identity: RootArtifactIdentity::new(
                RootArtifactRole::CurrentSelector,
                donor_store.bytes(),
                CURRENT_SELECTOR_IDENTITY + 10,
                CURRENT_ROOT_GENERATION,
            ),
            duplicate_path: record_path(RecordArtifactFile::RootSelectorCandidate {
                role: RootSelectorRole::Current,
                publication: 101,
            }),
            covered_edit_offset: 65,
            pointer_range: 65..73,
            expected_reachable_paths: vec![
                current_root_path.clone(),
                current_free_space_path.clone(),
            ],
        },
        RootArtifactManifestDeclaration {
            identity: RootArtifactIdentity::new(
                RootArtifactRole::PreviousSelector,
                store.bytes(),
                PREVIOUS_SELECTOR_IDENTITY,
                PREVIOUS_ROOT_GENERATION,
            ),
            relative_path: previous_path,
            substitution_source_path: donor_previous_path,
            substitution_source_identity: RootArtifactIdentity::new(
                RootArtifactRole::PreviousSelector,
                donor_store.bytes(),
                PREVIOUS_SELECTOR_IDENTITY + 10,
                PREVIOUS_ROOT_GENERATION,
            ),
            duplicate_path: record_path(RecordArtifactFile::RootSelectorCandidate {
                role: RootSelectorRole::Previous,
                publication: 102,
            }),
            covered_edit_offset: 65,
            pointer_range: 65..73,
            expected_reachable_paths: vec![
                previous_root_path.clone(),
                previous_free_space_path.clone(),
            ],
        },
        RootArtifactManifestDeclaration {
            identity: RootArtifactIdentity::new(
                RootArtifactRole::AddressedRootManifest,
                store.bytes(),
                CURRENT_ROOT_GENERATION,
                CURRENT_ROOT_GENERATION,
            ),
            relative_path: current_root_path,
            substitution_source_path: previous_root_path,
            substitution_source_identity: RootArtifactIdentity::new(
                RootArtifactRole::AddressedRootManifest,
                store.bytes(),
                PREVIOUS_ROOT_GENERATION,
                PREVIOUS_ROOT_GENERATION,
            ),
            duplicate_path: root_path(109),
            covered_edit_offset: 56,
            pointer_range: 296..304,
            expected_reachable_paths: vec![current_free_space_path.clone()],
        },
    ];
    CleanRootArtifactManifest::observe(
        store_root,
        declarations,
        vec![current_free_space_path, previous_free_space_path],
    )
    .map_err(RootFixtureMaterializationDenial::Manifest)
}

fn selector(
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    identity: u64,
    role: RootSelectorRole,
    root_generation: u64,
    linked: Option<(u64, u64)>,
) -> DurableRootSelector {
    DurableRootSelector::new(
        store,
        format,
        RootSelectorIdentity::new(identity).unwrap(),
        role,
        root_generation,
        linked.and_then(|(selector, _)| RootSelectorIdentity::new(selector)),
        linked.map(|(_, generation)| generation),
    )
    .expect("the focused selector world has valid linkage")
}

fn root_world(
    generation: u64,
    format: PhysicalRecordFormatDeclaration,
) -> (DurablePhysicalRootManifest, Vec<u8>) {
    let entry =
        RecordFreeSpaceManifestEntry::new(RecordAllocationClass::InlinePage, 1, 2, 3, generation)
            .unwrap();
    let block =
        PhysicalFreeSpaceMembershipBlock::leaf(generation + 200, generation, 1, vec![entry], 4)
            .unwrap();
    let block_bytes = block.encode(format);
    let free_space = block.reference(durable_artifact_checksum(&block_bytes));
    let manifest = DurablePhysicalRootManifest::builder(generation, generation + 100, 4, 0x5678)
        .free_space_root(Some(free_space))
        .admit()
        .unwrap_or_else(|| panic!("root generation {generation} must admit under {format:?}"));
    (manifest, block_bytes)
}

fn stable_store(bytes: [u8; 16]) -> StableStoreIdentity {
    StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes(bytes).unwrap(),
    )
    .published_identity()
}

fn record_path(artifact: RecordArtifactFile) -> PathBuf {
    PathBuf::from("records").join(artifact.file_name())
}

fn root_path(generation: u64) -> PathBuf {
    PathBuf::from("records/roots").join(RecordArtifactFile::RootManifest { generation }.file_name())
}

fn free_space_path(generation: u64) -> PathBuf {
    PathBuf::from("records/free-space").join(
        RecordArtifactFile::FreeSpaceMembershipBlock {
            generation,
            block: 1,
        }
        .file_name(),
    )
}

fn write(
    root: &Path,
    relative: &Path,
    bytes: &[u8],
) -> Result<(), RootFixtureMaterializationDenial> {
    let path = root.join(relative);
    let parent = path
        .parent()
        .ok_or(RootFixtureMaterializationDenial::Write)?;
    std::fs::create_dir_all(parent).map_err(|_| RootFixtureMaterializationDenial::Write)?;
    std::fs::write(path, bytes).map_err(|_| RootFixtureMaterializationDenial::Write)
}
