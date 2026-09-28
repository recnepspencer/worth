use worth_store_physical_backend::QualifiedFilesystemMedia;
use worth_store_physical_format::{
    DurableFreeSpaceManifestHeader, FreeSpaceBlockReference, FreeSpaceKey,
    FreeSpaceMembershipBlockScopeIdentity, PhysicalFreeSpaceMembershipBlock, PhysicalTreeIdentity,
    RecordArtifactFile, RecordFreeSpaceManifestEntry,
};
use worth_store_physical_integrity::{PhysicalArtifactScope, PhysicalByteRange};

use super::super::super::access::manifest_routing::{
    ManifestDiscoveryCounterSnapshot, ManifestLookupFailure,
};
use super::super::super::residency::{
    record_frame_reader::RecordFrameReader, PhysicalResidencyWorkPort,
};
use super::super::super::{AdmittedPhysicalRecordFormat, AdmittedRecordAccessPolicy};

pub(in crate::physical_runtime::record_serving) struct FreeSpaceReader<'media> {
    artifacts: RecordFrameReader<'media>,
    pub(super) format: AdmittedPhysicalRecordFormat,
    access: AdmittedRecordAccessPolicy,
    pub(super) header: &'media DurableFreeSpaceManifestHeader,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    admission:
        crate::physical_runtime::integrity::resident_admission::load::ResidentAdmissionContext<
            'media,
        >,
}

impl<'media> FreeSpaceReader<'media> {
    /// Bootstrap-only walk. Steady-state allocation consults the resident index.
    pub(in crate::physical_runtime::record_serving) fn visit_arena_ranges(
        &self,
        allocation: &worth_store_buffer_pool::OperationAllocationGrant,
        counters: &mut ManifestDiscoveryCounterSnapshot,
        mut visit: impl FnMut(RecordFreeSpaceManifestEntry) -> Result<(), ()>,
    ) -> Result<(), ManifestLookupFailure> {
        let minimum = FreeSpaceKey::arena(
            worth_store_physical_format::ExtentArenaId::new(1).expect("nonzero"),
            0,
        );
        let stack_limit = usize::try_from(
            allocation.bytes() / 2 / std::mem::size_of::<FreeSpaceBlockReference>() as u64,
        )
        .unwrap_or(usize::MAX);
        if stack_limit == 0 {
            return Err(traversal_pressure());
        }
        let mut pending = self
            .header
            .root()
            .into_iter()
            .filter(|root| root.last() >= minimum)
            .collect::<Vec<_>>();
        let mut observed = 0_u64;
        let mut visited_blocks = 0_u64;
        let block_limit = self
            .header
            .entry_count()
            .saturating_mul(64)
            .saturating_add(1);
        while let Some(reference) = pending.pop() {
            visited_blocks = visited_blocks
                .checked_add(1)
                .ok_or(ManifestLookupFailure::Damaged)?;
            if visited_blocks > block_limit {
                return Err(ManifestLookupFailure::Damaged);
            }
            match self.read_block(allocation, reference, counters)? {
                PhysicalFreeSpaceMembershipBlock::Leaf { entries, .. } => {
                    for entry in entries {
                        if entry.arena_free_range().is_none() {
                            continue;
                        }
                        observed = observed
                            .checked_add(1)
                            .ok_or(ManifestLookupFailure::Damaged)?;
                        if observed > self.header.entry_count() {
                            return Err(ManifestLookupFailure::Damaged);
                        }
                        visit(entry).map_err(|_| ManifestLookupFailure::Damaged)?;
                    }
                }
                PhysicalFreeSpaceMembershipBlock::Branch { children, .. } => {
                    let children = children
                        .into_iter()
                        .rev()
                        .filter(|child| child.last() >= minimum);
                    for child in children {
                        if pending.len() == stack_limit {
                            return Err(traversal_pressure());
                        }
                        pending
                            .try_reserve_exact(1)
                            .map_err(|_| traversal_pressure())?;
                        pending.push(child);
                    }
                }
            }
        }
        Ok(())
    }

