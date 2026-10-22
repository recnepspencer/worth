use super::{
    construction::Shape,
    edit,
    recombine::Engine,
    value_storage::{NativeValues, ValueStorage},
    ReductionDenial, ReductionMetrics, ReductionPlan, ReductionRunFailure, ReductionRunStop,
    ReductionTree,
};
use crate::{CanonicalBits, ExecutionResourceLease};
use worth_foundational::PartitionIdentity;

/// Native storage on the same canonical tree, with no bounded-payload contract.
/// This owner cannot enter the explicitly bounded edit or measurement APIs.
///
/// ```
/// use worth_execution::{CancellationToken,ExecutionRequest,ExecutionWorkCeiling,
///     SerialRequest,SerialMemoryBudget,ReductionPlan,ReductionTree};
/// use worth_foundational::PartitionIdentity;
/// let backing=SerialRequest::from_memory(SerialMemoryBudget::new(None),CancellationToken::new(),None);
/// ExecutionRequest::serial(&backing).run(ExecutionWorkCeiling::new(None),|lease| {
///     let id=PartitionIdentity::new(1);
///     let (mut tree,_)=ReductionTree::try_from_declared_native(
///         ReductionPlan::try_from_sorted_unique(vec![id]).unwrap(),vec![(id,1_u64)],
///         0,|a,b|a+b,lease,||Ok::<_,()>(())).unwrap();
///     tree.update_native(id,2,lease,||Ok::<_,()>(())).unwrap();
///     assert_eq!(*tree.result(),2);
/// }).unwrap();
/// ```
/// ```compile_fail,E0599
/// use worth_execution::{CancellationToken,ExecutionRequest,ExecutionWorkCeiling,
///     SerialRequest,SerialMemoryBudget,ReductionPlan,ReductionTree};
/// use worth_foundational::PartitionIdentity;
/// let backing=SerialRequest::from_memory(SerialMemoryBudget::new(None),CancellationToken::new(),None);
/// ExecutionRequest::serial(&backing).run(ExecutionWorkCeiling::new(None),|lease| {
///     let id=PartitionIdentity::new(1);
///     let (mut tree,_)=ReductionTree::try_from_declared_native(
///         ReductionPlan::try_from_sorted_unique(vec![id]).unwrap(),vec![(id,1_u64)],
///         0,|a,b|a+b,lease,||Ok::<_,()>(())).unwrap();
///     tree.update_checked(id,2,8,||Ok::<_,()>(()));
/// }).unwrap();
/// ```
/// ```compile_fail,E0277
/// use worth_execution::{CancellationToken,ExecutionRequest,ExecutionWorkCeiling,
///     SerialRequest,SerialMemoryBudget,ReductionPlan,ReductionTree,ChargedBytes};
/// use worth_foundational::PartitionIdentity;
/// fn measured(_:impl ChargedBytes) {}
/// let backing=SerialRequest::from_memory(SerialMemoryBudget::new(None),CancellationToken::new(),None);
/// ExecutionRequest::serial(&backing).run(ExecutionWorkCeiling::new(None),|lease| {
///     let id=PartitionIdentity::new(1);
///     let (tree,_)=ReductionTree::try_from_declared_native(
///         ReductionPlan::try_from_sorted_unique(vec![id]).unwrap(),vec![(id,1_u64)],
///         0,|a,b|a+b,lease,||Ok::<_,()>(())).unwrap();
///     measured(tree);
/// }).unwrap();
/// ```
pub struct NativeReductionTree<T, F>(ReductionTree<T, F>);

