//! Register the observation's exact retained root with its owning directory.
use super::{bump_active_snapshot_ref, SnapshotObservationInputs};
use crate::history::retention::VisibilitySnapshotBasis;
use crate::visibility::snapshot_admission::RelationalSnapshotAdmissionDenial;

impl SnapshotObservationInputs<'_> {
    pub fn open_active_snapshot_for_basis(
        &self,
        basis: VisibilitySnapshotBasis,
        read_policy: crate::snapshots::data::SnapshotReadPolicy,
    ) -> Result<crate::snapshots::data::SnapshotHandle, RelationalSnapshotAdmissionDenial> {
        let maximum_active_snapshots = self.maximum_active_snapshots;
        if let Some(maximum_active_snapshots) = maximum_active_snapshots
            .filter(|maximum| self.visibility.active_snapshot_count() >= *maximum)
        {
            return Err(
                RelationalSnapshotAdmissionDenial::ActiveSnapshotCapacityExhausted {
                    maximum_active_snapshots,
                },
            );
        }
        let snapshot_id = self
            .visibility
            .allocate_snapshot_id()
            .ok_or(RelationalSnapshotAdmissionDenial::SnapshotIdentityExhausted)?;
        let handle = crate::snapshots::data::SnapshotHandle {
            runtime_instance_id: self.runtime_identity,
            branch_id: basis.branch_id().clone(),
            snapshot_id,
            version_id: basis.version_id(),
            read_policy,
        };
        // The binding carries the exact immutable owner root and its retained
        // observation. That O(1) obligation is the snapshot's storage fence;
        // materialized visibility remains requested read work.
        self.visibility.insert_active_handle(
            handle.snapshot_id,
            crate::runtime::state::subsystems::visibility::SnapshotHandleBinding::new(
                basis.clone(),
                handle.read_policy,
            ),
        );
        bump_active_snapshot_ref(self.cache, basis.key(), 1);
        Ok(handle)
    }

    pub fn snapshot_for_observation(
        &self,
        observation: &crate::mvcc::observation::RelationalBranchObservation,
    ) -> Result<crate::snapshots::data::SnapshotHandle, RelationalSnapshotAdmissionDenial> {
        if observation.identity().runtime_instance_id() != self.runtime_identity {
            return Err(RelationalSnapshotAdmissionDenial::ForeignRuntime {
                expected_runtime_instance_id: self.runtime_identity,
                actual_runtime_instance_id: observation.identity().runtime_instance_id(),
            });
        }
        let basis = VisibilitySnapshotBasis::from_observation(observation);
        let handle = self.open_active_snapshot_for_basis(
            basis,
            crate::snapshots::data::SnapshotReadPolicy::ImmutablePinnedNoLazyMutation,
        )?;
        self.history
            .record_snapshot_root_read(observation.identity().branch_id());
        Ok(handle)
    }
}
