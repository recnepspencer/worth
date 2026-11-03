//! The next canonical tree, funded by its actual native node owner.
use super::super::super::request_execution::QueryRequestExecution;
use super::super::{
    WorthQueryDeterministicReducer, WorthQueryPartitionedComputationDenial as Denial,
};
use super::retained::RetainedPartitions;
use super::tree_attempt::{EditDisposition, TreeAttempt};
pub(super) use super::tree_membership::TreeTarget;
use super::tree_report::{ReportedTree, WorthQueryPartitionedTreeRebuildCause as Cause};
use std::collections::BTreeMap;
use worth_execution::{MapKernelStop, NativeReductionTree, ReductionPlan, ReductionTree};
use worth_foundational::facade::PartitionIdentity;
type Tree<Reduced> = NativeReductionTree<Reduced, fn(&Reduced, &Reduced) -> Reduced>;

/// Only entry/preparation can mint a zero-work beginning.
pub(super) struct TreeBeginning(pub(super) bool);

/// Canonical changed-path edits, or the same full canonical build when the
/// actual request work ceiling cannot cover the full-build contractual work.
/// Every native attempt retains its original stop and accepted-prefix metrics.
pub(super) fn next_tree<Key, Item, Reduced, Stopped>(
    retained: &RetainedPartitions<Key, Item, Reduced>,
    target: impl Into<TreeTarget>,
    reduction_work: Option<u64>,
    mut results: BTreeMap<PartitionIdentity, Reduced>,
    remaining: Option<u64>,
    reducer: &WorthQueryDeterministicReducer<Reduced>,
    execution: &QueryRequestExecution<'_>,
) -> ReportedTree<Tree<Reduced>, Stopped>
where
    Reduced: Clone + worth_execution::CanonicalBits,
{
    let target = target.into();
    let removed = target.removed(retained);
    let beginning = || TreeBeginning(execution.has_accounting());
    let edit = if !execution.has_accounting()
        || reduction_work.is_some_and(|work| remaining.is_none_or(|left| work <= left))
    {
        match execution.with_native_lease(|lease| {
            super::tree_edits::apply(
                TreeAttempt::unstarted(beginning(), Ok(retained.tree.clone())),
                &removed,
                &results,
                execution,
                lease,
            )
        }) {
            Ok(attempt) => attempt,
            Err(cause) => TreeAttempt::unstarted(
                beginning(),
                Err(Denial::from_work_ceiling(cause, Denial::ReducerPanicked)),
            ),
        }
    } else {
        TreeAttempt::unstarted(
            beginning(),
            Ok(EditDisposition::Rebuild(Cause::WorkCeiling)),
        )
    };
    edit.resolve(|| {
        rebuild(
            retained,
            target.into_plan(),
            &mut results,
            remaining,
            reducer,
            execution,
        )
    })
}

fn rebuild<Key, Item, Reduced, Stopped>(
    retained: &RetainedPartitions<Key, Item, Reduced>,
    plan: ReductionPlan,
    results: &mut BTreeMap<PartitionIdentity, Reduced>,
    remaining: Option<u64>,
    reducer: &WorthQueryDeterministicReducer<Reduced>,
    execution: &QueryRequestExecution<'_>,
) -> TreeAttempt<Tree<Reduced>, Denial<Stopped>>
where
    Reduced: Clone + worth_execution::CanonicalBits,
{
    let mut leaves = Vec::new();
    if leaves.try_reserve_exact(plan.identities().len()).is_err() {
        return TreeAttempt::unstarted(TreeBeginning(execution.has_accounting()),Err(Denial::Resource(
            super::super::super::WorthQueryManagedComputationResourceDenial::NativeAllocationRefused {
                kind:worth_execution::ExecutionPhysicalAllocationDenialKind::Allocator,
                requested_payload_bytes:None,
            })));
    }
    for &identity in plan.identities() {
        let value = results
            .remove(&identity)
            .or_else(|| retained.tree.leaf(identity).cloned())
            .expect("each partition has a current result or retained leaf");
        leaves.push((identity, value));
    }
    match execution.with_native_lease(|lease| {
        let mut left = remaining;
        ReductionTree::try_from_declared_native(
            plan,
            leaves,
            (reducer.identity)(),
            reducer.combine,
            Some(lease),
            || {
                execution.checkpoint()?;
                if let Some(left) = &mut left {
                    *left = left.checked_sub(1).ok_or(MapKernelStop::WorkCeiling)?;
                }
                Ok(())
            },
        )
    }) {
        Ok(outcome) => {
            TreeAttempt::native(outcome).map_result(|result| result.map_err(Denial::from_reduction))
        }
        Err(cause) => TreeAttempt::unstarted(
            TreeBeginning(execution.has_accounting()),
            Err(Denial::from_work_ceiling(cause, Denial::ReducerPanicked)),
        ),
    }
}

/// Invalid canonical encodings never establish equality.
pub(super) fn same_bits<Value: worth_execution::CanonicalBits>(
    left: &Value,
    right: &Value,
) -> bool {
    fn bits<Value: worth_execution::CanonicalBits>(value: &Value) -> Option<Vec<u8>> {
        let length = value.canonical_len()?;
        let mut out = Vec::new();
        out.try_reserve_exact(length).ok()?;
        let mut valid = true;
        let complete = value.visit_canonical_bits(&mut |chunk| {
            if chunk.len() > length.saturating_sub(out.len()) {
                valid = false;
                return false;
            }
            out.extend_from_slice(chunk);
            true
        });
        (valid && complete && out.len() == length).then_some(out)
    }
    matches!((bits(left),bits(right)),(Some(left),Some(right)) if left==right)
}
