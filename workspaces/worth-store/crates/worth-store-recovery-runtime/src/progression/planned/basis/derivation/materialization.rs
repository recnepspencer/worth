//! Borrowed, bounded view of projected materialization until the owned image.
//! Explicit source ordinals preserve BTreeMap insertion's last-writer result.

use super::super::frame_identity::frame_identity;
use super::super::*;
use super::pending::PendingProjectionBasis;
use worth_store_physical_format::{
    PersistedPhysicalRecoveryFrame, PersistedPhysicalRecoveryManifest, PersistedRecordIdentity,
    RecordFrameCoordinate,
};

pub(super) struct ProjectedMaterializationBasis<'plan> {
    pub(super) frames: Vec<(
        PhysicalRedoTargetIdentity,
        usize,
        &'plan PersistedPhysicalRecoveryFrame,
    )>,
    pub(super) placements: Vec<(
        PersistedRecordIdentity,
        usize,
        CurrentPhysicalRecordPlacement,
    )>,
    pub(super) projected_records: Vec<PersistedRecordIdentity>,
    pub(super) segment_updates: Vec<((u64, u64), usize, RecordSegmentPageManifestEntry)>,
    pub(super) manifests: Vec<(
        RecordFrameCoordinate,
        usize,
        &'plan PersistedPhysicalRecoveryManifest,
    )>,
    pub(super) root_states: Vec<&'plan PersistedPhysicalRecoveryRootState>,
}

#[derive(Default)]
struct SourceCounts {
    frames: usize,
    placements: usize,
    updates: usize,
    manifests: usize,
    root_states: usize,
}

impl SourceCounts {
    fn add(
        &mut self,
        materialization: &worth_store_physical_format::PersistedPhysicalRecoveryProjection,
    ) -> Result<(), ExecutionBasisDenial> {
        self.frames = self
            .frames
            .checked_add(
                materialization
                    .frames()
                    .ok_or(ExecutionBasisDenial::Invalid)?
                    .len(),
            )
            .ok_or(ExecutionBasisDenial::Invalid)?;
        self.placements = self
            .placements
            .checked_add(materialization.placements().len())
            .ok_or(ExecutionBasisDenial::Invalid)?;
        self.updates = self
            .updates
            .checked_add(materialization.segment_updates().len())
            .ok_or(ExecutionBasisDenial::Invalid)?;
        self.manifests = self
            .manifests
            .checked_add(materialization.manifests().len())
            .ok_or(ExecutionBasisDenial::Invalid)?;
        self.root_states = self
            .root_states
            .checked_add(1)
            .ok_or(ExecutionBasisDenial::Invalid)?;
        Ok(())
    }
}

pub(super) fn collect<'plan>(
    pending: &PendingProjectionBasis<'plan>,
    allowance: &mut PlanningResidentAllowance,
) -> Result<ProjectedMaterializationBasis<'plan>, ExecutionBasisDenial> {
    let mut counts = SourceCounts::default();
    for projection in &pending.projections {
        counts.add(projection.materialization())?;
    }
    for copy in &pending.source_copies {
        counts.placements = counts
            .placements
            .checked_add(copy.projection().placements().len())
            .ok_or(ExecutionBasisDenial::Invalid)?;
        counts.root_states = counts
            .root_states
            .checked_add(1)
            .ok_or(ExecutionBasisDenial::Invalid)?;
    }
    // The projected-record union cannot exceed all placement observations.
    // Reserve every backing before filling any table; push never grows.
    let mut basis = ProjectedMaterializationBasis {
        frames: allowance.reserve(counts.frames)?,
        placements: allowance.reserve(counts.placements)?,
        projected_records: allowance.reserve(counts.placements)?,
        segment_updates: allowance.reserve(counts.updates)?,
        manifests: allowance.reserve(counts.manifests)?,
        root_states: allowance.reserve(counts.root_states)?,
    };
    for projection in &pending.projections {
        let projection: &'plan worth_store_recovery_physics::PhysicalRedoProjection = *projection;
        let materialization = projection.materialization();
        basis.root_states.push(materialization.root_state());
        for frame in materialization
            .frames()
            .ok_or(ExecutionBasisDenial::Invalid)?
        {
            basis
                .frames
                .push((frame_identity(frame.subject()), basis.frames.len(), frame));
        }
        for placement in materialization.placements() {
            if materialization
                .derived_retirement()
                .is_none_or(|retirement| {
                    retirement
                        .dropped_records()
                        .binary_search(&placement.record())
                        .is_err()
                })
            {
                basis.projected_records.push(placement.record());
            }
            basis
                .placements
                .push((placement.record(), basis.placements.len(), *placement));
        }
        for update in materialization.segment_updates() {
            basis.segment_updates.push((
                (update.page_cell().segment_id().get(), update.page().get()),
                basis.segment_updates.len(),
                *update,
            ));
        }
        for manifest in materialization.manifests() {
            basis
                .manifests
                .push((manifest.coordinate(), basis.manifests.len(), manifest));
        }
    }
    for copy in &pending.source_copies {
        let copy: &'plan worth_store_recovery_physics::PhysicalExtentCopyAdmission = *copy;
        let projection = copy.projection();
        basis.root_states.push(projection.root_state());
        for placement in projection.placements() {
            basis.projected_records.push(placement.record());
            basis
                .placements
                .push((placement.record(), basis.placements.len(), *placement));
        }
    }
    compact_last(&mut basis.frames);
    compact_last(&mut basis.placements);
    compact_last(&mut basis.segment_updates);
    compact_last(&mut basis.manifests);
    basis.projected_records.sort_unstable();
    basis.projected_records.dedup();
    Ok(basis)
}

/// Sorted rows retain the highest encounter ordinal for each key. Swapping
/// into an already-admitted Vec slot avoids a second table or sort scratch.
fn compact_last<Key: Ord, Value>(rows: &mut Vec<(Key, usize, Value)>) {
    rows.sort_unstable_by(|left, right| left.0.cmp(&right.0).then(left.1.cmp(&right.1)));
    let mut retained = 0;
    for read in 0..rows.len() {
        if retained > 0 && rows[retained - 1].0 == rows[read].0 {
            rows.swap(retained - 1, read);
        } else {
            rows.swap(retained, read);
            retained += 1;
        }
    }
    rows.truncate(retained);
}

#[cfg(test)]
mod tests {
    use super::compact_last;
    use worth_store_physical_format::PersistedRecordIdentity;

    #[test]
    fn duplicate_record_and_segment_update_keep_last_encounter() {
        let first = PersistedRecordIdentity::new([7; 16], 1).unwrap();
        let second = PersistedRecordIdentity::new([7; 16], 2).unwrap();
        let mut placements = vec![(second, 0, 10_u64), (first, 1, 20), (second, 2, 30)];
        compact_last(&mut placements);
        assert_eq!(placements, [(first, 1, 20), (second, 2, 30)]);

        let mut updates = vec![((4_u64, 9_u64), 0, 1_u64), ((4, 9), 1, 2), ((1, 3), 2, 3)];
        compact_last(&mut updates);
        assert_eq!(updates, [((1, 3), 2, 3), ((4, 9), 1, 2)]);
    }
}
