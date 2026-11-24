//! Unique private count storage; removal retains actual nodes outside gates.
use super::*;
pub(super) struct RemovedObligation {
    pub(super) _record: ExecutionOrderedMapCell<u64, RelationalRetentionObligationRecord>,
    pub(super) _counts: [Option<ExecutionOrderedMapCell<u64, usize>>; 2],
}
pub(super) fn release_root(
    counts: &mut ExecutionOrderedMap<u64, usize>,
    root: u64,
) -> Option<ExecutionOrderedMapCell<u64, usize>> {
    let prior = *counts
        .get(&root)
        .expect("every released occurrence owns a registered count");
    let next = prior
        .checked_sub(1)
        .expect("root multiplicity remains balanced");
    if next == 0 {
        counts
            .remove_retaining_unique(&root)
            .expect("private count map is never cloned")
    } else {
        counts
            .set_existing_unique(&root, next)
            .expect("private count map is never cloned");
        None
    }
}
impl RelationalBranchRetentionLaneState {
    pub(super) fn release_roots(
        &mut self,
        roots: RootIds,
    ) -> [Option<ExecutionOrderedMapCell<u64, usize>>; 2] {
        let mut removed = [None, None];
        for (index, root) in roots.iter().enumerate() {
            removed[index] = release_root(&mut self.root_obligations, *root);
        }
        removed
    }
    pub(super) fn release_obligation(
        &mut self,
        id: u64,
        roots: &RetainedRoots,
        owner: &RelationalBranchRetentionOwnerInner,
    ) -> Option<RemovedObligation> {
        let record = self
            .obligations
            .remove_owned_retaining_unique(&id)
            .expect("private obligation map is never cloned")?;
        let value = record.value();
        let ids = value.root_ids();
        let counts = self.release_roots(ids);
        debug_assert!(
            ids.len() == roots.len()
                && ids
                    .iter()
                    .zip(roots.iter())
                    .all(|(id, root)| *id == root.id()),
            "removed record and guard retain identical original root occurrences"
        );
        if value.active_operation {
            self.active_operations = self
                .active_operations
                .checked_sub(1)
                .expect("active operation accounting remains balanced");
        }
        record_release(&mut self.counters, value.kind());
        owner.ordinary_counters.record_release(value.kind());
        release_live_obligation_capacity(owner);
        Some(RemovedObligation {
            _record: record,
            _counts: counts,
        })
    }
}
