//! Pure in-memory (`MemoryOnly`, zero filesystem) Relational reference runtime
//! mirroring every Native branch fork and `crate::batches::create(round)`
//! publication for the C4b.4 C2.8 World Branch lane.
use super::super::world_branch_child::WorldSnap;
use super::super::StepCtx;
use super::setup::ROOT_NAME;
use std::collections::BTreeMap;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{history::BranchId, runtime::RelationalRuntime};

pub(super) struct MemoryOnlyWorldRef {
    runtime: RelationalRuntime,
    native_branches: BTreeMap<String, BranchId>,
}

impl MemoryOnlyWorldRef {
    pub(super) fn new() -> Self {
        let runtime = crate::workload::runtime_builder(crate::schema_transition::SOURCE)
            .build()
            .expect("valid MemoryOnly Native reference runtime");
        let mut native_branches = BTreeMap::new();
        native_branches.insert(ROOT_NAME.to_owned(), BranchId("main".to_owned()));
        Self {
            runtime,
            native_branches,
        }
    }

    pub(super) fn create_branch(
        &mut self,
        parent: &str,
        name: &str,
        target_native: &str,
        native_fork: bool,
        exec: &ExecutionRequest<'_, '_>,
    ) {
        let parent_id = self.native_branches[parent].clone();
        if native_fork {
            let (_, basis) = self.runtime.observe_fork_source(&parent_id).unwrap();
            let target = BranchId(target_native.to_owned());
            self.runtime
                .fork_branch(exec, target.clone(), basis)
                .unwrap();
            self.native_branches.insert(name.to_owned(), target);
        } else {
            self.native_branches.insert(name.to_owned(), parent_id);
        }
    }

    pub(super) fn retire_branch(&mut self, name: &str) {
        self.native_branches.remove(name);
    }

    pub(super) fn publish_on(&self, branch: &str, round: usize, exec: &ExecutionRequest<'_, '_>) {
        let id = &self.native_branches[branch];
        crate::workload::commit(&self.runtime, exec, id, crate::batches::create(round)).unwrap();
    }

    pub(super) fn payloads_for(
        &self,
        branch: &str,
        exec: &ExecutionRequest<'_, '_>,
    ) -> Vec<String> {
        let id = &self.native_branches[branch];
        let handle = self.runtime.branch_identity(id).unwrap();
        let (_, basis) = self.runtime.observe_branch(&handle).unwrap();
        let snapshot = self
            .runtime
            .snapshots()
            .snapshot_for_observation(&basis.observation())
            .unwrap();
        let view = self
            .runtime
            .read_truth()
            .read_snapshot(&snapshot, exec)
            .unwrap()
            .unwrap();
        let mut payloads: Vec<String> = view
            .entities()
            .iter()
            .map(|e| format!("{:?}|{:?}", e.lifecycle, e.authoritative_aspect_state))
            .collect();
        payloads.sort();
        self.runtime
            .snapshots()
            .release_snapshot(&snapshot)
            .unwrap();
        payloads
    }

    pub(super) fn assert_payload_parity(
        &self,
        snap: &WorldSnap,
        exec: &ExecutionRequest<'_, '_>,
        ctx: &StepCtx,
    ) {
        assert_eq!(
            snap.branches.len(),
            self.native_branches.len(),
            "{ctx}: MemoryOnly branch count mismatch"
        );
        for b in &snap.branches {
            let expected_native = self
                .native_branches
                .get(&b.name)
                .unwrap_or_else(|| panic!("{ctx}: unexpected durable branch {}", b.name));
            assert_eq!(
                b.native_branch, expected_native.0,
                "{ctx}: branch {} native_branch mismatch vs MemoryOnly",
                b.name
            );
            assert_eq!(
                b.entity_payloads,
                self.payloads_for(&b.name, exec),
                "{ctx}: branch {} entity payload mismatch vs MemoryOnly reference",
                b.name
            );
        }
    }
}