    pub(in crate::physical_runtime::record_serving) fn with_loader(
        media: &'media QualifiedFilesystemMedia,
        loader: &'media (dyn super::super::super::residency::frame_ports::FrameLoadPort
                     + Send
                     + Sync),
        format: AdmittedPhysicalRecordFormat,
        access: AdmittedRecordAccessPolicy,
        header: &'media DurableFreeSpaceManifestHeader,
        lifecycle: std::sync::Arc<crate::physical_runtime::lifecycle::LifecycleState>,
        counters: &'media crate::physical_runtime::ResidentAdmissionCounterCells,
    ) -> Self {
        Self {
            artifacts: RecordFrameReader::bootstrap(media, loader),
            format,
            access,
            header,
            store: media.store_identity(),
            admission: crate::physical_runtime::integrity::resident_admission::load::ResidentAdmissionContext::new(lifecycle, counters),
        }
    }

    pub(in crate::physical_runtime::record_serving) fn serving(
        residency: PhysicalResidencyWorkPort,
        format: AdmittedPhysicalRecordFormat,
        access: AdmittedRecordAccessPolicy,
        header: &'media DurableFreeSpaceManifestHeader,
    ) -> Self {
        let admission = residency.resident_admission_context();
        let store = residency.store_identity();
        Self {
            artifacts: RecordFrameReader::serving(residency),
            format,
            access,
            header,
            store,
            admission,
        }
    }

    pub(in crate::physical_runtime::record_serving) fn locate(
        &self,
        allocation: &worth_store_buffer_pool::OperationAllocationGrant,
        key: FreeSpaceKey,
        counters: &mut ManifestDiscoveryCounterSnapshot,
    ) -> Result<Option<RecordFreeSpaceManifestEntry>, ManifestLookupFailure> {
        let Some(mut reference) = self.header.root() else {
            return Ok(None);
        };
        if !reference.contains(key) {
            return Ok(None);
        }
        loop {
            match self.read_block(allocation, reference, counters)? {
                PhysicalFreeSpaceMembershipBlock::Leaf { entries, .. } => {
                    let (result, comparisons) =
                        super::super::super::access::counted_search::binary_search_by(
                            &entries,
                            |entry| FreeSpaceKey::from(*entry).cmp(&key),
                        );
                    counters.observe_comparisons(comparisons);
                    return Ok(result.ok().map(|index| entries[index]));
                }
                PhysicalFreeSpaceMembershipBlock::Branch { children, .. } => {
                    let (index, comparisons) =
                        super::super::super::access::counted_search::partition_point(
                            &children,
                            |child| child.last() < key,
                        );
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

    pub(in crate::physical_runtime::record_serving) fn floor(
        &self,
        allocation: &worth_store_buffer_pool::OperationAllocationGrant,
        key: FreeSpaceKey,
        counters: &mut ManifestDiscoveryCounterSnapshot,
    ) -> Result<Option<RecordFreeSpaceManifestEntry>, ManifestLookupFailure> {
        let Some(mut reference) = self.header.root() else {
            return Ok(None);
        };
        if reference.first() > key {
            return Ok(None);
        }
        loop {
            match self.read_block(allocation, reference, counters)? {
                PhysicalFreeSpaceMembershipBlock::Leaf { entries, .. } => {
                    let (index, comparisons) =
                        super::super::super::access::counted_search::partition_point(
                            &entries,
                            |entry| FreeSpaceKey::from(*entry) <= key,
                        );
                    counters.observe_comparisons(comparisons);
                    return Ok(index.checked_sub(1).map(|index| entries[index]));
                }
                PhysicalFreeSpaceMembershipBlock::Branch { children, .. } => {
                    let (index, comparisons) =
                        super::super::super::access::counted_search::partition_point(
                            &children,
                            |child| child.first() <= key,
                        );
                    counters.observe_comparisons(comparisons);
                    reference = *children
                        .get(index.checked_sub(1).ok_or(ManifestLookupFailure::Damaged)?)
                        .ok_or(ManifestLookupFailure::Damaged)?;
                }
            }
        }
    }

    pub(in crate::physical_runtime::record_serving) fn read_block(
        &self,
        allocation: &worth_store_buffer_pool::OperationAllocationGrant,
        reference: FreeSpaceBlockReference,
        counters: &mut ManifestDiscoveryCounterSnapshot,
    ) -> Result<PhysicalFreeSpaceMembershipBlock, ManifestLookupFailure> {
        let bytes = self
            .artifacts
            .load_bounded(
                allocation,
                RecordArtifactFile::FreeSpaceMembershipBlock {
                    generation: reference.generation(),
                    block: reference.block(),
                },
                self.access
                    .transfer_limit()
                    .get()
                    .min(self.format.declaration().page_size().bytes()),
            )
            .map_err(|failure| {
                counters.observe_failed_work(failure.work_trace());
                frame_load_failure(failure)
            })?;
        counters.observe_block(bytes.len(), bytes.work_trace());
        let Some(tree) = PhysicalTreeIdentity::new(self.header.tree_identity()) else {
            bytes.reject_projection_failure();
            return Err(ManifestLookupFailure::Damaged);
        };
        let Ok(range) = PhysicalByteRange::new(0, bytes.len() as u64) else {
            bytes.reject_projection_failure();
            return Err(ManifestLookupFailure::Damaged);
        };
        let scope = PhysicalArtifactScope::free_space_membership_block(
            self.store,
            self.format.declaration(),
            FreeSpaceMembershipBlockScopeIdentity::new(tree, reference),
            range,
        );
        let admitted = crate::physical_runtime::integrity::resident_admission::free_space::admit_resident_free_space_membership_block(
            bytes.lease(),
            scope,
            self.admission.clone(),
        );
        let decoded = admitted.and_then(|admitted| {
            admitted.with_owner_decoder(self.admission.clone(), |view| {
                view.project_block(self.header.node_capacity())
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
}

fn traversal_pressure() -> ManifestLookupFailure {
    ManifestLookupFailure::Residency(
        worth_store_buffer_pool::PhysicalResidencyDenial::MetadataBudgetExceeded,
    )
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
