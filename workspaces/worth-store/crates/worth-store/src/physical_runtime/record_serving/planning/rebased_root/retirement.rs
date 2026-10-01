use super::{
    assembly::{assemble_successor, RootAssemblyContext},
    damaged,
    projection::ProjectedSuccessorRoot,
};
use crate::physical_runtime::record_serving::{
    access::manifest_routing::ManifestDiscoveryCounterSnapshot,
    planning::free_space_routing::{
        plan_free_space_successor, FreeSpaceReader, FreeSpaceSuccessorRequest, FreeSpaceUpdate,
    },
    planning::inline_plan_failure::manifest_lookup_failure,
    publication::{append_observation::PublicationObservation, PublicationPlan},
    residency::PhysicalResidencyWorkPort,
    AdmittedPhysicalRecordFormat, AdmittedRecordAccessPolicy, RecordAppendError,
};
use std::collections::BTreeMap;
use worth_store_physical_format::{
    durable_artifact_checksum, DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest,
    ExtentArenaRange, FreeSpaceKey, RecordArtifactFile, RecordFreeSpaceManifestEntry,
};

pub(in crate::physical_runtime::record_serving) struct RetirementRootPlanningContext<'a> {
    pub allocation: &'a worth_store_buffer_pool::OperationAllocationGrant,
    pub residency: PhysicalResidencyWorkPort,
    pub format: AdmittedPhysicalRecordFormat,
    pub access: AdmittedRecordAccessPolicy,
    pub media: &'a worth_store_physical_backend::QualifiedFilesystemMedia,
    pub current_root: &'a DurablePhysicalRootManifest,
    pub current_free: &'a DurableFreeSpaceManifestHeader,
}

pub(in crate::physical_runtime::record_serving) fn plan_retirement_release(
    context: RetirementRootPlanningContext<'_>,
    range: ExtentArenaRange,
    candidate: RecordArtifactFile,
) -> Result<(PublicationPlan, DurableFreeSpaceManifestHeader), RecordAppendError> {
    plan_retirement_change(context, RetirementChange::Release(range), candidate)
}

pub(in crate::physical_runtime::record_serving) fn plan_arena_forget(
    context: RetirementRootPlanningContext<'_>,
    arena: worth_store_physical_format::ExtentArenaId,
    candidate: RecordArtifactFile,
) -> Result<(PublicationPlan, DurableFreeSpaceManifestHeader), RecordAppendError> {
    plan_retirement_change(context, RetirementChange::Forget(arena), candidate)
}

enum RetirementChange {
    Release(ExtentArenaRange),
    Forget(worth_store_physical_format::ExtentArenaId),
}

