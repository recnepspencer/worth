//! The final owned base image, lowered from borrowed materialization tables.

use super::super::super::*;
use super::super::{
    materialization::ProjectedMaterializationBasis, pending::PendingProjectionBasis,
};

pub(super) fn assemble(
    selection: &PhysicalSourceSelection,
    pending: &PendingProjectionBasis<'_>,
    materialization: ProjectedMaterializationBasis<'_>,
    source_artifacts: Box<[RecordArtifactFile]>,
    verified_drops: &[PersistedRecordIdentity],
    cleanup: Option<crate::orchestration::ValidatedManifestResidueCleanup>,
    release_head_replay: Option<&crate::progression::PendingReleaseReplay>,
    allowance: &mut PlanningResidentAllowance,
) -> Result<RecoveryBaseImagePlan, ExecutionBasisDenial> {
    super::super::release_head::require_exact_pending_replay(pending, release_head_replay)?;
    let (latest_blob_publication, derived_family_directory, latest_blob_quarantine) =
        super::derived_binding::projected_derived_binding(
            selection,
            pending,
            verified_drops,
            release_head_replay,
        )?;
    let selected = selection.page_facts().placements();
    let count = selected
        .len()
        .checked_add(materialization.placements.len())
        .ok_or(ExecutionBasisDenial::RecoveryMemoryBytes { observed: u64::MAX })?;
    let mut placements = allowance.reserve(count)?;
    placements.extend(
        selected
            .iter()
            .enumerate()
            .map(|(ordinal, placement)| (placement.record(), ordinal, *placement)),
    );
    placements.extend(
        materialization
            .placements
            .iter()
            .enumerate()
            .map(|(ordinal, row)| (row.0, selected.len() + ordinal, row.2)),
    );
    placements.sort_unstable_by_key(|row| (row.0, row.1));
    retain_latest(&mut placements);
    if verified_drops.iter().any(|record| {
        placements
            .binary_search_by_key(record, |row| row.0)
            .is_err()
    }) {
        return Err(ExecutionBasisDenial::Invalid);
    }
    let manifest = cleanup.map(|cleanup| cleanup.manifest_record());
    let reserved = cleanup.and_then(|cleanup| cleanup.reserved_record());
    placements.retain(|row| retained_after_removals(row.0, verified_drops, manifest, reserved));
    let mut actions = allowance.reserve(placements.len())?;
    actions.extend(placements.iter().enumerate().map(|(ordinal, row)| {
        if materialization
            .projected_records
            .binary_search(&row.0)
            .is_ok()
        {
            RecoveryBaseImageAction::ProjectRecoveryPlacement {
                ordinal: ordinal as u64,
                placement: row.2,
            }
        } else {
            RecoveryBaseImageAction::ReuseImmutableSelectedPlacement {
                ordinal: ordinal as u64,
                placement: row.2,
            }
        }
    }));
    let mut topology = allowance.reserve(selection.page_facts().routing_topology().len())?;
    for (reference, block) in selection.page_facts().routing_topology() {
        let bytes = block
            .owned_heap_bytes()
            .ok_or(ExecutionBasisDenial::Invalid)?;
        topology.push((*reference, allowance.clone_owned(block, bytes)?));
    }
    let mut updates = allowance.reserve(materialization.segment_updates.len())?;
    updates.extend(
        materialization
            .segment_updates
            .iter()
            .enumerate()
            .map(|(ordinal, row)| RecoverySegmentRoutingAction {
                ordinal: ordinal as u64,
                update: row.2,
            }),
    );
    let mut manifests = allowance.reserve(materialization.manifests.len())?;
    manifests.extend(
        materialization
            .manifests
            .iter()
            .enumerate()
            .map(|(ordinal, row)| RecoveryPayloadManifestAction {
                ordinal: ordinal as u64,
                coordinate: row.0,
            }),
    );
    let mut root_states = allowance.reserve(materialization.root_states.len())?;
    for state in &materialization.root_states {
        let bytes = state
            .owned_heap_bytes()
            .ok_or(ExecutionBasisDenial::Invalid)?;
        root_states.push(allowance.clone_owned(*state, bytes)?);
    }
    let replay = match release_head_replay {
        Some(replay) => Some(
            allowance.clone_owned(
                replay,
                replay
                    .owned_heap_bytes()
                    .ok_or(ExecutionBasisDenial::Invalid)?,
            )?,
        ),
        None => None,
    };
    let scratch = PlanningResidentAllowance::vector_bytes(&placements)?
        .checked_add(materialization_backing(&materialization)?)
        .ok_or(ExecutionBasisDenial::Invalid)?;
    drop((placements, materialization));
    allowance.release(scratch);
    Ok(RecoveryBaseImagePlan {
        selected_selector: selection.root().selected().selector(),
        selected_root: selection.root().selected().manifest().clone(),
        latest_blob_publication,
        latest_blob_quarantine,
        tier_epoch_anchor: selection.root().selected().manifest().tier_epoch_anchor(),
        derived_family_directory,
        selected_root_topology: allowance.into_box(topology)?,
        destination_generation: pending.staging_generation,
        actions: allowance.into_box(actions)?,
        segment_updates: allowance.into_box(updates)?,
        manifests: allowance.into_box(manifests)?,
        root_states: allowance.into_box(root_states)?,
        release_head_replay: replay,
        source_artifacts,
    })
}

