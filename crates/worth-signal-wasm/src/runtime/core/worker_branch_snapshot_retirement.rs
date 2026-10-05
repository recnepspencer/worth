use std::collections::BTreeMap;

use worth_signal::facade::branch::AdmittedSignalBranchSnapshot;

use super::RuntimeCore;

pub(super) struct WorkerBranchSnapshotRetirement;

impl WorkerBranchSnapshotRetirement {
    pub(super) fn admitted_for(
        snapshots: &BTreeMap<(u64, u64), AdmittedSignalBranchSnapshot>,
        branch_id: u64,
    ) -> Vec<&AdmittedSignalBranchSnapshot> {
        snapshots
            .range((branch_id, 0)..=(branch_id, u64::MAX))
            .map(|(_, snapshot)| snapshot)
            .collect()
    }

    pub(super) fn release(runtime: &mut RuntimeCore, branch_ids: &[u64]) {
        let keys = branch_ids
            .iter()
            .flat_map(|branch_id| {
                runtime
                    .admitted_runtime_snapshots
                    .range((*branch_id, 0)..=(*branch_id, u64::MAX))
                    .map(|(key, _)| *key)
            })
            .collect::<Vec<_>>();
        for key in keys {
            if let Some(snapshot) = runtime.admitted_runtime_snapshots.remove(&key) {
                drop(snapshot.into_snapshot());
            }
        }
    }
}
