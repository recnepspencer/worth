use super::*;

impl super::super::authority::VisibilityAuthority<'_> {
    pub(in crate::visibility) fn open_active_snapshot_for_basis(
        &self,
        basis: super::super::snapshot_states::VisibilitySnapshotBasis,
        read_policy: crate::snapshots::data::SnapshotReadPolicy,
    ) -> Result<crate::snapshots::data::SnapshotHandle, RelationalSnapshotAdmissionDenial> {
        let maximum_active_snapshots = self
            .runtime
            .config
            .publication
            .policy
            .max_active_snapshot_handles;
        if let Some(maximum_active_snapshots) = maximum_active_snapshots
            .filter(|maximum| self.runtime.visibility.active_snapshot_count() >= *maximum)
        {
            return Err(
                RelationalSnapshotAdmissionDenial::ActiveSnapshotCapacityExhausted {
                    maximum_active_snapshots,
                },
            );
        }
        let snapshot_id = self
            .runtime
            .visibility
            .allocate_snapshot_id()
            .ok_or(RelationalSnapshotAdmissionDenial::SnapshotIdentityExhausted)?;
        let handle = crate::snapshots::data::SnapshotHandle {
            runtime_instance_id: self.runtime.runtime_instance_id(),
            branch_id: basis.branch_id().clone(),
            snapshot_id,
            version_id: basis.version_id(),
            read_policy,
        };
        // The binding carries the exact immutable owner root and its retained
        // observation. That O(1) obligation is the snapshot's storage fence;
        // materialized visibility remains requested read work.
        self.runtime.visibility.insert_active_handle(
            handle.snapshot_id,
            crate::runtime::SnapshotHandleBinding::new(basis.clone(), handle.read_policy),
        );
        super::super::cache_state::bump_active_snapshot_ref(self.runtime, &basis, 1);
        Ok(handle)
    }

    pub fn snapshot_for_observation(
        &self,
        observation: &crate::mvcc::observation::RelationalBranchObservation,
    ) -> Result<crate::snapshots::data::SnapshotHandle, RelationalSnapshotAdmissionDenial> {
        if observation.identity().runtime_instance_id() != self.runtime.runtime_instance_id() {
            return Err(RelationalSnapshotAdmissionDenial::ForeignRuntime {
                expected_runtime_instance_id: self.runtime.runtime_instance_id(),
                actual_runtime_instance_id: observation.identity().runtime_instance_id(),
            });
        }
        let basis =
            super::super::snapshot_states::VisibilitySnapshotBasis::from_observation(observation);
        let handle = self.open_active_snapshot_for_basis(
            basis,
            crate::snapshots::data::SnapshotReadPolicy::ImmutablePinnedNoLazyMutation,
        )?;
        self.runtime
            .history
            .record_snapshot_root_read(observation.identity().branch_id());
        Ok(handle)
    }

    pub(crate) fn snapshot(
        &self,
    ) -> Result<crate::snapshots::data::SnapshotHandle, CurrentBranchSnapshotAdmissionDenial> {
        use CurrentBranchSnapshotAdmissionDenial as Denial;
        let version = self.runtime.current_version_id();
        let branch = super::super::branch_scope::branch_for_version(self.runtime, version)
            .ok_or(Denial::CurrentVersionUnavailable)?;
        let basis = self
            .runtime
            .capture_current_visibility_basis(&branch, version)
            .map_err(Denial::Basis)?
            .ok_or(Denial::CurrentVersionUnavailable)?;
        self.open_active_snapshot_for_basis(
            basis,
            crate::snapshots::data::SnapshotReadPolicy::ImmutablePinnedNoLazyMutation,
        )
        .map_err(Denial::Snapshot)
    }
}
