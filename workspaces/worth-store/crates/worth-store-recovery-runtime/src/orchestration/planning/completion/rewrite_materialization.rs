use sha2::{Digest, Sha256};
use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    decode_data_frame_page_lsn, encode_data_frame_page_lsn, inspect_inline_page,
    inspect_inline_page_records, restamp_inline_page_generation, CurrentPhysicalRecordPlacement,
    DurableFrameKind, DurableInlineRecordPlacement, PersistedInlineSegmentAllocation,
    PersistedPhysicalDataFrameSubject, PersistedPhysicalRecoveryFrame,
    PersistedPhysicalRecoveryProjection, PersistedPhysicalRecoveryRootState,
    PersistedRecordIdentity, PhysicalGeneration, PhysicalGenerationAuthority, PhysicalPageId,
    PhysicalPageLsn, PhysicalRecordFormatDeclaration, PhysicalSegmentId, RecordArtifactFile,
    RecordFrameCoordinate, RecordSegmentPageManifestEntry,
};
use worth_store_recovery_physics::{
    PhysicalRedoProjection, PhysicalRewriteAdmission, RecoveryOperationFate,
};

use super::super::context::PlanningContext;
use super::super::resolved_basis::ResolvedPlanningBasis;

#[path = "rewrite_extent.rs"]
mod rewrite_extent;
#[path = "rewrite_span.rs"]
mod rewrite_span;

pub(super) fn install(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
) -> Result<PlanningContext, crate::entry::PhysicalRecoveryOutcome> {
    let pending: Vec<PhysicalRewriteAdmission> = basis
        .redo
        .rewrite_admissions()
        .iter()
        .copied()
        .filter(|admission| admission.fate() == RecoveryOperationFate::Indeterminate)
        .collect();
    if pending.is_empty() {
        return Ok(context);
    }
    let format = context.authority.record_format;
    let byte_limit = context.limits.observation_bytes;
    let media = context.authority.media;
    let mut discovery = media
        .bounded_discovery(64, byte_limit)
        .expect("admitted nonzero recovery limits create a bounded planning reader");
    let mut applying = Vec::new();
    for admission in pending {
        match rewrite_disposition(
            &mut discovery,
            &context.selection,
            format,
            byte_limit,
            admission,
        ) {
            Ok(RewriteDisposition::Published) => {}
            Ok(RewriteDisposition::Apply) => applying.push(admission),
            Err(()) => {
                context.authority.media = discovery.finish();
                return Err(context.redo_block(basis.planning_counters(), None));
            }
        }
    }
    if applying.is_empty() {
        context.authority.media = discovery.finish();
        return Ok(context);
    }
    let built = applying
        .into_iter()
        .map(|admission| {
            project_rewrite(
                &mut discovery,
                &context.selection,
                &basis.observed_pages.selected_source,
                format,
                byte_limit,
                admission,
            )
        })
        .collect::<Result<Vec<_>, ()>>();
    context.authority.media = discovery.finish();
    let projections = match built {
        Ok(projections) => projections,
        Err(()) => return Err(context.redo_block(basis.planning_counters(), None)),
    };
    for projection in projections {
        basis.redo.install_rewrite_materialization(projection);
    }
    Ok(context)
}

enum RewriteDisposition {
    Apply,
    Published,
}

fn rewrite_disposition(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    selection: &worth_store_recovery_physics::PhysicalSourceSelection,
    format: PhysicalRecordFormatDeclaration,
    byte_limit: u64,
    admission: PhysicalRewriteAdmission,
) -> Result<RewriteDisposition, ()> {
    let rewrite = admission.redo();
    let selected = selection.root().selected().selector().root_generation();
    if rewrite.resulting_root_generation() == selected {
        prove_published_rewrite(discovery, selection, format, byte_limit, admission)?;
        return Ok(RewriteDisposition::Published);
    }
    if rewrite.source_root_generation() == selected
        && rewrite.resulting_root_generation() == selected.saturating_add(1)
    {
        return Ok(RewriteDisposition::Apply);
    }
    Err(())
}

