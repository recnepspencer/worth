//! Selected-source exclusivity. Every selected incoming blob edge is checked
//! before the manifest's identities can become a recovery removal plan.

use worth_store_physical_format::{
    decode_blob_record, BlobRecordV1, BlobSessionDeclarationV1, CurrentPhysicalRecordPlacement,
    DropSetManifestV1, PersistedRecordIdentity, BLOB_TREE_NODE_FRAME_MAX_BYTES,
};

use super::super::super::{context::PlanningContext, resolved_basis::ResolvedPlanningBasis};
use super::super::historical_publication::HistoricalFailure;
use super::record;

#[path = "selected/record_semantics.rs"]
mod record_semantics;
#[path = "selected/reservation.rs"]
mod reservation;
pub(super) use record_semantics::{blob_store, incoming_edge};
use record_semantics::{publication_conflict, source_identity_conflict};
pub(super) use reservation::ExpectedReserved;

pub(super) fn verify_source(
    context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    manifest: &DropSetManifestV1,
    declaration: BlobSessionDeclarationV1,
    descriptor_record: PersistedRecordIdentity,
    reservation: Option<ExpectedReserved>,
) -> Result<(PlanningContext, Vec<PersistedRecordIdentity>), crate::entry::PhysicalRecoveryOutcome>
{
    scan_source(
        context,
        basis,
        manifest,
        declaration,
        SourceInspectionKind::DropDescriptor {
            descriptor: descriptor_record,
            reservation,
        },
    )
}

pub(super) fn verify_manifest_residue(
    context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    manifest: &DropSetManifestV1,
    declaration: BlobSessionDeclarationV1,
    manifest_record: PersistedRecordIdentity,
    reserved_record: Option<PersistedRecordIdentity>,
) -> Result<PlanningContext, crate::entry::PhysicalRecoveryOutcome> {
    scan_source(
        context,
        basis,
        manifest,
        declaration,
        SourceInspectionKind::ManifestResidue {
            manifest: manifest_record,
            reserved: reserved_record,
        },
    )
    .map(|(context, _)| context)
}

#[derive(Clone, Copy)]
enum SourceInspectionKind {
    DropDescriptor {
        descriptor: PersistedRecordIdentity,
        reservation: Option<ExpectedReserved>,
    },
    ManifestResidue {
        manifest: PersistedRecordIdentity,
        reserved: Option<PersistedRecordIdentity>,
    },
}

impl SourceInspectionKind {
    fn conflicting_record(
        self,
        record: PersistedRecordIdentity,
        fact: &BlobRecordV1<'_>,
        attempt: [u8; 16],
    ) -> bool {
        match self {
            Self::DropDescriptor { descriptor, .. } => record == descriptor,
            Self::ManifestResidue { manifest, reserved } => {
                matches!(fact, BlobRecordV1::ReclaimDescriptor(value) if value.reclaim_attempt() == attempt || value.manifest_record() == manifest)
                    || matches!(fact, BlobRecordV1::ReclaimDescriptorV2(value) if value.reclaim_attempt() == attempt || value.manifest_record() == manifest)
                    || matches!(fact, BlobRecordV1::ReclaimDescriptorV3(value) if value.base().reclaim_attempt() == attempt || value.base().manifest_record() == manifest)
                    || matches!(fact, BlobRecordV1::DropSetManifest(value) if value.reclaim_attempt() == attempt && record != manifest)
                    || matches!(fact, BlobRecordV1::DropSetManifestV2(value) if value.reclaim_attempt() == attempt && record != manifest)
                    || matches!(fact, BlobRecordV1::DropSetManifestV3(value) if value.reclaim_attempt() == attempt && record != manifest)
                    || matches!(fact, BlobRecordV1::OriginalDropReserved(value)
                        if (value.reclaim_attempt() == attempt || value.manifest_record() == manifest)
                            && reserved != Some(record))
            }
        }
    }

    fn selected_conflict(
        self,
        record: PersistedRecordIdentity,
        fact: &BlobRecordV1<'_>,
        manifest: &DropSetManifestV1,
        declaration: BlobSessionDeclarationV1,
        candidate: Option<usize>,
        seen: &[bool],
    ) -> bool {
        match self {
            Self::DropDescriptor { .. } => {
                incoming_edge(fact, manifest.dropped())
                    || publication_conflict(fact, declaration)
                    || source_identity_conflict(fact, record, manifest, declaration)
                    || candidate.is_some_and(|ordinal| {
                        seen[ordinal] || !candidate_kind(fact, manifest, declaration)
                    })
                    || self.conflicting_record(record, fact, manifest.reclaim_attempt())
            }
            Self::ManifestResidue {
                manifest: manifest_record,
                reserved,
            } => {
                candidate.is_some()
                    || incoming_edge(fact, &[manifest_record])
                    || reserved.is_some_and(|record| incoming_edge(fact, &[record]))
                    || self.conflicting_record(record, fact, manifest.reclaim_attempt())
            }
        }
    }
}

