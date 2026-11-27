//! Corruption targets the actual pinned storage image selected for an SDK read.
use super::RelationalRuntime;
use crate::{identity::data::EntityId, snapshots::data::SnapshotHandle};
use worth_foundational::facade::{AspectFieldLocator, AspectValue};

impl RelationalRuntime {
    #[doc(hidden)]
    pub fn corrupt_snapshot_field_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
        entity: EntityId,
        locator: &AspectFieldLocator,
        replacement: Option<AspectValue>,
    ) -> bool {
        if snapshot.runtime_instance_id() != self.runtime_instance_id() {
            return false;
        }
        let Some(binding) = self
            .visibility
            .handles
            .active_binding(snapshot.snapshot_id())
        else {
            return false;
        };
        let root = self.services.symbols.with_read(|symbols| {
            crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(
                |lane| {
                    binding.basis.root().corrupt_field_image_for_test(
                        lane,
                        execution,
                        entity,
                        locator,
                        replacement,
                        symbols,
                    )
                },
            )
        });
        let Some(root) = root else { return false };
        let basis = crate::history::retention::VisibilitySnapshotBasis::from_applied_root(
            binding.branch_id,
            binding.version_id,
            root,
        );
        // Retain the issued snapshot identity and its original closed read policy.
        self.visibility
            .handles
            .remove_active(snapshot.snapshot_id());
        self.visibility.handles.insert_active(
            snapshot.snapshot_id(),
            crate::runtime::state::subsystems::SnapshotHandleBinding::new(
                basis,
                binding.read_policy,
            ),
        );
        self.visibility.cache.clear();
        true
    }

    #[cfg(test)]
    pub(crate) fn rebuild_lineage_derived_indexes_for_test(&self) -> crate::runtime::LineageState {
        crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(|lane| {
            let mut snapshot = self.lineage.snapshot_for_reconstruction(lane);
            assert!(
                snapshot.rebuild_derived_indexes_if_full(lane),
                "full-history test runtime holds recovery token"
            );
            snapshot
        })
    }
}

#[cfg(test)]
impl crate::runtime::RelationalRuntime {
    pub(crate) fn set_retention_capacity_for_test(
        &mut self,
        max_live_root_obligations: usize,
        max_retired_branch_roots: usize,
    ) {
        self.unshared_state_mut()
            .expect("retention capacity test support requires the only handle to this runtime")
            .history
            .reset_retention_owner_with_test_limits_for_test(
                max_live_root_obligations,
                max_retired_branch_roots,
            );
    }
}

#[cfg(test)]
impl crate::runtime::RelationalRuntime {
    pub(crate) fn fill_retired_branch_name_capacity_for_test(&self) {
        self.history.exhaust_retired_branch_name_roster_for_test();
    }
}
