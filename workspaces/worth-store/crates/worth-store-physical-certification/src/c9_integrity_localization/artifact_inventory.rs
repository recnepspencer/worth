//! Closed, production-issued fixture facts only. No integrity validator or damaged-byte
//! decoder is used to derive an expected outcome; canonical decoders are confined to
//! reading this immutable lawful baseline's published child coordinates.
use super::ClosedStoreProcessManifest;
use std::path::{Path, PathBuf};
use worth_store::physical_runtime::PhysicalIntegrityScrubTarget;
use worth_store_physical_format::{
    store_namespace::{
        ProposedStoreIdentity, StableStoreIdentity, StoreNamespaceIdentityRecord,
        StoreNamespaceVersion,
    },
    DurablePhysicalRootManifest, PhysicalArtifactReadTarget, PhysicalRecordFormatDeclaration,
    RecordArtifactFile,
};
use worth_store_physical_integrity::{PhysicalArtifactScope, PhysicalByteRange};

mod extents;
mod streams;
mod trees;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FrameGrammar {
    Common,
    Wal,
    Checkpoint,
}

#[derive(Debug, Clone)]
pub(super) struct ArtifactGranule {
    pub(super) path: PathBuf,
    pub(super) family: &'static str,
    pub(super) scope: PhysicalArtifactScope,
    pub(super) target: PhysicalArtifactReadTarget,
    pub(super) grammar: FrameGrammar,
}

impl ArtifactGranule {
    pub(super) fn scrub_target(&self) -> PhysicalIntegrityScrubTarget {
        PhysicalIntegrityScrubTarget::new(self.target, self.scope).unwrap()
    }
    pub(super) fn offset(&self) -> usize {
        self.scope.byte_range().offset() as usize
    }
    pub(super) fn length(&self) -> usize {
        self.scope.byte_range().length() as usize
    }
}

pub(super) struct ArtifactInventory {
    pub(super) granules: Vec<ArtifactGranule>,
    pub(super) store: StableStoreIdentity,
    pub(super) format: PhysicalRecordFormatDeclaration,
    root: PathBuf,
    manifest: ClosedStoreProcessManifest,
}

impl ArtifactInventory {
    pub(super) fn observe(root: &Path) -> Self {
        let manifest = ClosedStoreProcessManifest::observe(root).unwrap();
        let store = StoreNamespaceIdentityRecord::new(
            StoreNamespaceVersion::CURRENT,
            ProposedStoreIdentity::from_nonzero_bytes(manifest.store_identity()).unwrap(),
        )
        .published_identity();
        let selector = std::fs::read(root.join("families/records/root-current.selector")).unwrap();
        let current_generation = u64_at(&selector, 65);
        let root_file = RecordArtifactFile::RootManifest {
            generation: current_generation,
        };
        let root_path = manifest
            .paths()
            .find(|path| path.file_name().unwrap() == root_file.file_name().as_str())
            .unwrap();
        let (current, format) = DurablePhysicalRootManifest::decode(
            &std::fs::read(root.join(root_path)).unwrap(),
            u16::MAX,
        )
        .unwrap();
        let mut inventory = Self {
            granules: Vec::new(),
            store,
            format,
            root: root.to_owned(),
            manifest,
        };
        for (file, family) in [
            (RecordArtifactFile::BootstrapCatalog, "bootstrap_catalog"),
            (
                RecordArtifactFile::CurrentRootSelector,
                "current_root_selector",
            ),
            (
                RecordArtifactFile::PreviousRootSelector,
                "previous_root_selector",
            ),
        ] {
            let range = inventory.record_range(file);
            let scope = match file {
                RecordArtifactFile::BootstrapCatalog => {
                    PhysicalArtifactScope::bootstrap_catalog(store, format, range)
                }
                RecordArtifactFile::CurrentRootSelector => {
                    PhysicalArtifactScope::current_root_selector(store, format, range)
                }
                RecordArtifactFile::PreviousRootSelector => {
                    PhysicalArtifactScope::previous_root_selector(store, format, range)
                }
                _ => unreachable!(),
            };
            inventory.push_record(file, family, scope);
        }
        inventory.root_manifest(current);
        let previous = std::fs::read(root.join("families/records/root-previous.selector")).unwrap();
        let file = RecordArtifactFile::RootManifest {
            generation: u64_at(&previous, 65),
        };
        let (previous, _) =
            DurablePhysicalRootManifest::decode(&inventory.record_bytes(file), u16::MAX).unwrap();
        inventory.root_manifest(previous);
        streams::collect(&mut inventory);
        inventory
    }

    fn root_manifest(&mut self, root: DurablePhysicalRootManifest) {
        let file = RecordArtifactFile::RootManifest {
            generation: root.generation(),
        };
        self.push_record(
            file,
            "root_manifest",
            PhysicalArtifactScope::root_manifest(
                self.store,
                self.format,
                root.generation(),
                self.record_range(file),
            )
            .unwrap(),
        );
        trees::collect(self, &root);
    }

    fn record_path(&self, file: RecordArtifactFile) -> PathBuf {
        let name = file.file_name();
        self.manifest
            .paths()
            .find(|path| path.file_name().unwrap() == name.as_str())
            .unwrap()
            .to_owned()
    }
    fn record_bytes(&self, file: RecordArtifactFile) -> Vec<u8> {
        std::fs::read(self.root.join(self.record_path(file))).unwrap()
    }
    fn record_range(&self, file: RecordArtifactFile) -> PhysicalByteRange {
        PhysicalByteRange::new(0, self.record_bytes(file).len() as u64).unwrap()
    }
    fn push_record(
        &mut self,
        file: RecordArtifactFile,
        family: &'static str,
        scope: PhysicalArtifactScope,
    ) {
        let path = self.record_path(file);
        self.push(ArtifactGranule {
            path,
            family,
            scope,
            target: PhysicalArtifactReadTarget::Record(file),
            grammar: FrameGrammar::Common,
        });
    }
    fn push(&mut self, granule: ArtifactGranule) {
        if !self.granules.iter().any(|old| {
            old.path == granule.path && old.scope.byte_range() == granule.scope.byte_range()
        }) {
            self.granules.push(granule);
        }
    }
}

fn u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}
fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}
