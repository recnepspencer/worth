//! Receiver-owned evidence about every closure-scoped installed-owner acquisition.
use super::WorthQueryRelationalSourceOwner;
#[derive(Clone)]
pub(in crate::domain_computation) struct ReadLockProbeForTest {
    owner: WorthQueryRelationalSourceOwner,
}
impl WorthQueryRelationalSourceOwner {
    pub(in crate::domain_computation) fn read_lock_probe_for_test(&self) -> ReadLockProbeForTest {
        ReadLockProbeForTest {
            owner: self.clone(),
        }
    }
}
impl ReadLockProbeForTest {
    pub(in crate::domain_computation) fn is_locked_for_test(&self) -> bool {
        self.owner.lock_is_held_for_test()
    }
    pub(in crate::domain_computation) fn acquisitions_for_test(&self) -> u64 {
        self.owner.lock_acquisitions_for_test()
    }
    pub(in crate::domain_computation) fn with_owner_lock_for_test<T>(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        read: impl FnOnce() -> T,
    ) -> T {
        self.owner.with_runtime(request, |_| read())
    }
}

impl ReadLockProbeForTest {
    pub(in crate::domain_computation) fn retain_snapshot_for_test(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> worth_relational::facade::snapshots::SnapshotHandle {
        self.owner.with_runtime(request, |runtime| {
            let (_, basis) = runtime
                .observe_branch(&runtime.main_branch_identity())
                .unwrap();
            runtime
                .snapshots()
                .snapshot_for_observation(&basis.observation())
                .unwrap()
        })
    }
    pub(in crate::domain_computation) fn release_snapshot_for_test(
        &self,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) {
        self.owner.release_query_snapshot(snapshot);
    }
    pub(in crate::domain_computation) fn count_bridge_lock_for_test(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) {
        let basis = self.owner.with_runtime(request, |runtime| {
            runtime
                .observe_branch(&runtime.main_branch_identity())
                .unwrap()
                .1
        });
        // Deliberately use a cloned Bridge source, rather than a Query door.
        let source = self.owner.bridge_source();
        let _head = source.bind_branch_head_basis_for_bridge(&basis).unwrap();
    }
}