fn materialization_backing(
    basis: &ProjectedMaterializationBasis<'_>,
) -> Result<u64, ExecutionBasisDenial> {
    let arrays = [
        PlanningResidentAllowance::vector_bytes(&basis.frames)?,
        PlanningResidentAllowance::vector_bytes(&basis.placements)?,
        PlanningResidentAllowance::vector_bytes(&basis.projected_records)?,
        PlanningResidentAllowance::vector_bytes(&basis.segment_updates)?,
        PlanningResidentAllowance::vector_bytes(&basis.manifests)?,
        PlanningResidentAllowance::vector_bytes(&basis.root_states)?,
    ];
    arrays
        .into_iter()
        .try_fold(0_u64, |bytes, next| bytes.checked_add(next))
        .ok_or(ExecutionBasisDenial::Invalid)
}

fn retain_latest(
    rows: &mut Vec<(
        PersistedRecordIdentity,
        usize,
        CurrentPhysicalRecordPlacement,
    )>,
) {
    let mut write = 0;
    for read in 0..rows.len() {
        if write > 0 && rows[write - 1].0 == rows[read].0 {
            rows[write - 1] = rows[read];
        } else {
            rows[write] = rows[read];
            write += 1;
        }
    }
    rows.truncate(write);
}

fn retained_after_removals(
    record: PersistedRecordIdentity,
    drops: &[PersistedRecordIdentity],
    manifest: Option<PersistedRecordIdentity>,
    reserved: Option<PersistedRecordIdentity>,
) -> bool {
    drops.binary_search(&record).is_err() && manifest != Some(record) && reserved != Some(record)
}

#[cfg(test)]
mod manifest_cleanup_tests {
    use super::*;
    fn record(ordinal: u64) -> PersistedRecordIdentity {
        PersistedRecordIdentity::new([7; 16], ordinal).unwrap()
    }
    #[test]
    fn manifest_cleanup_unroutes_exactly_one_id_and_keeps_manifest_payload_records() {
        let manifest = record(4);
        let payload = [record(1), record(2), record(3)];
        let remaining = payload
            .into_iter()
            .chain([manifest])
            .filter(|identity| retained_after_removals(*identity, &[], Some(manifest), None))
            .collect::<Vec<_>>();
        assert_eq!(remaining, payload.to_vec());
        assert!(retained_after_removals(
            record(5),
            &[],
            Some(manifest),
            None
        ));
        assert!(!retained_after_removals(
            record(5),
            &[],
            Some(manifest),
            Some(record(5))
        ));
    }
}
