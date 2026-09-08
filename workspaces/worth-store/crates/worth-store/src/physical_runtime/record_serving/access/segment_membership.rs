#[cfg(feature = "certification-test-authority")]
use worth_store_physical_backend::QualifiedFilesystemMedia;
use worth_store_physical_format::{
    DurablePhysicalRootManifest, PhysicalPageId, PhysicalSegmentId, PhysicalSegmentMembershipBlock,
    PhysicalTreeIdentity, RecordArtifactFile, RecordSegmentPageManifestEntry,
    SegmentManifestBlockReference, SegmentMembershipBlockScopeIdentity, SegmentPageKey,
};
use worth_store_physical_integrity::{PhysicalArtifactScope, PhysicalByteRange};

use super::super::access::manifest_routing::{
    ManifestDiscoveryCounterSnapshot, ManifestLookupFailure,
};
use super::super::residency::{record_frame_reader::RecordFrameReader, PhysicalResidencyWorkPort};
use super::super::{AdmittedPhysicalRecordFormat, AdmittedRecordAccessPolicy};

mod update_planning;
pub(in crate::physical_runtime::record_serving) use update_planning::{
    plan_segment_membership_updates, SegmentMembershipPublicationPlan,
    SegmentMembershipUpdateContext,
};

pub(in crate::physical_runtime::record_serving) struct SegmentMembershipReader<'media> {
    artifacts: RecordFrameReader<'media>,
    format: AdmittedPhysicalRecordFormat,
    access: AdmittedRecordAccessPolicy,
    root: DurablePhysicalRootManifest,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    admission:
        crate::physical_runtime::integrity::resident_admission::load::ResidentAdmissionContext<
            'media,
        >,
}

impl<'media> SegmentMembershipReader<'media> {
    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime::record_serving) fn with_loader(
        media: &'media QualifiedFilesystemMedia,
        loader: &'media (dyn super::super::residency::frame_ports::FrameLoadPort + Send + Sync),
        format: AdmittedPhysicalRecordFormat,
        access: AdmittedRecordAccessPolicy,
        root: &'media DurablePhysicalRootManifest,
        lifecycle: std::sync::Arc<crate::physical_runtime::lifecycle::LifecycleState>,
        counters: &'media crate::physical_runtime::ResidentAdmissionCounterCells,
    ) -> Self {
        Self {
            artifacts: RecordFrameReader::bootstrap(media, loader),
            format,
            access,
            root: root.clone(),
            store: media.store_identity(),
            admission: crate::physical_runtime::integrity::resident_admission::load::ResidentAdmissionContext::new(lifecycle, counters),
        }
    }

    pub(in crate::physical_runtime::record_serving) fn serving(
        residency: PhysicalResidencyWorkPort,
        format: AdmittedPhysicalRecordFormat,
        access: AdmittedRecordAccessPolicy,
        root: DurablePhysicalRootManifest,
    ) -> SegmentMembershipReader<'static> {
        let admission = residency.resident_admission_context();
        let store = residency.store_identity();
        SegmentMembershipReader {
            artifacts: RecordFrameReader::serving(residency),
            format,
            access,
            root,
            store,
            admission,
        }
    }

    pub(in crate::physical_runtime::record_serving) fn locate(
        &self,
        allocation: &worth_store_buffer_pool::OperationAllocationGrant,
        segment: PhysicalSegmentId,
        page: PhysicalPageId,
        counters: &mut ManifestDiscoveryCounterSnapshot,
    ) -> Result<Option<RecordSegmentPageManifestEntry>, ManifestLookupFailure> {
        let key = SegmentPageKey::new(segment, page);
        let Some(mut reference) = self.root.segment_root() else {
            return Ok(None);
        };
        if !reference.contains(key) {
            return Ok(None);
        }
        loop {
            match self.read_block(allocation, reference, counters)? {
                PhysicalSegmentMembershipBlock::Leaf { entries, .. } => {
                    let (result, comparisons) =
                        super::super::access::counted_search::binary_search_by(&entries, |entry| {
                            SegmentPageKey::from(*entry).cmp(&key)
                        });
                    counters.observe_comparisons(comparisons);
                    return Ok(result.ok().map(|index| entries[index]));
                }
                PhysicalSegmentMembershipBlock::Branch { children, .. } => {
                    let (index, comparisons) =
                        super::super::access::counted_search::partition_point(&children, |child| {
                            child.last() < key
                        });
                    counters.observe_comparisons(comparisons);
                    let Some(child) = children
                        .get(index)
                        .copied()
                        .filter(|child| child.contains(key))
                    else {
                        return Ok(None);
                    };
                    reference = child;
                }
            }
        }
    }

    pub(in crate::physical_runtime::record_serving) fn read_block(
        &self,
        allocation: &worth_store_buffer_pool::OperationAllocationGrant,
        reference: SegmentManifestBlockReference,
        counters: &mut ManifestDiscoveryCounterSnapshot,
    ) -> Result<PhysicalSegmentMembershipBlock, ManifestLookupFailure> {
        let limit = self
            .access
            .transfer_limit()
            .get()
            .min(self.format.declaration().page_size().bytes());
        let bytes = self
            .artifacts
            .load_bounded(
                allocation,
                RecordArtifactFile::SegmentMembershipBlock {
                    generation: reference.generation(),
                    block: reference.block(),
                },
                limit,
            )
            .map_err(|failure| {
                counters.observe_failed_work(failure.work_trace());
                frame_load_failure(failure)
            })?;
        counters.observe_block(bytes.len(), bytes.work_trace());
        let Some(tree) = PhysicalTreeIdentity::new(self.root.tree_identity()) else {
            bytes.reject_projection_failure();
            return Err(ManifestLookupFailure::Damaged);
        };
        let Ok(range) = PhysicalByteRange::new(0, bytes.len() as u64) else {
            bytes.reject_projection_failure();
            return Err(ManifestLookupFailure::Damaged);
        };
        let scope = PhysicalArtifactScope::segment_membership_block(
            self.store,
            self.format.declaration(),
            SegmentMembershipBlockScopeIdentity::new(tree, reference),
            range,
        );
        let admitted = crate::physical_runtime::integrity::resident_admission::root_tree::admit_resident_segment_membership_block(
            bytes.lease(),
            scope,
            self.admission.clone(),
        );
        let decoded = admitted.and_then(|admitted| {
            admitted.with_owner_decoder(self.admission.clone(), |view| {
                view.project_block(self.root.node_capacity())
            })
        });
        match decoded {
            Ok(Ok(block)) => Ok(block),
            Ok(Err(_)) => {
                bytes.reject_projection_failure();
                Err(ManifestLookupFailure::Damaged)
            }
            Err(denial) => {
                if !denial.preserves_resident_bytes() {
                    bytes.reject_projection_failure();
                }
                Err(ManifestLookupFailure::ResidentAdmission(denial))
            }
        }
    }

    fn format_declaration(&self) -> worth_store_physical_format::PhysicalRecordFormatDeclaration {
        self.format.declaration()
    }
}

fn frame_load_failure(
    failure: crate::physical_runtime::record_serving::residency::frame_loading::FrameLoadFailure,
) -> ManifestLookupFailure {
    use crate::physical_runtime::record_serving::residency::frame_loading::FrameLoadFailureKind;
    match failure.kind() {
        FrameLoadFailureKind::Backend(reason) => ManifestLookupFailure::Backend(reason),
        FrameLoadFailureKind::Residency(reason) => ManifestLookupFailure::Residency(reason),
        kind => ManifestLookupFailure::Frame(kind),
    }
}