impl<T: Clone, F: Clone> Clone for NativeReductionTree<T, F> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T, F> NativeReductionTree<T, F> {
    pub fn result(&self) -> &T {
        self.0.result()
    }
    pub fn partition_count(&self) -> usize {
        self.0.partition_count()
    }
    pub fn leaf(&self, partition: PartitionIdentity) -> Option<&T> {
        self.0.leaf(partition)
    }
    pub fn checked_full_build_cost(&self) -> Option<(u64, u64)> {
        self.0.checked_full_build_cost()
    }
    pub fn checked_full_build_cost_after_edits(
        &self,
        removed: &worth_proof::CanonicalUniqueVec<PartitionIdentity>,
        inserted: &worth_proof::CanonicalUniqueVec<PartitionIdentity>,
        policy: crate::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(u64, u64), super::ReductionShapeCostDenial> {
        self.0
            .checked_full_build_cost_after_edits(removed, inserted, policy)
    }
    pub(super) fn from_raw(tree: ReductionTree<T, F>) -> Self {
        Self(tree)
    }
}

impl<T, F> ReductionTree<T, F> {
    pub fn result(&self) -> &T {
        self.root
            .as_ref()
            .map_or(&self.identity, |node| &node.aggregate)
    }
    pub fn partition_count(&self) -> usize {
        self.len
    }
    /// Current canonical leaf value, without recomputing its ancestors.
    pub fn leaf(&self, partition: PartitionIdentity) -> Option<&T> {
        edit::leaf(&self.root, partition)
    }
}

impl<T: Clone + CanonicalBits, F: Fn(&T, &T) -> T> ReductionTree<T, F> {
    pub(crate) fn from_plan_native<E>(
        plan: ReductionPlan,
        values: Vec<T>,
        identity: T,
        combine: F,
        lease: Option<&ExecutionResourceLease<'_>>,
        checkpoint: impl FnMut() -> Result<(), E>,
    ) -> Result<(NativeReductionTree<T, F>, ReductionMetrics), ReductionRunFailure<E>> {
        Self::from_plan_with_storage(
            plan,
            values,
            identity,
            combine,
            NativeValues { lease },
            checkpoint,
        )
        .map(|(tree, metrics)| (NativeReductionTree(tree), metrics))
    }
    pub(super) fn from_plan_with_storage<E, Storage: ValueStorage<T>>(
        plan: ReductionPlan,
        values: Vec<T>,
        identity: T,
        combine: F,
        storage: Storage,
        before_combine: impl FnMut() -> Result<(), E>,
    ) -> Result<(Self, ReductionMetrics), ReductionRunFailure<E>> {
        if plan.identities().len() != values.len() {
            return Err(ReductionRunFailure {
                reason: ReductionRunStop::Denial(ReductionDenial::ValueCountMismatch),
                metrics: ReductionMetrics::for_mode(storage.has_accounting()),
            });
        }
        let mut tree = Self {
            root: None,
            identity,
            combine,
            len: values.len(),
            accounted: storage.has_accounting(),
        };
        let mut engine =
            Engine::with_storage(&tree.identity, &tree.combine, storage, before_combine);
        engine
            .validate(&tree.identity)
            .map_err(|reason| engine.failure(reason))?;
        let shape = Shape::build(plan.identities(), values, &mut engine)
            .map_err(|reason| engine.failure(reason))?;
        let shape_span = engine
            .metrics()
            .accounted_metrics()
            .map(|metrics| metrics.charged_span);
        if let Some(spec) = shape.full_spec() {
            let (candidate, dependency_span) = shape
                .evaluate(spec, &mut engine)
                .map_err(|reason| engine.failure(reason))?;
            tree.root = candidate;
            let span = match (shape_span, dependency_span) {
                (Some(shape), Some(dependency)) => Some(
                    shape
                        .checked_add(dependency)
                        .ok_or_else(|| engine.failure(ReductionRunStop::work_counter_overflow()))?,
                ),
                (None, None) => None,
                _ => unreachable!("one reducer storage mode owns construction"),
            };
            engine.set_full_span(span);
        }
        let metrics = engine.metrics();
        Ok((tree, metrics))
    }