fn scan_source(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    manifest: &DropSetManifestV1,
    declaration: BlobSessionDeclarationV1,
    kind: SourceInspectionKind,
) -> Result<(PlanningContext, Vec<PersistedRecordIdentity>), crate::entry::PhysicalRecoveryOutcome>
{
    let selected = context.selection.page_facts().placements();
    let remaining_entries = basis.observed_pages.manifest_budget.remaining();
    let remaining_bytes = context
        .limits
        .observation_bytes
        .saturating_sub(context.counters.bytes_observed)
        .saturating_sub(basis.observed_pages.bytes_read)
        .saturating_sub(basis.observed_pages.candidate_bytes_read)
        .saturating_sub(basis.observed_pages.source_copy_bytes_read)
        .saturating_sub(basis.observed_pages.historical_publication_bytes_read);
    if remaining_entries == 0 || remaining_bytes == 0 {
        return Err(denial(
            context,
            basis,
            if remaining_entries == 0 {
                HistoricalFailure::ManifestEntries
            } else {
                HistoricalFailure::ObservationBytes(1)
            },
            remaining_bytes,
        ));
    }
    let format = context.authority.record_format;
    let mut discovery = context
        .authority
        .media
        .bounded_discovery(remaining_entries, remaining_bytes)
        .expect("nonzero selected reclaim scan limits");
    let descriptor_candidate_count = match kind {
        SourceInspectionKind::DropDescriptor { .. } => manifest.dropped().len(),
        SourceInspectionKind::ManifestResidue { .. } => 0,
    };
    let mut seen = vec![false; descriptor_candidate_count];
    let mut candidate_chunk_ordinals = vec![None; descriptor_candidate_count];
    let mut selected_frontier_end = 0_u64;
    let mut reservation_seen = false;
    let mut scratch = 0_u64;
    let mut result = Ok(());
    for placement in selected.iter().copied() {
        let record_id = placement.record();
        let candidate = manifest.dropped().binary_search(&record_id).ok();
        let CurrentPhysicalRecordPlacement::Extent(extent) = placement else {
            if candidate.is_some() {
                result = Err(HistoricalFailure::Invalid);
                break;
            }
            continue;
        };
        if extent.payload_bytes() > BLOB_TREE_NODE_FRAME_MAX_BYTES as u64 {
            if candidate.is_some() {
                result = Err(HistoricalFailure::Invalid);
                break;
            }
            continue;
        }
        let bytes = match record::read(
            &mut discovery,
            format,
            Some(placement),
            record_id,
            BLOB_TREE_NODE_FRAME_MAX_BYTES as u64,
            &mut basis.observed_pages.manifest_budget,
            &mut context.integrity_trace,
            &mut scratch,
        ) {
            Ok(bytes) => bytes,
            Err(failure) => {
                result = Err(failure);
                break;
            }
        };
        if !bytes.starts_with(b"WRC11BLB") {
            if candidate.is_some() {
                result = Err(HistoricalFailure::Invalid);
                break;
            }
            continue;
        }
        let Ok(fact) = decode_blob_record(&bytes) else {
            result = Err(HistoricalFailure::Invalid);
            break;
        };
        if blob_store(&fact) != discovery.store_identity().bytes() {
            result = Err(HistoricalFailure::Invalid);
            break;
        }
        if let BlobRecordV1::TreeNode(node) = &fact {
            // The decoder temporarily owns its canonical content and entry
            // vector while the authenticated C.5 payload remains live.
            let decoder = node.entries().len()
                * (std::mem::size_of::<worth_store_physical_format::BlobTreeEntryV1>() + 64);
            scratch = scratch.max(
                bytes.capacity() as u64 + decoder as u64 + u64::from(format.page_size().bytes()),
            );
        }
        if let BlobRecordV1::SessionFrontier(frontier) = &fact {
            if frontier.session() == manifest.source_basis().session() {
                selected_frontier_end = selected_frontier_end.max(frontier.next_chunk_ordinal());
            }
        }
        if let BlobRecordV1::OriginalDropReserved(value) = &fact {
            if let SourceInspectionKind::DropDescriptor { reservation, .. } = kind {
                let related = value.reclaim_attempt() == manifest.reclaim_attempt()
                    || reservation.is_some_and(|expected| {
                        value.manifest_record() == expected.manifest_record()
                    });
                if related {
                    if reservation_seen
                        || !reservation.is_some_and(|expected| expected.matches(*value))
                    {
                        result = Err(HistoricalFailure::Invalid);
                        break;
                    }
                    reservation_seen = true;
                }
            }
        }
        if kind.selected_conflict(record_id, &fact, manifest, declaration, candidate, &seen) {
            result = Err(HistoricalFailure::Invalid);
            break;
        }
        if let Some(ordinal) = candidate {
            seen[ordinal] = true;
            if let BlobRecordV1::Chunk(chunk) = &fact {
                candidate_chunk_ordinals[ordinal] = Some(chunk.occurrence().ordinal());
            }
        }
    }
    let counters = discovery.counters();
    context.authority.media = discovery.finish();
    basis.observed_pages.historical_publication_reads = basis
        .observed_pages
        .historical_publication_reads
        .saturating_add(counters.addressed_artifacts_read);
    basis.observed_pages.historical_publication_bytes_read = basis
        .observed_pages
        .historical_publication_bytes_read
        .saturating_add(counters.bytes_read);
    basis
        .observed_pages
        .historical_publication_peak_scratch_bytes = basis
        .observed_pages
        .historical_publication_peak_scratch_bytes
        .max(
            scratch
                + seen.capacity() as u64
                + (candidate_chunk_ordinals.capacity() * std::mem::size_of::<Option<u64>>()) as u64,
        );
    if let Err(failure) = result {
        return Err(denial(context, basis, failure, remaining_bytes));
    }
    if matches!(
        kind,
        SourceInspectionKind::DropDescriptor {
            reservation: Some(_),
            ..
        }
    ) && !reservation_seen
    {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    if candidate_coverage_conflicts(
        kind,
        &seen,
        &candidate_chunk_ordinals,
        selected_frontier_end,
    ) {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let dropped = match kind {
        SourceInspectionKind::DropDescriptor { .. } => manifest.dropped().to_vec(),
        SourceInspectionKind::ManifestResidue { .. } => Vec::new(),
    };
    Ok((context, dropped))
}

fn candidate_coverage_conflicts(
    kind: SourceInspectionKind,
    seen: &[bool],
    candidate_chunk_ordinals: &[Option<u64>],
    selected_frontier_end: u64,
) -> bool {
    match kind {
        // A descriptor drop requires the complete candidate set to still be
        // selected; a residue cleanup requires the opposite custody fact.
        SourceInspectionKind::DropDescriptor { .. } => {
            seen.iter().any(|seen| !seen)
                || candidate_chunk_ordinals
                    .iter()
                    .flatten()
                    .any(|ordinal| *ordinal < selected_frontier_end)
        }
        SourceInspectionKind::ManifestResidue { .. } => seen.iter().any(|seen| *seen),
    }
}

#[cfg(test)]
#[path = "selected/tests.rs"]
mod tests;

fn denial(
    context: PlanningContext,
    basis: &ResolvedPlanningBasis,
    failure: HistoricalFailure,
    remaining_bytes: u64,
) -> crate::entry::PhysicalRecoveryOutcome {
    let limit = match failure {
        HistoricalFailure::Invalid => None,
        HistoricalFailure::ManifestEntries => Some(crate::entry::PhysicalRecoveryLimitFailure {
            dimension: crate::entry::PhysicalRecoveryLimitDimension::ManifestEntries,
            observed: context.limits.manifest_entries.saturating_add(1),
            admitted: context.limits.manifest_entries,
        }),
        HistoricalFailure::ObservationBytes(observed) => {
            Some(crate::entry::PhysicalRecoveryLimitFailure {
                dimension: crate::entry::PhysicalRecoveryLimitDimension::ObservationBytes,
                observed: context
                    .limits
                    .observation_bytes
                    .saturating_sub(remaining_bytes)
                    .saturating_add(observed),
                admitted: context.limits.observation_bytes,
            })
        }
    };
    context.redo_block(basis.planning_counters(), limit)
}

fn candidate_kind(
    fact: &BlobRecordV1<'_>,
    manifest: &DropSetManifestV1,
    declaration: BlobSessionDeclarationV1,
) -> bool {
    let source = manifest.source_basis();
    match fact {
        BlobRecordV1::Chunk(chunk) => {
            let occurrence = chunk.occurrence();
            let expected = occurrence
                .ordinal()
                .checked_mul(u64::from(declaration.chunk_size()))
                .and_then(|start| declaration.declared_bytes().checked_sub(start))
                .filter(|remaining| *remaining > 0)
                .map(|remaining| remaining.min(u64::from(declaration.chunk_size())));
            occurrence.session() == source.session()
                && chunk.chunk_size() == declaration.chunk_size()
                && expected == Some(chunk.bytes().len() as u64)
        }
        BlobRecordV1::TreeNode(node) => node.occurrence().session() == source.session(),
        BlobRecordV1::SessionFrontier(frontier) => {
            frontier.session() == source.session()
                && frontier.declaration_record() == source.declaration_record()
                && frontier.declaration_digest() == source.declaration_frame_sha256()
        }
        _ => false,
    }
}