fn prove_published_rewrite(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    selection: &worth_store_recovery_physics::PhysicalSourceSelection,
    format: PhysicalRecordFormatDeclaration,
    byte_limit: u64,
    admission: PhysicalRewriteAdmission,
) -> Result<(), ()> {
    let rewrite = admission.redo();
    if let Some(placement) = rewrite_extent::selected_source(selection, rewrite) {
        return rewrite_extent::prove(discovery, format, byte_limit, rewrite, placement);
    }
    let page_bytes = format.page_size().bytes();
    if rewrite.destination_offset() != 0
        || rewrite.destination_length() != page_bytes
        || rewrite.source_length() != page_bytes
    {
        return rewrite_span::prove(discovery, selection, format, byte_limit, admission);
    }
    let record = decode_record(rewrite.record_identity())?;
    let inline = selection
        .page_facts()
        .placements()
        .iter()
        .copied()
        .find_map(|placement| match placement {
            CurrentPhysicalRecordPlacement::Inline(inline)
                if inline.record() == record
                    && inline.page_generation() == rewrite.destination_placement()
                    && inline.segment_generation() == rewrite.destination_generation() =>
            {
                Some(inline)
            }
            _ => None,
        })
        .ok_or(())?;
    let source_page = discovery
        .read_segment_range(
            inline.segment().get(),
            rewrite.source_generation(),
            rewrite.source_offset(),
            rewrite.source_length(),
            byte_limit,
        )
        .map_err(|_| ())?
        .into_bytes()
        .ok_or(())?;
    let source_digest: [u8; 32] = Sha256::digest(&source_page).into();
    if source_digest != rewrite.source_digest() {
        return Err(());
    }
    let mut expected =
        restamp_inline_page_generation(format, &source_page, rewrite.destination_placement())
            .map_err(|_| ())?;
    encode_data_frame_page_lsn(
        &mut expected,
        DurableFrameKind::InlinePage,
        PhysicalPageLsn::new(rewrite.page_lsn()),
    )
    .map_err(|_| ())?;
    let page = discovery
        .read_segment_range(
            inline.segment().get(),
            rewrite.destination_generation(),
            rewrite.destination_offset(),
            rewrite.destination_length(),
            byte_limit,
        )
        .map_err(|_| ())?
        .into_bytes()
        .ok_or(())?;
    if page != expected {
        return Err(());
    }
    let geometry = inspect_inline_page(format, &page).map_err(|_| ())?;
    if geometry.page_cell() != inline.page_cell() {
        return Err(());
    }
    let page_lsn =
        decode_data_frame_page_lsn(&page, DurableFrameKind::InlinePage).map_err(|_| ())?;
    if page_lsn.get() != rewrite.page_lsn() {
        return Err(());
    }
    let records = inspect_inline_page_records(format, &page).map_err(|_| ())?;
    if !records.iter().any(|found| found.record() == record) {
        return Err(());
    }
    Ok(())
}

