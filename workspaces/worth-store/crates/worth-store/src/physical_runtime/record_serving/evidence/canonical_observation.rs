use worth_store_physical_format::{
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest, PhysicalFreeSpaceMembershipBlock,
    PhysicalSegmentMembershipBlock, RecordAllocationClass, RecordFreeSpaceManifestEntry,
    RecordSegmentPageManifestEntry,
};

use super::super::evidence::canonical_evidence::{
    runtime_placement, CanonicalFreeSpace, CanonicalSegmentPage, PhysicalRecordPublicationSummary,
    RecordCanonicalObservationDenial,
};
use super::super::{
    access::manifest_routing::{
        ManifestDiscoveryCounterSnapshot, ManifestRangeCursor, ManifestReader,
    },
    residency::frame_ports::FrameLoadPort,
    AdmittedPhysicalRecordFormat, AdmittedRecordAccessPolicy,
};

pub(in crate::physical_runtime::record_serving) struct RuntimeTopologySource<'runtime> {
    pub(in crate::physical_runtime::record_serving) allocation:
        &'runtime worth_store_buffer_pool::OperationAllocationGrant,
    pub(in crate::physical_runtime::record_serving) media:
        &'runtime worth_store_physical_backend::QualifiedFilesystemMedia,
    pub(in crate::physical_runtime::record_serving) frame_load:
        &'runtime (dyn FrameLoadPort + Send + Sync),
    pub(in crate::physical_runtime::record_serving) format: AdmittedPhysicalRecordFormat,
    pub(in crate::physical_runtime::record_serving) access: AdmittedRecordAccessPolicy,
    pub(in crate::physical_runtime::record_serving) root: &'runtime DurablePhysicalRootManifest,
    pub(in crate::physical_runtime::record_serving) free_space:
        &'runtime DurableFreeSpaceManifestHeader,
    pub(in crate::physical_runtime::record_serving) lifecycle:
        std::sync::Arc<crate::physical_runtime::lifecycle::LifecycleState>,
    pub(in crate::physical_runtime::record_serving) integrity_counters:
        &'runtime crate::physical_runtime::ResidentAdmissionCounterCells,
}

pub(in crate::physical_runtime::record_serving) fn observe_runtime_topology(
    source: RuntimeTopologySource<'_>,
) -> Result<PhysicalRecordPublicationSummary, RecordCanonicalObservationDenial> {
    let RuntimeTopologySource {
        allocation,
        media,
        frame_load,
        format,
        access,
        root,
        free_space,
        lifecycle,
        integrity_counters,
    } = source;
    let reader = ManifestReader::with_loader(
        media,
        frame_load,
        format,
        access,
        root,
        std::sync::Arc::clone(&lifecycle),
        integrity_counters,
    );
    let mut cursor = ManifestRangeCursor::new(reader);
    cursor
        .seek(allocation, root.routing_root(), None)
        .map_err(|_| RecordCanonicalObservationDenial::ManifestUnavailable)?;
    let mut placements = Vec::new();
    while let Some(placement) = cursor
        .next(allocation)
        .map_err(|_| RecordCanonicalObservationDenial::ManifestUnavailable)?
    {
        placements.push(runtime_placement(placement));
    }
    let mut segment_pages = runtime_segment_pages(
        allocation,
        media,
        frame_load,
        format,
        access,
        root,
        std::sync::Arc::clone(&lifecycle),
        integrity_counters,
    )?;
    let mut free_space = runtime_free_space(
        allocation,
        media,
        frame_load,
        format,
        access,
        free_space,
        lifecycle,
        integrity_counters,
    )?;
    placements.sort_unstable();
    segment_pages.sort_unstable();
    free_space.sort_unstable();
    Ok(PhysicalRecordPublicationSummary {
        store_identity: media.store_identity().bytes(),
        format_identity: format.declaration().canonical_identity_bytes(),
        root_generation: root.generation(),
        tree_identity: root.tree_identity(),
        node_capacity: root.node_capacity(),
        routing_level: root.routing_root().map(|reference| reference.level()),
        placements,
        segment_pages,
        free_space,
    })
}

