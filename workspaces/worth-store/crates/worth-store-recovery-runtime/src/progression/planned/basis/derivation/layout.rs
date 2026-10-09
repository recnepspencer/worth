use super::super::*;
use super::actions::StagingActionBasis;
use super::materialization::ProjectedMaterializationBasis;
use super::pending::PendingProjectionBasis;

mod base_image;
#[path = "layout/derived_binding.rs"]
mod derived_binding;
mod source_artifacts;

pub(super) fn assemble(
    selection: &PhysicalSourceSelection,
    selected_source: &RecoverySelectedSourceInventory,
    verified_drops: &[worth_store_physical_format::PersistedRecordIdentity],
    validated_manifest_cleanup: Option<crate::orchestration::ValidatedManifestResidueCleanup>,
    release_head_replay: Option<&crate::progression::PendingReleaseReplay>,
    pending: &PendingProjectionBasis<'_>,
    materialization: ProjectedMaterializationBasis<'_>,
    actions: StagingActionBasis,
    allowance: &mut PlanningResidentAllowance,
) -> Result<RecoveryStagingLayoutPlan, ExecutionBasisDenial> {
    if validated_manifest_cleanup.is_some_and(|cleanup| {
        !selection
            .page_facts()
            .placements()
            .iter()
            .any(|placement| placement.record() == cleanup.manifest_record())
            || cleanup.reserved_record().is_some_and(|reserved| {
                !selection
                    .page_facts()
                    .placements()
                    .iter()
                    .any(|placement| placement.record() == reserved)
            })
    }) {
        return Err(ExecutionBasisDenial::Invalid);
    }
    // A WAL-only inline witness may complete a rewritten page's physical slot
    // closure, but it cannot invent the record that a derived retirement drops.
    for projection in &pending.projections {
        let recovery = projection.materialization();
        if let worth_store_physical_format::PersistedPhysicalRecoveryOperation::DerivedDirectory {
            retirement: Some(retirement),
            ..
        } = recovery.operation()
        {
            if retirement.dropped_records().iter().any(|record| {
                !selection
                    .page_facts()
                    .placements()
                    .iter()
                    .any(|selected| selected.record() == *record)
            }) || recovery.placements().iter().any(|placement| {
                retirement
                    .dropped_records()
                    .binary_search(&placement.record())
                    .is_ok()
                    && !matches!(placement, CurrentPhysicalRecordPlacement::Inline(_))
            }) {
                return Err(ExecutionBasisDenial::Invalid);
            }
        }
    }
    let source_artifacts = source_artifacts::collect(selection, selected_source, allowance)?;
    let protected_ranges = source_artifacts::protected_ranges(selection, allowance)?;
    let destination = |row: &(
        PersistedRecordIdentity,
        usize,
        CurrentPhysicalRecordPlacement,
    )| match row.2 {
        CurrentPhysicalRecordPlacement::Extent(extent)
            if !pending.source_copies.iter().any(|copy| {
                copy.recipe().intent().destination().arena_range() == extent.arena_range()
            }) =>
        {
            Some(extent.arena_range())
        }
        _ => None,
    };
    let mut destination_ranges = allowance.reserve(
        materialization
            .placements
            .iter()
            .filter_map(destination)
            .count(),
    )?;
    destination_ranges.extend(materialization.placements.iter().filter_map(destination));
    let commands = super::super::command::exact_commands(
        materialization.frames.iter().map(|row| row.2),
        materialization.manifests.iter().map(|row| row.2),
        &source_artifacts,
        &protected_ranges,
        &destination_ranges,
        allowance,
    )?;
    let range_scratch = PlanningResidentAllowance::vector_bytes(&protected_ranges)?
        .checked_add(PlanningResidentAllowance::vector_bytes(
            &destination_ranges,
        )?)
        .ok_or(ExecutionBasisDenial::Invalid)?;
    drop((protected_ranges, destination_ranges));
    allowance.release(range_scratch);
    let write_bytes = commands.iter().try_fold(0_u64, |bytes, command| {
        bytes.checked_add(command.byte_count())
    });
    let write_bytes = write_bytes.ok_or(ExecutionBasisDenial::Invalid)?;
    let write_bytes = pending
        .source_copies
        .iter()
        .try_fold(write_bytes, |bytes, copy| {
            let intent = copy.recipe().intent();
            bytes
                .checked_add(intent.source().payload_bytes())
                .and_then(|bytes| {
                    bytes.checked_add(
                        u64::from(intent.chunk_count())
                            * (worth_store_physical_format::DURABLE_EXTENT_FRAME_HEADER_BYTES
                                + worth_store_physical_format::EXTENT_CHUNK_METADATA_BYTES)
                                as u64
                            + 104,
                    )
                })
        })
        .ok_or(ExecutionBasisDenial::Invalid)?;
    let count = pending
        .projections
        .iter()
        .try_fold(verified_drops.len(), |count, projection| {
            count.checked_add(
                match projection.materialization().operation() {
                    worth_store_physical_format::PersistedPhysicalRecoveryOperation::DerivedDirectory {
                        retirement: Some(retirement),
                        ..
                    } => retirement.dropped_records().len(),
                    _ => 0,
                },
            )
        })
        .ok_or(ExecutionBasisDenial::Invalid)?;
    let mut all_drops = allowance.reserve(count)?;
    all_drops.extend_from_slice(verified_drops);
    for projection in &pending.projections {
        if let worth_store_physical_format::PersistedPhysicalRecoveryOperation::DerivedDirectory {
            retirement: Some(retirement),
            ..
        } = projection.materialization().operation()
        {
            all_drops.extend_from_slice(retirement.dropped_records());
        }
    }
    all_drops.sort_unstable();
    if all_drops.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(ExecutionBasisDenial::Invalid);
    }
    let base = base_image::assemble(
        selection,
        pending,
        materialization,
        source_artifacts,
        &all_drops,
        validated_manifest_cleanup,
        release_head_replay,
        allowance,
    )?;
    let drop_scratch = PlanningResidentAllowance::vector_bytes(&all_drops)?;
    drop(all_drops);
    allowance.release(drop_scratch);
    let mut source_copies = allowance.reserve(pending.source_copies.len())?;
    source_copies.extend(pending.source_copies.iter().map(|copy| copy.recipe()));
    Ok(RecoveryStagingLayoutPlan {
        source_generation: pending.source_generation,
        staging_generation: pending.staging_generation,
        base,
        actions: allowance.into_box(actions.actions)?,
        commands,
        source_copies: allowance.into_box(source_copies)?,
        allocated_targets: allowance.into_box(actions.allocated_targets)?,
        allocated_bytes: pending.allocated_bytes,
        write_bytes,
    })
}
