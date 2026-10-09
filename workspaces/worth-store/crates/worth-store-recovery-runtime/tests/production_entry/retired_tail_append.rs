//! An ordinary append after an earlier release must not copy an unselected
//! physical tail slot into its new WAL projection.

use std::{fs, path::Path};

use super::*;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, ManifestEntryCapacity, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationOutcome, PhysicalMutationPreparationSuccess, PhysicalMutationRequest,
    PhysicalRecordFormatDeclaration, PhysicalRecordPlacementPolicy, RecordAppendBatch,
};
use worth_store_physical_format::{
    durable_artifact_checksum, inspect_inline_page_records, BlobRecordKind,
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest, DurableRootSelector,
    PhysicalRootRoutingBlock, PhysicalSegmentMembershipBlock, RecordArtifactFile,
    RecordSegmentPageManifestEntry, SelectedRecordContentClass,
};
use worth_store_recovery_runtime::{PhysicalRecoveryOutcome, WorthStoreRecovery};

#[test]
fn retired_physical_tail_slot_forces_fresh_page_with_c9_admissible_wal() {
    let world = pending_wal_world::first();
    world.kill_distinct_release_before_checkpoint();
    let outcome = WorthStoreRecovery::recover(certified_release_serving::request(world.root()));
    let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("two real released batches must recover before append: {outcome:?}");
    };
    let seal = handoff
        .into_core()
        .into_checkpoint_custody()
        .expect("selected two-batch custody seal");
    let serving = certified_release_serving::admit_serving_with_seal(world.root(), seal);
    let before = SelectedRoot::read(world.root());
    assert_eq!(
        before
            .routes
            .iter()
            .filter(|route| route.content_class()
                == SelectedRecordContentClass::Blob(BlobRecordKind::ReclaimDescriptorV3))
            .count(),
        2,
        "two genuine V3 release descriptors must precede the ordinary append",
    );
    let tail_record = before
        .manifest
        .last_inline_record()
        .expect("selected inline tail");
    let CurrentPhysicalRecordPlacement::Inline(tail) = before
        .routes
        .iter()
        .find(|route| route.record() == tail_record)
        .expect("last inline record has a selected route")
    else {
        panic!("last inline record must be inline");
    };
    let prior_page = tail.page_cell();
    let old_page_records = before.tail_page_records(world.root(), prior_page);
    let stale = old_page_records
        .iter()
        .filter(|physical| {
            !before
                .routes
                .iter()
                .any(|route| route.record() == physical.record())
        })
        .map(|physical| physical.record())
        .collect::<Vec<_>>();
    assert!(
        !stale.is_empty(),
        "baseline must have an earlier-retired physical tail slot"
    );
    assert!(old_page_records
        .iter()
        .any(|physical| physical.record() == tail_record));

    let placement = PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(64).unwrap())
        .admit(AdmittedPhysicalRecordFormat::admit(
            PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
        ))
        .unwrap();
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([0xb2; 32]))
        .unwrap();
    let TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) =
        submission
            .prepare_durable_append(
                RecordAppendBatch::try_from_iter([b"append-after-retired-tail".as_slice()])
                    .unwrap(),
                placement,
                PhysicalMutationRequest::platform_durable(
                    key,
                    PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
                ),
            )
            .into_raw()
    else {
        panic!("ordinary append after retired tail must prepare");
    };
    assert!(matches!(
        prepared.execute(),
        PhysicalMutationOutcome::Completed(_)
    ));

    let after = SelectedRoot::read(world.root());
    for prior in &before.routes {
        assert!(
            after.routes.contains(prior),
            "surviving selected route changed"
        );
    }
    for retired in stale {
        assert!(
            !after.routes.iter().any(|route| route.record() == retired),
            "append resurrected an earlier-retired physical slot"
        );
    }
    let new_routes = after
        .routes
        .iter()
        .filter(|route| {
            !before
                .routes
                .iter()
                .any(|old| old.record() == route.record())
        })
        .collect::<Vec<_>>();
    assert_eq!(
        new_routes.len(),
        1,
        "ordinary batch appends exactly one record"
    );
    let CurrentPhysicalRecordPlacement::Inline(appended) = new_routes[0] else {
        panic!("short appended record must be inline");
    };
    assert_ne!(
        appended.page_cell(),
        prior_page,
        "retired physical tail must not be copied"
    );

    serving.close();
    let root = world.root().to_path_buf();
    let outcome = WorthStoreRecovery::recover(certified_release_serving::request(&root));
    let PhysicalRecoveryOutcome::Recovered(_) = outcome else {
        panic!("C9 must admit the real producer WAL after fresh-page fallback: {outcome:?}");
    };
}

