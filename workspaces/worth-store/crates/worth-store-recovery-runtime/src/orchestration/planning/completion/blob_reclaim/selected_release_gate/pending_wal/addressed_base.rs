//! A Batch/Accumulator checkpoint base is observed at its own addressed
//! source root, not assumed to survive the current postcheckpoint root.

use sha2::{Digest, Sha256};
use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    BlobRecordKind, BlobRecordV1, CurrentPhysicalRecordPlacement, PersistedRecordIdentity,
    PhysicalRecordFormatDeclaration, RecordSegmentPageManifestEntry, ReleasedDropWalFateWitnessV1,
    SelectedRecordContentClass,
    BLOB_CONTROL_FRAME_MAX_BYTES,
};
use worth_store_recovery_physics::{
    AddressedCheckpointBatchControl, ReleasedInventoryView,
    VerifiedAddressedCheckpointReleaseBase, VerifiedOrderedRootHistory,
    WitnessedSelectedControlFrame,
};

use crate::integrity_ingress::{
    admit_addressed_root, RecoveryArtifactNamespaceJoin, RecoveryIntegrityIngressTrace,
};
use crate::orchestration::planning::{
    completion::blob_reclaim::record, manifest_entry_budget::ManifestEntryBudget,
    selected_source_inventory,
};

use super::{PlanningContext, ResolvedPlanningBasis};

pub(super) fn admit(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    history: &VerifiedOrderedRootHistory,
) -> Result<
    (PlanningContext, Option<VerifiedAddressedCheckpointReleaseBase>),
    crate::entry::PhysicalRecoveryOutcome,