fn project_rewrite(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    selection: &worth_store_recovery_physics::PhysicalSourceSelection,
    source: &crate::progression::RecoverySelectedSourceInventory,
    format: PhysicalRecordFormatDeclaration,
    byte_limit: u64,
    admission: PhysicalRewriteAdmission,
) -> Result<PhysicalRedoProjection, ()> {
    let rewrite = admission.redo();
    if let Some(extent) = rewrite_extent::selected_source(selection, rewrite) {
        return rewrite_extent::project(
            discovery, selection, format, byte_limit, admission, extent,
        );
    }
    let page_bytes = format.page_size().bytes();
    let selected_generation = selection.root().selected().selector().root_generation();
    if rewrite.destination_offset() != 0
        || rewrite.destination_length() != page_bytes
        || rewrite.source_length() != page_bytes
    {
        return rewrite_span::project(discovery, selection, source, format, byte_limit, admission);
    }
    if rewrite.source_root_generation() != selected_generation
        || rewrite.resulting_root_generation() != selected_generation.saturating_add(1)
    {
        return Err(());
    }
    let record = decode_record(rewrite.record_identity())?;
    let placements = selection.page_facts().placements();
    let inline = placements
        .iter()
        .copied()
        .find_map(|placement| match placement {
            CurrentPhysicalRecordPlacement::Inline(inline)
                if inline.record() == record
                    && inline.page_generation() == rewrite.source_placement()
                    && inline.segment_generation() == rewrite.source_generation() =>
            {
                Some(inline)
            }
            _ => None,
        })
        .ok_or(())?;
    let selected = source
        .segment_pages
        .get(&(inline.segment().get(), inline.page().get()))
        .copied()
        .ok_or(())?;
    let entry = selected.entry;
    let page_bytes = u64::from(format.page_size().bytes());
    if entry.page_generation() != rewrite.source_placement()
        || entry.data_generation() != rewrite.source_generation()
        || entry.data_page_count() == 0
        || entry.frame_index() >= entry.data_page_count()
        || u64::from(entry.frame_index()) * page_bytes != rewrite.source_offset()
    {
        return Err(());
    }
    let page = discovery
        .read_segment_range(
            inline.segment().get(),
            rewrite.source_generation(),
            rewrite.source_offset(),
            rewrite.source_length(),
            byte_limit,
        )
        .map_err(|_| ())?
        .into_bytes()
        .ok_or(())?;
    if page.len() != rewrite.source_length() as usize {
        return Err(());
    }
    let digest: [u8; 32] = Sha256::digest(&page).into();
    if digest != rewrite.source_digest() {
        return Err(());
    }
    let mut restamped =
        restamp_inline_page_generation(format, &page, rewrite.destination_placement())
            .map_err(|_| ())?;
    encode_data_frame_page_lsn(
        &mut restamped,
        DurableFrameKind::InlinePage,
        PhysicalPageLsn::new(rewrite.page_lsn()),
    )
    .map_err(|_| ())?;
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let segment_id = PhysicalSegmentId::from_raw(inline.segment().get()).map_err(|_| ())?;
    let page_id = PhysicalPageId::from_raw(inline.page().get()).map_err(|_| ())?;
    let destination_page = authority
        .page_cell(segment_id, page_id)
        .with_page_generation(
            PhysicalGeneration::from_raw(rewrite.destination_placement()).map_err(|_| ())?,
        );
    let destination_segment = authority.segment_cell(segment_id).with_segment_generation(
        PhysicalGeneration::from_raw(rewrite.destination_generation()).map_err(|_| ())?,
    );
    let coordinate = RecordFrameCoordinate::new(
        RecordArtifactFile::Segment {
            segment: inline.segment().get(),
            generation: rewrite.destination_generation(),
        },
        rewrite.destination_offset(),
        rewrite.destination_length(),
    )
    .ok_or(())?;
    let frame = PersistedPhysicalRecoveryFrame::new(
        PersistedPhysicalDataFrameSubject::InlinePage(destination_page),
        coordinate,
        &restamped,
    )
    .ok_or(())?;
    let mut rebound = Vec::new();
    for placement in placements {
        let CurrentPhysicalRecordPlacement::Inline(existing) = placement else {
            continue;
        };
        if existing.page_cell() != inline.page_cell() {
            continue;
        }
        rebound.push(
            DurableInlineRecordPlacement::new(
                existing.record(),
                destination_segment,
                destination_page,
                existing.slot_cell(),
                existing.segment_page_capacity(),
                existing.payload_bytes(),
            )
            .ok_or(())?,
        );
    }
    rebound.sort_by_key(|placement| placement.record());
    if rebound.is_empty() {
        return Err(());
    }
    let capacity = inline.segment_page_capacity();
    let allocation =
        PersistedInlineSegmentAllocation::new(destination_segment, capacity, 1).ok_or(())?;
    let update = RecordSegmentPageManifestEntry::new(destination_page, destination_segment, 1, 0)
        .ok_or(())?;
    let root_state = PersistedPhysicalRecoveryRootState::new(
        rewrite.candidate_bytes(),
        1,
        selection.root().selected().manifest().node_capacity(),
        vec![allocation],
        Some(record),
        Some(destination_segment),
    )
    .ok_or(())?;
    let projection = PersistedPhysicalRecoveryProjection::new(
        rewrite.source_root_generation(),
        root_state,
        rebound.iter().map(|placement| placement.record()).collect(),
        vec![frame],
        rebound
            .into_iter()
            .map(CurrentPhysicalRecordPlacement::Inline)
            .collect(),
        vec![update],
        Vec::new(),
    )
    .ok_or(())?;
    Ok(PhysicalRedoProjection::from_rewrite_materialization(
        admission.operation(),
        admission.group(),
        admission.fate(),
        projection,
    ))
}

fn decode_record(bytes: [u8; 32]) -> Result<PersistedRecordIdentity, ()> {
    let mut epoch = [0; 16];
    epoch.copy_from_slice(&bytes[..16]);
    let ordinal = u64::from_le_bytes(bytes[16..24].try_into().map_err(|_| ())?);
    PersistedRecordIdentity::new(epoch, ordinal).ok_or(())
}
