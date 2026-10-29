//! Independent persisted-media witness for the ordinary ingest root step
//! between two V3 WAL descriptors under one NoRelease checkpoint.

pub use {std::fs, std::path::Path};

pub use {worth_store_physical_format::durable_artifact_checksum, worth_store_physical_format::BlobRecordKind, worth_store_physical_format::CurrentPhysicalRecordPlacement, worth_store_physical_format::DurablePhysicalRootManifest, worth_store_physical_format::DurableRootSelector, worth_store_physical_format::ExtentArenaRange, worth_store_physical_format::PhysicalRootRoutingBlock, worth_store_physical_format::RecordArtifactFile, worth_store_physical_format::SelectedRecordContentClass};

pub struct Snapshot {
    pub generation: u64,
    pub routes: Vec<CurrentPhysicalRecordPlacement>,
}

pub fn read(root: &Path) -> Snapshot {
    let records = root.join("families/records");
    let selector = worth_store_physical_format::DurableRootSelector::decode(
        &std::fs::read(records.join(worth_store_physical_format::RecordArtifactFile::CurrentRootSelector.file_name())).unwrap(),
    )
    .unwrap();
    let manifest_path = records.join("roots").join(
        worth_store_physical_format::RecordArtifactFile::RootManifest {
            generation: selector.root_generation(),
        }
        .file_name(),
    );
    let (manifest, format) =
        worth_store_physical_format::DurablePhysicalRootManifest::decode(&std::fs::read(manifest_path).unwrap(), u16::MAX).unwrap();
    assert_eq!(format, selector.format());
    let mut pending = manifest.routing_root().into_iter().collect::<Vec<_>>();
    let mut routes = Vec::new();
    while let Some(reference) = pending.pop() {
        let path = records.join("roots").join(
            worth_store_physical_format::RecordArtifactFile::RootRoutingBlock {
                generation: reference.generation(),
                block: reference.block(),
            }
            .file_name(),
        );
        let bytes = std::fs::read(path).unwrap();
        let (block, observed_format) =
            worth_store_physical_format::PhysicalRootRoutingBlock::decode(&bytes, manifest.node_capacity()).unwrap();
        assert_eq!(observed_format, format);
        assert_eq!(
            block.reference(durable_artifact_checksum(&bytes)),
            reference
        );
        match block {
            worth_store_physical_format::PhysicalRootRoutingBlock::Leaf { entries, .. } => routes.extend(entries),
            worth_store_physical_format::PhysicalRootRoutingBlock::Branch { children, .. } => pending.extend(children),
        }
    }
    routes.sort_unstable_by_key(|route| route.record());
    assert_eq!(routes.len(), manifest.record_count() as usize);
    assert!(routes
        .windows(2)
        .all(|pair| pair[0].record() != pair[1].record()));
    Snapshot {
        generation: manifest.generation(),
        routes,
    }
}

pub fn selected_session_declared_extent(root: &Path) -> ExtentArenaRange {
    let snapshot = read(root);
    let selected = snapshot.routes.into_iter().find(|route| {
        route.content_class() == worth_store_physical_format::SelectedRecordContentClass::Blob(worth_store_physical_format::BlobRecordKind::SessionDeclared)
    });
    let worth_store_physical_format::CurrentPhysicalRecordPlacement::Extent(extent) =
        selected.expect("real selected SessionDeclared control")
    else {
        panic!("selected SessionDeclared must have an actual extent placement")
    };
    extent.arena_range()
}

pub fn assert_distinct_ingest_step(before: &Snapshot, after: &Snapshot) {
    assert!(
        after.generation > before.generation,
        "ordinary ingest must publish a new root"
    );
    let prior_v3 = before
        .routes
        .iter()
        .filter(|route| {
            route.content_class()
                == worth_store_physical_format::SelectedRecordContentClass::Blob(worth_store_physical_format::BlobRecordKind::ReclaimDescriptorV3)
        })
        .collect::<Vec<_>>();
    assert!(
        !prior_v3.is_empty(),
        "a prior C8 V3 result must be selected"
    );
    for descriptor in prior_v3 {
        assert!(
            after
                .routes
                .iter()
                .any(|route| route.record() == descriptor.record()),
            "ordinary ingest must preserve every prior V3 descriptor"
        );
    }
    let new_publications = after
        .routes
        .iter()
        .filter(|route| {
            route.content_class()
                == worth_store_physical_format::SelectedRecordContentClass::Blob(worth_store_physical_format::BlobRecordKind::GenerationPublished)
                && !before
                    .routes
                    .iter()
                    .any(|old| old.record() == route.record())
        })
        .count();
    assert_eq!(
        new_publications, 1,
        "the next object must be an ordinary selected ingest"
    );
}