> {
    let roster = match context.selection.checkpoint() {
        Some(checkpoint) => super::super::certificates::selected_roster(checkpoint),
        None => return Err(context.redo_block(basis.planning_counters(), None)),
    };
    let roster = match roster {
        Ok(Some(roster)) => roster,
        Ok(None) => return Ok((context, None)),
        Err(()) => return Err(context.redo_block(basis.planning_counters(), None)),
    };
    let remaining_entries = basis.observed_pages.manifest_budget.remaining();
    let remaining_bytes = context.limits.observation_bytes
        .saturating_sub(context.counters.bytes_observed)
        .saturating_sub(basis.observed_pages.bytes_read)
        .saturating_sub(basis.observed_pages.historical_publication_bytes_read);
    if remaining_entries == 0 || remaining_bytes == 0 {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let format = context.authority.record_format;
    let maximum_entries = context.limits.manifest_entries;
    let maximum_staging = context.limits.staging_bytes;
    let Some(addressed_allowance) = maximum_staging.checked_sub(history.peak_scratch_bytes())
    else { return Err(context.redo_block(basis.planning_counters(), None)); };
    let mut resident = selected_source_inventory::ResidentAllowance::new(addressed_allowance);
    let frame_count = context.integrity.admitted_wal()
        .recoverable_frame_iter(context.selection.wal_tail()).count();
    if resident.entries(frame_count, std::mem::size_of::<ReleasedDropWalFateWitnessV1>()).is_err()
        || resident.entries(basis.sample.wal_members().len(),
            std::mem::size_of::<([u8; 32], u64, u64)>()).is_err()
        || resident.bytes(u64::from(format.page_size().bytes()).saturating_mul(2)).is_err()
    {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let selected_frames = context.integrity.admitted_wal()
        .recoverable_frame_iter(context.selection.wal_tail()).map(|frame| {
            ReleasedDropWalFateWitnessV1::new(
                frame.lsn_start(), frame.lsn_end(),
                frame.identity_digest(), frame.payload_digest(),
            )
        }).collect::<Result<Vec<_>, _>>();
    let Ok(selected_frames) = selected_frames else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    let selected_members = basis.sample.wal_members().iter().map(|member| {
        (member.operation_identity(), member.lsn_range().start().get(),
            member.lsn_range().end_exclusive().get())
    }).collect::<Vec<_>>();
    let mut discovery = context.authority.media
        .bounded_discovery(remaining_entries, remaining_bytes)
        .expect("admitted checkpoint-source observation bounds");
    let observed = observe(
        &mut discovery,
        &context.selection,
        history,
        roster.accumulator().tip().descriptor_record(),
        roster.accumulator().tip().reservation_record(),
        &basis.fates,
        &selected_frames,
        &selected_members,
        basis.sample.policy_identity(),
        format,
        maximum_entries,
        &mut resident,
        remaining_bytes,
        &mut basis.observed_pages.manifest_budget,
        &mut context.integrity_trace,
    );
    let counters = discovery.counters();
    context.authority.media = discovery.finish();
    basis.observed_pages.historical_publication_reads = basis.observed_pages
        .historical_publication_reads.saturating_add(counters.addressed_artifacts_read);
    basis.observed_pages.historical_publication_bytes_read = basis.observed_pages
        .historical_publication_bytes_read.saturating_add(counters.bytes_read);
    let Some((base, peak)) = observed else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    basis.observed_pages.historical_publication_peak_scratch_bytes = basis.observed_pages
        .historical_publication_peak_scratch_bytes.max(peak);
    Ok((context, Some(base)))
}

#[allow(clippy::too_many_arguments)]
fn observe(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    selection: &worth_store_recovery_physics::PhysicalSourceSelection,
    history: &VerifiedOrderedRootHistory,
    tip_descriptor: PersistedRecordIdentity,
    tip_reservation: PersistedRecordIdentity,
    fates: &worth_store_recovery_physics::ReconciledOperationFates,
    selected_frames: &[ReleasedDropWalFateWitnessV1],
    selected_members: &[([u8; 32], u64, u64)],
    policy: [u8; 32],
    format: PhysicalRecordFormatDeclaration,
    maximum_entries: u64,
    resident: &mut selected_source_inventory::ResidentAllowance,
    byte_limit: u64,
    budget: &mut ManifestEntryBudget,
    trace: &mut RecoveryIntegrityIngressTrace,
) -> Option<(VerifiedAddressedCheckpointReleaseBase, u64)> {
    let checkpoint = selection.checkpoint()?;
    let generation = checkpoint.checkpoint().source().root().generation();
    let bytes = discovery
        .read_root_manifest(generation, u64::from(format.page_size().bytes()))
        .ok()?;
    let admitted = admit_addressed_root(
        RecoveryArtifactNamespaceJoin::from_canonical(&bytes),
        discovery.store_identity(), format, generation,
    ).ok()?;
    let (root, observed_format) = admitted.project();
    if observed_format != format
        || <[u8; 32]>::from(Sha256::digest(root.encode(format)))
            != checkpoint.source_root_frame_sha256()
    {
        return None;
    }
    let inventory = selected_source_inventory::observe_with_resident_budget(
        discovery, &root, format, budget, byte_limit, trace, resident,
    ).ok()?;
    let routes = selected_source_inventory::observe_routes_with_resident_budget(
        discovery, &root, format, budget, trace, resident,
    ).ok()?;
    let segment_count = inventory.segment_pages.len() as u64;
    let segment_bytes = segment_count
        .checked_mul(std::mem::size_of::<RecordSegmentPageManifestEntry>() as u64)?;
    if segment_count > maximum_entries || resident.bytes(segment_bytes.checked_mul(2)?).is_err() {
        return None;
    }
    let mut segments = Vec::new();
    segments.try_reserve_exact(inventory.segment_pages.len()).ok()?;
    segments.extend(inventory.segment_pages.values().map(|page| page.entry));
    let mut frame_scratch = 0;
    let checkpoint_roster = super::super::certificates::selected_roster(checkpoint)
        .ok()??;
    let prior_count = checkpoint_roster.batches().len().saturating_sub(1);
    if resident.entries(prior_count, std::mem::size_of::<AddressedCheckpointBatchControl>()).is_err() {
        return None;
    }
    let mut prior_controls = Vec::new();
    prior_controls.try_reserve_exact(prior_count).ok()?;
    for batch in checkpoint_roster.batches().iter().take(checkpoint_roster.batches().len().saturating_sub(1)) {
        let descriptor = read_control(
            discovery, &routes, batch.descriptor_record(), BlobRecordKind::ReclaimDescriptorV3,
            format, budget, trace, &mut frame_scratch, resident,
        )?;
        let BlobRecordV1::ReclaimDescriptorV3(decoded) =
            worth_store_physical_format::decode_blob_record(descriptor.bytes()).ok()?
        else { return None; };
        let reservation = read_control(
            discovery, &routes, batch.reservation_record(), BlobRecordKind::OriginalDropReserved,
            format, budget, trace, &mut frame_scratch, resident,
        )?;
        let manifest = read_control(
            discovery, &routes, decoded.base().manifest_record(), BlobRecordKind::DropSetManifestV3,
            format, budget, trace, &mut frame_scratch, resident,
        )?;
        prior_controls.push(AddressedCheckpointBatchControl::new(
            descriptor, reservation, manifest,
        ));
    }
    let descriptor = read_control(
        discovery, &routes, tip_descriptor, BlobRecordKind::ReclaimDescriptorV3,
        format, budget, trace, &mut frame_scratch, resident,
    )?;
    let reservation = read_control(
        discovery, &routes, tip_reservation, BlobRecordKind::OriginalDropReserved,
        format, budget, trace, &mut frame_scratch, resident,
    )?;
    let BlobRecordV1::ReclaimDescriptorV3(drop) =
        worth_store_physical_format::decode_blob_record(descriptor.bytes()).ok()?
    else { return None; };
    let manifest = read_control(
        discovery, &routes, drop.base().manifest_record(), BlobRecordKind::DropSetManifestV3,
        format, budget, trace, &mut frame_scratch, resident,
    )?;
    let source = ReleasedInventoryView::new(
        &root, &inventory.free_space, &routes, &segments, &inventory.free_entries,
    );
    let checkpoint_catalog_bytes = (checkpoint_roster.batches().len().max(1) as u64)
        .checked_mul((std::mem::size_of::<worth_store_physical_format::ReleaseCheckpointBatchV1>()
            + std::mem::size_of::<worth_store_physical_format::BlobReclaimDescriptorV3>()
            + 128) as u64)?;
    resident.bytes(checkpoint_catalog_bytes
        .checked_add(std::mem::size_of::<VerifiedAddressedCheckpointReleaseBase>() as u64)?).ok()?;
    let available = resident.remaining()
        .checked_add(checkpoint_catalog_bytes)?
        .checked_add(std::mem::size_of::<VerifiedAddressedCheckpointReleaseBase>() as u64)?;
    let base = VerifiedAddressedCheckpointReleaseBase::admit(
        selection, history, source, descriptor, reservation, manifest, prior_controls, fates,
        selected_frames, selected_members, policy,
        format, maximum_entries, available,
    ).ok()?;
    Some((base, history.peak_scratch_bytes().checked_add(resident.used())?))
}

#[allow(clippy::too_many_arguments)]
fn read_control(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    routes: &[CurrentPhysicalRecordPlacement],
    id: PersistedRecordIdentity,
    kind: BlobRecordKind,
    format: PhysicalRecordFormatDeclaration,
    budget: &mut ManifestEntryBudget,
    trace: &mut RecoveryIntegrityIngressTrace,
    scratch: &mut u64,
    resident: &mut selected_source_inventory::ResidentAllowance,
) -> Option<WitnessedSelectedControlFrame> {
    let route = routes.iter().copied().find(|route| {
        route.record() == id && matches!(route,
            CurrentPhysicalRecordPlacement::Extent(extent)
                if extent.content_class() == SelectedRecordContentClass::Blob(kind)
                    && extent.payload_bytes() <= BLOB_CONTROL_FRAME_MAX_BYTES as u64)
    })?;
    if resident.bytes(route.payload_bytes().checked_add(64)?).is_err() {
        return None;
    }
    let (bytes, witness) = record::read_with_witness(
        discovery, format, Some(route), id, BLOB_CONTROL_FRAME_MAX_BYTES as u64,
        budget, trace, scratch,
    ).ok()?;
    WitnessedSelectedControlFrame::from_validated(bytes, witness).ok()
}
