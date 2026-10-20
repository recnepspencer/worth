#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalSnapshotAdmissionDenial {
    ForeignRuntime {
        expected_runtime_instance_id: u64,
        actual_runtime_instance_id: u64,
    },
    ActiveSnapshotCapacityExhausted {
        maximum_active_snapshots: usize,
    },
    SnapshotIdentityExhausted,
}

#[derive(Debug)]
enum CurrentBranchSnapshotAdmissionDenial {
    Basis(crate::branch::RelationalBranchBasisDenial),
    Snapshot(RelationalSnapshotAdmissionDenial),
}

impl From<crate::branch::RelationalBranchBasisDenial> for CurrentBranchSnapshotAdmissionDenial {
    fn from(denial: crate::branch::RelationalBranchBasisDenial) -> Self {
        Self::Basis(denial)
    }
}

impl From<RelationalSnapshotAdmissionDenial> for CurrentBranchSnapshotAdmissionDenial {
    fn from(denial: RelationalSnapshotAdmissionDenial) -> Self {
        Self::Snapshot(denial)
    }
}

impl super::authority::VisibilityAuthority<'_> {
    fn open_active_snapshot(
        &self,
        version_id: crate::identity::data::VersionId,
        read_policy: crate::snapshots::data::SnapshotReadPolicy,
    ) -> crate::snapshots::data::SnapshotHandle {
        let branch_id = crate::visibility::branch_scope::authoritative_branch_for_version(
            self.runtime,
            version_id,
        );
        match self.open_active_snapshot_for_branch(version_id, branch_id, read_policy) {
            Ok(Some(handle)) => handle,
            Ok(None) => {
                panic!("authoritative visible version has an owner-retained immutable root")
            }
            Err(CurrentBranchSnapshotAdmissionDenial::Basis(denial)) => {
                panic!("authoritative current basis admission failed: {denial:?}")
            }
            Err(CurrentBranchSnapshotAdmissionDenial::Snapshot(denial)) => {
                panic!("authoritative current snapshot admission failed: {denial:?}")
            }
        }
    }

    fn open_active_snapshot_for_branch(
        &self,
        version_id: crate::identity::data::VersionId,
        branch_id: crate::history::data::BranchId,
        read_policy: crate::snapshots::data::SnapshotReadPolicy,
    ) -> Result<Option<crate::snapshots::data::SnapshotHandle>, CurrentBranchSnapshotAdmissionDenial>
    {
        let basis = super::snapshot_states::VisibilitySnapshotBasis::capture_current(
            self.runtime,
            &branch_id,
            version_id,
        )?;
        basis
            .map(|basis| self.open_active_snapshot_for_basis(basis, read_policy))
            .transpose()
            .map_err(Into::into)
    }

    pub(super) fn open_active_snapshot_for_basis(
        &self,
        basis: super::snapshot_states::VisibilitySnapshotBasis,
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
        super::cache_state::bump_active_snapshot_ref(self.runtime, &basis, 1);
        Ok(handle)
    }

    pub fn snapshot_for_observation(
        &self,
        observation: &crate::mvcc::RelationalBranchObservation,
    ) -> Result<crate::snapshots::data::SnapshotHandle, RelationalSnapshotAdmissionDenial> {
        if observation.identity().runtime_instance_id() != self.runtime.runtime_instance_id() {
            return Err(RelationalSnapshotAdmissionDenial::ForeignRuntime {
                expected_runtime_instance_id: self.runtime.runtime_instance_id(),
                actual_runtime_instance_id: observation.identity().runtime_instance_id(),
            });
        }
        let basis = super::snapshot_states::VisibilitySnapshotBasis::from_observation(observation);
        let handle = self.open_active_snapshot_for_basis(
            basis,
            crate::snapshots::data::SnapshotReadPolicy::ImmutablePinnedNoLazyMutation,
        )?;
        self.runtime
            .history
            .record_snapshot_root_read(observation.identity().branch_id());
        Ok(handle)
    }

    pub(crate) fn snapshot(&self) -> crate::snapshots::data::SnapshotHandle {
        self.open_active_snapshot(
            self.runtime.current_version_id(),
            crate::snapshots::data::SnapshotReadPolicy::ImmutablePinnedNoLazyMutation,
        )
    }
}
