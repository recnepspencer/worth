use super::super::{context::PlanningContext, resolved_basis::ResolvedPlanningBasis};
use crate::orchestration::reader_limit::UNCOUNTED_READS;
use crate::orchestration::source_copy::SourceCopyCursor;
use worth_store::physical_runtime::{ReadGrant, UnchargedRead};
use worth_store_physical_format::{CurrentPhysicalRecordPlacement, RecordFrameCoordinate};
use worth_store_recovery_physics::RecoveryOperationFate;
#[path = "source_copy/wal_evidence.rs"]
mod wal_evidence;

/// Validate the entire bounded source before any replay write. A selected
/// successor additionally must contain every exact regenerated destination byte.
pub(super) fn verify(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
) -> Result<PlanningContext, crate::entry::PhysicalRecoveryOutcome> {
    let published_evidence = match wal_evidence::verify(&context, basis) {
        Ok(evidence) => evidence,
        Err(()) => return Err(context.redo_block(basis.planning_counters(), None)),
    };
    let copies = basis
        .redo
        .source_copies()
        .iter()
        .filter(|copy| copy.fate() == RecoveryOperationFate::Indeterminate)
        .cloned()
        .collect::<Vec<_>>();
    if copies.is_empty() {
        return Ok(context);
    }
    let selected = context
        .selection
        .root()
        .selected()
        .selector()
        .root_generation();
    for copy in &copies {
        let result = copy
            .projection()
            .source_root_generation()
            .checked_add(1)
            .expect("admitted SourceCopy successor generation");
        if selected > result
            && !published_evidence.contains(&(copy.operation(), copy.publication_lsn()))
        {
            let destination = copy.recipe().intent().destination();
            context = super::historical_publication::observe(
                context,
                basis,
                result,
                destination.record(),
                |_, _, route, _, _, _| {
                    (route == Some(CurrentPhysicalRecordPlacement::Extent(destination)))
                        .then_some(())
                        .ok_or(super::historical_publication::HistoricalFailure::Invalid)
                },
            )?
            .0;
        }
    }
    let budget = copies
        .iter()
        .filter(|copy| !published_evidence.contains(&(copy.operation(), copy.publication_lsn())))
        .try_fold(0_u64, |bytes, copy| {
            let intent = copy.recipe().intent();
            let source_root = copy.projection().source_root_generation();
            let comparisons = if selected > source_root { 2 } else { 1 };
            let frame_bytes = intent
                .source()
                .payload_bytes()
                .checked_add(
                    u64::from(intent.chunk_count())
                        * (worth_store_physical_format::DURABLE_EXTENT_FRAME_HEADER_BYTES
                            + worth_store_physical_format::EXTENT_CHUNK_METADATA_BYTES)
                            as u64,
                )?
                .checked_add(104)?;
            bytes.checked_add(frame_bytes.checked_mul(comparisons)?)
        });
    let Some(total_io_bytes) = budget else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    let byte_limit = context
        .limits
        .observation_bytes
        .saturating_sub(context.counters.bytes_observed)
        .saturating_sub(basis.observed_pages.bytes_read)
        .saturating_sub(basis.observed_pages.candidate_bytes_read)
        .saturating_sub(basis.observed_pages.source_copy_bytes_read)
        .saturating_sub(basis.observed_pages.historical_publication_bytes_read);
    if byte_limit < total_io_bytes {
        // The copy was left `byte_limit` of recovery's observation bytes.
        let limit = crate::orchestration::recovery_budget::RecoveryAllowance::declared(
            &context.limits,
            crate::entry::PhysicalRecoveryLimitDimension::ObservationBytes,
        )
        .beside(total_io_bytes, byte_limit)
        .map(Into::into);
        return Err(context.redo_block(basis.planning_counters(), limit));
    }
    let format = context.authority.record_format;
    let store = context.authority.media.store_identity();
    // The frames' bytes were refused above against what the earlier phases
    // left of recovery's observation bytes; the copy keeps no count of its
    // own. The frames are streamed, not retained as an object.
    let mut discovery = context
        .authority
        .media
        .bounded_discovery(UNCOUNTED_READS, total_io_bytes)
        .expect("a reader that counts no reads opens on any byte bound");
    let result = (|| {
        for copy in &copies {
            let recipe = copy.recipe();
            let intent = recipe.intent();
            let source_generation = copy.projection().source_root_generation();
            let resulting_generation = source_generation.checked_add(1).ok_or(())?;
            let resolved = published_evidence.contains(&(copy.operation(), copy.publication_lsn()));
            let historical = selected > resulting_generation;
            if historical && resolved {
                // A durable Published resolution has released source custody.
                continue;
            }
            let published = selected >= resulting_generation;
            if !historical {
                let expected = if selected == source_generation {
                    intent.source()
                } else if published {
                    intent.destination()
                } else {
                    return Err(());
                };
                if !context
                    .selection
                    .page_facts()
                    .placements()
                    .contains(&CurrentPhysicalRecordPlacement::Extent(expected))
                {
                    return Err(());
                }
            }
            if resolved {
                if !published {
                    return Err(());
                }
                // A resolved copy may legitimately have released its source.
                // The selected destination was admitted by the root walk.
                continue;
            }
            let source = intent.source().arena_range();
            let observed = discovery
                .read_extent_manifest(source, ReadGrant::ceiling_only())
                .observed()
                .map_err(|_| ())?;
            let mut cursor = SourceCopyCursor::open(
                store,
                format,
                recipe,
                &observed,
                &mut context.integrity_trace,
            )?;
            drop(observed);
            while let Some(coordinate) = cursor.source_coordinate()? {
                let observed = discovery
                    .read_extent_range(
                        source,
                        coordinate.offset() - source.offset(),
                        coordinate.length(),
                        ReadGrant::ceiling_only(),
                    )
                    .observed()
                    .map_err(|_| ())?;
                let (destination, bytes) =
                    cursor.transform(&observed, &mut context.integrity_trace)?;
                if published {
                    compare(
                        &mut discovery,
                        intent.destination().arena_range(),
                        destination,
                        &bytes,
                    )?;
                }
            }
            let (destination, bytes) = cursor.finish()?;
            if published {
                compare(
                    &mut discovery,
                    intent.destination().arena_range(),
                    destination,
                    &bytes,
                )?;
            }
        }
        Ok::<_, ()>(())
    })();
    let counters = discovery.counters();
    context.authority.media = discovery.finish();
    basis.observed_pages.source_copy_reads = basis
        .observed_pages
        .source_copy_reads
        .saturating_add(counters.addressed_artifacts_read);
    basis.observed_pages.source_copy_bytes_read = basis
        .observed_pages
        .source_copy_bytes_read
        .saturating_add(counters.bytes_read);
    basis.observed_pages.source_copy_peak_scratch_bytes =
        basis.observed_pages.source_copy_peak_scratch_bytes.max(
            copies
                .iter()
                .filter(|copy| {
                    !published_evidence.contains(&(copy.operation(), copy.publication_lsn()))
                })
                .map(|copy| u64::from(copy.recipe().intent().maximum_frame_bytes()) * 4)
                .max()
                .unwrap_or(0),
        );
    if result.is_err() {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    Ok(context)
}

fn compare(
    discovery: &mut worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery,
    range: worth_store_physical_format::ExtentArenaRange,
    coordinate: RecordFrameCoordinate,
    expected: &[u8],
) -> Result<(), ()> {
    let observed = discovery
        .read_extent_range(
            range,
            coordinate.offset() - range.offset(),
            coordinate.length(),
            ReadGrant::ceiling_only(),
        )
        .observed()
        .map_err(|_| ())?;
    if observed.bytes() == Some(expected) {
        Ok(())
    } else {
        Err(())
    }
}