fn runtime_segment_pages(
    allocation: &worth_store_buffer_pool::OperationAllocationGrant,
    media: &worth_store_physical_backend::QualifiedFilesystemMedia,
    loader: &(dyn FrameLoadPort + Send + Sync),
    format: AdmittedPhysicalRecordFormat,
    access: AdmittedRecordAccessPolicy,
    root: &DurablePhysicalRootManifest,
    lifecycle: std::sync::Arc<crate::physical_runtime::lifecycle::LifecycleState>,
    integrity_counters: &crate::physical_runtime::ResidentAdmissionCounterCells,
) -> Result<Vec<CanonicalSegmentPage>, RecordCanonicalObservationDenial> {
    let reader = super::super::access::segment_membership::SegmentMembershipReader::with_loader(
        media,
        loader,
        format,
        access,
        root,
        lifecycle,
        integrity_counters,
    );
    let mut pending = root.segment_root().into_iter().collect::<Vec<_>>();
    let mut counters = ManifestDiscoveryCounterSnapshot::default();
    let mut result = Vec::new();
    while let Some(reference) = pending.pop() {
        match reader
            .read_block(allocation, reference, &mut counters)
            .map_err(|_| RecordCanonicalObservationDenial::ManifestUnavailable)?
        {
            PhysicalSegmentMembershipBlock::Leaf { entries, .. } => {
                result.extend(entries.into_iter().map(runtime_segment_page));
            }
            PhysicalSegmentMembershipBlock::Branch { children, .. } => {
                pending.extend(children.into_iter().rev());
            }
        }
    }
    Ok(result)
}

fn runtime_segment_page(entry: RecordSegmentPageManifestEntry) -> CanonicalSegmentPage {
    CanonicalSegmentPage {
        segment: entry.page_cell().segment_id().get(),
        page: entry.page().get(),
        page_generation: entry.page_generation(),
        data_generation: entry.data_generation(),
        data_page_count: u64::from(entry.data_page_count()),
        frame_index: u64::from(entry.frame_index()),
    }
}

fn runtime_free_space(
    allocation: &worth_store_buffer_pool::OperationAllocationGrant,
    media: &worth_store_physical_backend::QualifiedFilesystemMedia,
    loader: &(dyn FrameLoadPort + Send + Sync),
    format: AdmittedPhysicalRecordFormat,
    access: AdmittedRecordAccessPolicy,
    header: &DurableFreeSpaceManifestHeader,
    lifecycle: std::sync::Arc<crate::physical_runtime::lifecycle::LifecycleState>,
    integrity_counters: &crate::physical_runtime::ResidentAdmissionCounterCells,
) -> Result<Vec<CanonicalFreeSpace>, RecordCanonicalObservationDenial> {
    let reader = super::super::planning::free_space_routing::FreeSpaceReader::with_loader(
        media,
        loader,
        format,
        access,
        header,
        lifecycle,
        integrity_counters,
    );
    let mut pending = header.root().into_iter().collect::<Vec<_>>();
    let mut counters = ManifestDiscoveryCounterSnapshot::default();
    let mut result = Vec::new();
    while let Some(reference) = pending.pop() {
        match reader
            .read_block(allocation, reference, &mut counters)
            .map_err(|_| RecordCanonicalObservationDenial::ManifestUnavailable)?
        {
            PhysicalFreeSpaceMembershipBlock::Leaf { entries, .. } => {
                result.extend(entries.into_iter().map(runtime_free_entry));
            }
            PhysicalFreeSpaceMembershipBlock::Branch { children, .. } => {
                pending.extend(children.into_iter().rev());
            }
        }
    }
    Ok(result)
}

fn runtime_free_entry(entry: RecordFreeSpaceManifestEntry) -> CanonicalFreeSpace {
    CanonicalFreeSpace {
        class: match entry.class() {
            RecordAllocationClass::InlinePage => 1,
            RecordAllocationClass::Extent => 2,
        },
        owner: entry.owner(),
        first_unallocated: entry.first_unallocated(),
        unallocated_count: entry.unallocated_count(),
        generation: entry.generation(),
    }
}