struct SelectedRoot {
    manifest: DurablePhysicalRootManifest,
    format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    routes: Vec<CurrentPhysicalRecordPlacement>,
    pages: Vec<RecordSegmentPageManifestEntry>,
}

impl SelectedRoot {
    fn read(root: &Path) -> Self {
        let records = root.join("families/records");
        let selector = DurableRootSelector::decode(
            &fs::read(records.join(RecordArtifactFile::CurrentRootSelector.file_name())).unwrap(),
        )
        .unwrap();
        let (manifest, format) = DurablePhysicalRootManifest::decode(
            &fs::read(
                records.join("roots").join(
                    RecordArtifactFile::RootManifest {
                        generation: selector.root_generation(),
                    }
                    .file_name(),
                ),
            )
            .unwrap(),
            u16::MAX,
        )
        .unwrap();
        assert_eq!(format, selector.format());
        let mut routes = Vec::new();
        let mut pending = manifest.routing_root().into_iter().collect::<Vec<_>>();
        while let Some(reference) = pending.pop() {
            let bytes = fs::read(
                records.join("roots").join(
                    RecordArtifactFile::RootRoutingBlock {
                        generation: reference.generation(),
                        block: reference.block(),
                    }
                    .file_name(),
                ),
            )
            .unwrap();
            let (block, found) =
                PhysicalRootRoutingBlock::decode(&bytes, manifest.node_capacity()).unwrap();
            assert_eq!(found, format);
            assert_eq!(
                block.reference(durable_artifact_checksum(&bytes)),
                reference
            );
            match block {
                PhysicalRootRoutingBlock::Leaf { entries, .. } => routes.extend(entries),
                PhysicalRootRoutingBlock::Branch { children, .. } => pending.extend(children),
            }
        }
        let mut pages = Vec::new();
        let mut pending = manifest.segment_root().into_iter().collect::<Vec<_>>();
        while let Some(reference) = pending.pop() {
            let bytes = fs::read(
                records.join("segment-manifests").join(
                    RecordArtifactFile::SegmentMembershipBlock {
                        generation: reference.generation(),
                        block: reference.block(),
                    }
                    .file_name(),
                ),
            )
            .unwrap();
            let (block, found) =
                PhysicalSegmentMembershipBlock::decode(&bytes, manifest.node_capacity()).unwrap();
            assert_eq!(found, format);
            assert_eq!(
                block.reference(durable_artifact_checksum(&bytes)),
                reference
            );
            match block {
                PhysicalSegmentMembershipBlock::Leaf { entries, .. } => pages.extend(entries),
                PhysicalSegmentMembershipBlock::Branch { children, .. } => pending.extend(children),
            }
        }
        Self {
            manifest,
            format,
            routes,
            pages,
        }
    }

    fn tail_page_records(
        &self,
        root: &Path,
        page: worth_store_physical_format::PageGenerationCell,
    ) -> Vec<worth_store_physical_format::InlinePageRecordDescriptor> {
        let entry = self
            .pages
            .iter()
            .find(|entry| entry.page_cell() == page)
            .expect("selected tail page membership");
        let bytes = fs::read(
            root.join("families/records/segments").join(
                RecordArtifactFile::Segment {
                    segment: page.segment_id().get(),
                    generation: entry.data_generation(),
                }
                .file_name(),
            ),
        )
        .unwrap();
        let size = self.format.page_size().bytes() as usize;
        let start = entry.frame_index() as usize * size;
        inspect_inline_page_records(self.format, &bytes[start..start + size]).unwrap()
    }
}