fn plan_retirement_change(
    context: RetirementRootPlanningContext<'_>,
    change: RetirementChange,
    candidate: RecordArtifactFile,
) -> Result<(PublicationPlan, DurableFreeSpaceManifestHeader), RecordAppendError> {
    let current = context.current_root;
    let generation = current.generation().checked_add(1).ok_or_else(damaged)?;
    let reader = FreeSpaceReader::serving(
        context.residency.clone(),
        context.format,
        context.access,
        context.current_free,
    );
    let mut counters = ManifestDiscoveryCounterSnapshot::default();
    let updates = match change {
        RetirementChange::Release(range) => release_updates(
            &reader,
            context.allocation,
            range,
            generation,
            &mut counters,
        )?,
        RetirementChange::Forget(arena) => {
            let key = FreeSpaceKey::arena(arena, 0);
            let range = reader
                .locate(context.allocation, key, &mut counters)
                .map_err(manifest_lookup_failure)?
                .and_then(|entry| entry.arena_free_range())
                .ok_or_else(damaged)?;
            if range.length() != context.current_free.arena_capacity()
                || arena.get() >= context.current_free.next_arena()
            {
                return Err(damaged());
            }
            BTreeMap::from([(key, FreeSpaceUpdate::Exhausted)])
        }
    };
    let free = context.current_free;
    let mut projected = plan_free_space_successor(
        context.allocation,
        context.residency,
        context.format,
        context.access,
        free,
        FreeSpaceSuccessorRequest {
            generation,
            node_capacity: free.node_capacity(),
            segment_page_capacity: free.segment_page_capacity(),
            next_segment: free.next_segment(),
            next_page: free.next_page(),
            next_extent: free.next_extent(),
            next_arena: free.next_arena(),
            updates,
        },
    )?;
    counters.merge(projected.discovery);
    let bytes = projected.header.encode(context.format.declaration());
    let root = DurablePhysicalRootManifest::builder(
        generation,
        current.tree_identity(),
        current.node_capacity(),
        durable_artifact_checksum(&bytes),
    )
    .record_count(current.record_count())
    .next_block(current.next_block())
    .next_segment_block(current.next_segment_block())
    .next_release_custody_head_block(current.next_release_custody_head_block())
    .routing_root(current.routing_root())
    .release_custody_head_root(current.release_custody_head_root())
    .segment_root(current.segment_root())
    .free_space_root(projected.header.root())
    .tier_epoch_anchor(current.tier_epoch_anchor())
    .latest_blob_publication(current.latest_blob_publication())
    .latest_blob_quarantine(current.latest_blob_quarantine())
    .derived_family_directory(current.derived_family_directory())
    .last_inline_record(current.last_inline_record())
    .last_inline_segment(current.last_inline_segment())
    .admit()
    .ok_or_else(damaged)?
    .with_maintenance_protocol();
    projected
        .blocks
        .push((RecordArtifactFile::FreeSpaceManifest { generation }, bytes));
    let RecordArtifactFile::CatalogCandidate { publication } = candidate else {
        return Err(damaged());
    };
    let plan = PublicationPlan {
        routing_metadata_bytes: None,
        arena_reservations: Vec::new(),
        generation,
        manifests: Vec::new(),
        root: RecordArtifactFile::RootManifest { generation },
        candidate,
        manifest: current.clone(),
        root_bytes: Vec::new(),
        previous_selector_candidate: RecordArtifactFile::RootSelectorCandidate {
            role: worth_store_physical_format::RootSelectorRole::Previous,
            publication,
        },
        previous_selector_bytes: Vec::new(),
        current_selector_candidate: RecordArtifactFile::RootSelectorCandidate {
            role: worth_store_physical_format::RootSelectorRole::Current,
            publication,
        },
        current_selector_bytes: Vec::new(),
        catalog_bytes: Vec::new(),
        observation: PublicationObservation::default(),
    };
    Ok(assemble_successor(
        plan,
        RootAssemblyContext {
            media: context.media,
            format: context.format,
            current_root: current,
        },
        generation,
        ProjectedSuccessorRoot {
            free_space: projected.header,
            manifests: projected.blocks,
            root,
            discoveries: [
                counters,
                ManifestDiscoveryCounterSnapshot::default(),
                ManifestDiscoveryCounterSnapshot::default(),
            ],
        },
    ))
}

fn release_updates(
    reader: &FreeSpaceReader<'_>,
    allocation: &worth_store_buffer_pool::OperationAllocationGrant,
    range: ExtentArenaRange,
    generation: u64,
    counters: &mut ManifestDiscoveryCounterSnapshot,
) -> Result<BTreeMap<FreeSpaceKey, FreeSpaceUpdate>, RecordAppendError> {
    let mut updates = BTreeMap::new();
    let mut start = range.offset();
    let mut end = range.end();
    let left = reader
        .floor(
            allocation,
            FreeSpaceKey::arena(range.arena(), range.end() - 1),
            counters,
        )
        .map_err(manifest_lookup_failure)?
        .and_then(|entry| entry.arena_free_range());
    if let Some(left) = left.filter(|left| left.arena() == range.arena()) {
        if left.end() > range.offset() {
            return Err(damaged());
        }
        if left.end() == range.offset() {
            start = left.offset();
            updates.insert(
                FreeSpaceKey::arena(left.arena(), left.offset()),
                FreeSpaceUpdate::Exhausted,
            );
        }
    }
    let right = reader
        .locate(
            allocation,
            FreeSpaceKey::arena(range.arena(), range.end()),
            counters,
        )
        .map_err(manifest_lookup_failure)?
        .and_then(|entry| entry.arena_free_range());
    if let Some(right) = right {
        end = right.end();
        updates.insert(
            FreeSpaceKey::arena(right.arena(), right.offset()),
            FreeSpaceUpdate::Exhausted,
        );
    }
    let free = ExtentArenaRange::new(range.arena(), start, end - start).ok_or_else(damaged)?;
    updates.insert(
        FreeSpaceKey::arena(free.arena(), free.offset()),
        FreeSpaceUpdate::Available(
            RecordFreeSpaceManifestEntry::arena_range(free, generation).ok_or_else(damaged)?,
        ),
    );
    Ok(updates)
}
