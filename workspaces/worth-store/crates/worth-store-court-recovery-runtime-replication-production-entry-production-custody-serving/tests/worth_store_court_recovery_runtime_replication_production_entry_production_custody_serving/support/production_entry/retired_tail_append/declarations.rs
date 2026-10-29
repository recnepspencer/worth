//! An ordinary append after an earlier release must not copy an unselected
//! physical tail slot into its new WAL projection.

pub(crate) use {std::fs, std::path::Path};

pub(crate) use {worth_store::physical_runtime::PhysicalMutationDeadline, worth_proof::TransitionOutcome, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving, worth_store_testkit_recovery_runtime::production_entry::pending_wal_world};
pub(crate) use {worth_store::physical_runtime::AdmittedPhysicalRecordFormat, worth_store::physical_runtime::ManifestEntryCapacity, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::PhysicalRecordFormatDeclaration, worth_store::physical_runtime::PhysicalRecordPlacementPolicy, worth_store::physical_runtime::RecordAppendBatch};
pub(crate) use {worth_store_physical_format::durable_artifact_checksum, worth_store_physical_format::inspect_inline_page_records, worth_store_physical_format::BlobRecordKind, worth_store_physical_format::CurrentPhysicalRecordPlacement, worth_store_physical_format::DurablePhysicalRootManifest, worth_store_physical_format::DurableRootSelector, worth_store_physical_format::PhysicalRootRoutingBlock, worth_store_physical_format::PhysicalSegmentMembershipBlock, worth_store_physical_format::RecordArtifactFile, worth_store_physical_format::RecordSegmentPageManifestEntry, worth_store_physical_format::SelectedRecordContentClass};
pub(crate) use {worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::WorthStoreRecovery};

pub(crate) struct SelectedRoot {
    pub(crate) manifest: DurablePhysicalRootManifest,
    pub(crate) format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    pub(crate) routes: Vec<CurrentPhysicalRecordPlacement>,
    pub(crate) pages: Vec<RecordSegmentPageManifestEntry>,
}

impl SelectedRoot {
    pub(crate) fn read(root: &Path) -> Self {
        let records = root.join("families/records");
        let selector = worth_store_physical_format::DurableRootSelector::decode(
            &std::fs::read(records.join(worth_store_physical_format::RecordArtifactFile::CurrentRootSelector.file_name())).unwrap(),
        )
        .unwrap();
        let (manifest, format) = worth_store_physical_format::DurablePhysicalRootManifest::decode(
            &std::fs::read(
                records.join("roots").join(
                    worth_store_physical_format::RecordArtifactFile::RootManifest {
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
            let bytes = std::fs::read(
                records.join("roots").join(
                    worth_store_physical_format::RecordArtifactFile::RootRoutingBlock {
                        generation: reference.generation(),
                        block: reference.block(),
                    }
                    .file_name(),
                ),
            )
            .unwrap();
            let (block, found) =
                worth_store_physical_format::PhysicalRootRoutingBlock::decode(&bytes, manifest.node_capacity()).unwrap();
            assert_eq!(found, format);
            assert_eq!(
                block.reference(durable_artifact_checksum(&bytes)),
                reference
            );
            match block {
                worth_store_physical_format::PhysicalRootRoutingBlock::Leaf { entries, .. } => routes.extend(entries),
                worth_store_physical_format::PhysicalRootRoutingBlock::Branch { children, .. } => pending.extend(children),
            }
        }
        let mut pages = Vec::new();
        let mut pending = manifest.segment_root().into_iter().collect::<Vec<_>>();
        while let Some(reference) = pending.pop() {
            let bytes = std::fs::read(
                records.join("segment-manifests").join(
                    worth_store_physical_format::RecordArtifactFile::SegmentMembershipBlock {
                        generation: reference.generation(),
                        block: reference.block(),
                    }
                    .file_name(),
                ),
            )
            .unwrap();
            let (block, found) =
                worth_store_physical_format::PhysicalSegmentMembershipBlock::decode(&bytes, manifest.node_capacity()).unwrap();
            assert_eq!(found, format);
            assert_eq!(
                block.reference(durable_artifact_checksum(&bytes)),
                reference
            );
            match block {
                worth_store_physical_format::PhysicalSegmentMembershipBlock::Leaf { entries, .. } => pages.extend(entries),
                worth_store_physical_format::PhysicalSegmentMembershipBlock::Branch { children, .. } => pending.extend(children),
            }
        }
        Self {
            manifest,
            format,
            routes,
            pages,
        }
    }

    pub(crate) fn tail_page_records(
        &self,
        root: &Path,
        page: worth_store_physical_format::PageGenerationCell,
    ) -> Vec<worth_store_physical_format::InlinePageRecordDescriptor> {
        let entry = self
            .pages
            .iter()
            .find(|entry| entry.page_cell() == page)
            .expect("selected tail page membership");
        let bytes = std::fs::read(
            root.join("families/records/segments").join(
                worth_store_physical_format::RecordArtifactFile::Segment {
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