    /// Canonical construction funded at the actual storage owner. No result
    /// size forecast or `ChargedBytes` implementation is required.
    pub fn try_from_declared_native<E>(
        plan: ReductionPlan,
        entries: Vec<(PartitionIdentity, T)>,
        identity: T,
        combine: F,
        lease: Option<&ExecutionResourceLease<'_>>,
        checkpoint: impl FnMut() -> Result<(), E>,
    ) -> Result<(NativeReductionTree<T, F>, ReductionMetrics), ReductionRunFailure<E>> {
        if !plan
            .identities()
            .iter()
            .copied()
            .eq(entries.iter().map(|(id, _)| *id))
        {
            return Err(native_failure(
                lease,
                ReductionRunStop::Denial(ReductionDenial::CoverageMismatch),
            ));
        }
        let storage = NativeValues { lease };
        let (mut values, _values_memory) =
            <NativeValues<'_, '_> as ValueStorage<T>>::buffer::<T, E>(&storage, entries.len())
                .map_err(|reason| native_failure(lease, reason))?;
        values.extend(entries.into_iter().map(|(_, value)| value));
        Self::from_plan_with_storage(plan, values, identity, combine, storage, checkpoint)
            .map(|(tree, metrics)| (NativeReductionTree(tree), metrics))
    }
}

impl<T: Clone + CanonicalBits, F: Fn(&T, &T) -> T> NativeReductionTree<T, F> {
    pub fn update_native<E>(
        &mut self,
        partition: PartitionIdentity,
        value: T,
        lease: Option<&ExecutionResourceLease<'_>>,
        checkpoint: impl FnMut() -> Result<(), E>,
    ) -> Result<ReductionMetrics, ReductionRunFailure<E>> {
        let mut engine = Engine::with_storage(
            &self.0.identity,
            &self.0.combine,
            NativeValues { lease },
            checkpoint,
        );
        let candidate = edit::update(&self.0.root, partition, value, &mut engine)
            .map_err(|reason| engine.failure(reason))?;
        self.0.root = candidate.link;
        self.0.accounted = engine.metrics().has_accounting()
            && self
                .0
                .root
                .as_ref()
                .is_none_or(|root| root.full_subtree_span.is_some());
        Ok(engine.metrics())
    }

    pub fn insert_native<E>(
        &mut self,
        partition: PartitionIdentity,
        value: T,
        lease: Option<&ExecutionResourceLease<'_>>,
        checkpoint: impl FnMut() -> Result<(), E>,
    ) -> Result<ReductionMetrics, ReductionRunFailure<E>> {
        if edit::contains(&self.0.root, partition) {
            return Err(native_failure(
                lease,
                ReductionRunStop::Denial(ReductionDenial::IdentityAlreadyPresent(partition)),
            ));
        }
        let mut engine = Engine::with_storage(
            &self.0.identity,
            &self.0.combine,
            NativeValues { lease },
            checkpoint,
        );
        let candidate = edit::insert(&self.0.root, partition, value, &mut engine)
            .map_err(|reason| engine.failure(reason))?;
        self.0.root = candidate;
        self.0.len += 1;
        self.0.accounted = engine.metrics().has_accounting()
            && self
                .0
                .root
                .as_ref()
                .is_none_or(|root| root.full_subtree_span.is_some());
        Ok(engine.metrics())
    }

    pub fn delete_native<E>(
        &mut self,
        partition: PartitionIdentity,
        lease: Option<&ExecutionResourceLease<'_>>,
        checkpoint: impl FnMut() -> Result<(), E>,
    ) -> Result<ReductionMetrics, ReductionRunFailure<E>> {
        if !edit::contains(&self.0.root, partition) {
            return Err(native_failure(
                lease,
                ReductionRunStop::Denial(ReductionDenial::UnknownIdentity(partition)),
            ));
        }
        let mut engine = Engine::with_storage(
            &self.0.identity,
            &self.0.combine,
            NativeValues { lease },
            checkpoint,
        );
        let candidate = edit::delete(&self.0.root, partition, &mut engine)
            .map_err(|reason| engine.failure(reason))?;
        self.0.root = candidate;
        self.0.len -= 1;
        self.0.accounted = engine.metrics().has_accounting()
            && self
                .0
                .root
                .as_ref()
                .is_none_or(|root| root.full_subtree_span.is_some());
        Ok(engine.metrics())
    }
}

fn native_failure<E>(
    lease: Option<&ExecutionResourceLease<'_>>,
    reason: ReductionRunStop<E>,
) -> ReductionRunFailure<E> {
    ReductionRunFailure {
        reason,
        metrics: ReductionMetrics::for_mode(crate::backend::RunLimits::native_accounting(lease)),
    }
}
