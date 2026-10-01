//! Independent persisted-media witness for the ordinary ingest root step
//! between two V3 WAL descriptors under one NoRelease checkpoint.

use std::{fs, path::Path};

use worth_store_physical_format::{
    durable_artifact_checksum, BlobRecordKind, CurrentPhysicalRecordPlacement,
    DurablePhysicalRootManifest, DurableRootSelector, PhysicalRootRoutingBlock, RecordArtifactFile,
    SelectedRecordContentClass,
};

pub(super) struct Snapshot {
    generation: u64,
    routes: Vec<CurrentPhysicalRecordPlacement>,
}

pub(super) fn read(root: &Path) -> Snapshot {
    let records = root.join("families/records");
    let selector = DurableRootSelector::decode(
        &fs::read(records.join(RecordArtifactFile::CurrentRootSelector.file_name())).unwrap(),
    )
    .unwrap();
    let manifest_path = records.join("roots").join(
        RecordArtifactFile::RootManifest {
            generation: selector.root_generation(),
        }
        .file_name(),
    );
    let (manifest, format) =
        DurablePhysicalRootManifest::decode(&fs::read(manifest_path).unwrap(), u16::MAX).unwrap();
    assert_eq!(format, selector.format());
    let mut pending = manifest.routing_root().into_iter().collect::<Vec<_>>();
    let mut routes = Vec::new();
    while let Some(reference) = pending.pop() {
        let path = records.join("roots").join(
            RecordArtifactFile::RootRoutingBlock {
                generation: reference.generation(),
                block: reference.block(),
            }
            .file_name(),
        );
        let bytes = fs::read(path).unwrap();
        let (block, observed_format) =
            PhysicalRootRoutingBlock::decode(&bytes, manifest.node_capacity()).unwrap();
        assert_eq!(observed_format, format);
        assert_eq!(
            block.reference(durable_artifact_checksum(&bytes)),
            reference
        );
        match block {
            PhysicalRootRoutingBlock::Leaf { entries, .. } => routes.extend(entries),
            PhysicalRootRoutingBlock::Branch { children, .. } => pending.extend(children),
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

pub(super) fn assert_distinct_ingest_step(before: &Snapshot, after: &Snapshot) {
    assert!(
        after.generation > before.generation,
        "ordinary ingest must publish a new root"
    );
    let prior_v3 = before
        .routes
        .iter()
        .filter(|route| {
            route.content_class()
                == SelectedRecordContentClass::Blob(BlobRecordKind::ReclaimDescriptorV3)
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
                == SelectedRecordContentClass::Blob(BlobRecordKind::GenerationPublished)
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
