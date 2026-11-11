use std::{
    panic::{catch_unwind, AssertUnwindSafe},
    sync::Arc,
};

use worth_foundational::PartitionIdentity;

use crate::{oracle::CanonicalBits, report::ChargedBytes};

use super::{
    value_storage::{BoundedValues, ValueStorage},
    Admitted, Link, Node, ReductionMetrics, ReductionRunFailure, ReductionRunStop,
};

/// One node policy; both reduction values and native settled slots use the
/// same Cartesian insertion, path copying, deletion and merge machinery.
pub(super) trait NodeEngine<T, E> {
    fn inspect(&mut self) {}
    fn admit(&self, value: T) -> Result<Admitted<T>, ReductionRunStop<E>>;
    fn same(&self, left: &Admitted<T>, right: &Admitted<T>) -> Result<bool, ReductionRunStop<E>>;
    fn build(
        &mut self,
        identity: PartitionIdentity,
        value: T,
        left: Link<T>,
        right: Link<T>,
    ) -> Result<Arc<Node<T>>, ReductionRunStop<E>> {
        let value = self.admit(value)?;
        self.build_proven(identity, value, left, right)
    }
    fn build_proven(
        &mut self,
        identity: PartitionIdentity,
        value: Admitted<T>,
        left: Link<T>,
        right: Link<T>,
    ) -> Result<Arc<Node<T>>, ReductionRunStop<E>>;
    fn carry(
        &mut self,
        old: &Arc<Node<T>>,
        value: Admitted<T>,
        left: Link<T>,
        right: Link<T>,
    ) -> Result<Arc<Node<T>>, ReductionRunStop<E>>;
}

impl<T, F, H, E, Storage> NodeEngine<T, E> for Engine<'_, T, F, H, Storage>
where
    T: Clone + CanonicalBits,
    Storage: ValueStorage<T>,
    F: Fn(&T, &T) -> T,
    H: FnMut() -> Result<(), E>,
{
    fn admit(&self, value: T) -> Result<Admitted<T>, ReductionRunStop<E>> {
        self.admit(value)
    }
    fn same(&self, left: &Admitted<T>, right: &Admitted<T>) -> Result<bool, ReductionRunStop<E>> {
        self.same(left, right)
    }
    fn build_proven(
        &mut self,
        id: PartitionIdentity,
        value: Admitted<T>,
        left: Link<T>,
        right: Link<T>,
    ) -> Result<Arc<Node<T>>, ReductionRunStop<E>> {
        self.build_proven(id, value, left, right)
    }
    fn carry(
        &mut self,
        old: &Arc<Node<T>>,
        value: Admitted<T>,
        left: Link<T>,
        right: Link<T>,
    ) -> Result<Arc<Node<T>>, ReductionRunStop<E>> {
        Engine::carry(self, old, value, left, right)
    }
}

pub(super) struct Engine<'a, T, F, H, Storage = BoundedValues> {
    identity: &'a T,
    combine: &'a F,
    storage: Storage,
    before_combine: H,
    metrics: ReductionMetrics,
}

impl<'a, T, F, H, E> Engine<'a, T, F, H>
where
    T: Clone + ChargedBytes + CanonicalBits,
    F: Fn(&T, &T) -> T,
    H: FnMut() -> Result<(), E>,
{
    pub(super) fn new(
        identity: &'a T,
        combine: &'a F,
        max_value_bytes: u64,
        before_combine: H,
    ) -> Self {
        Self::with_storage(
            identity,
            combine,
            BoundedValues(max_value_bytes),
            before_combine,
        )
    }
}

impl<'a, T, F, H, E, Storage> Engine<'a, T, F, H, Storage>
where
    T: Clone + CanonicalBits,
    F: Fn(&T, &T) -> T,
    H: FnMut() -> Result<(), E>,
    Storage: ValueStorage<T>,
{
    pub(super) fn with_storage(
        identity: &'a T,
        combine: &'a F,
        storage: Storage,
        before_combine: H,
    ) -> Self {
        Self {
            identity,
            combine,
            storage,
            before_combine,
            metrics: ReductionMetrics::for_mode(storage.has_accounting()),
        }
    }

    pub(super) fn metrics(&self) -> ReductionMetrics {
        self.metrics
    }

    pub(super) fn buffer<U>(
        &self,
        length: usize,
    ) -> Result<(Vec<U>, Option<crate::ExecutionMemoryReservation>), ReductionRunStop<E>> {
        self.storage.buffer(length)
    }

    pub(super) fn visit(&mut self) -> Result<(), ReductionRunStop<E>> {
        (self.before_combine)().map_err(ReductionRunStop::Hook)?;
        if self.metrics.has_accounting() {
            self.metrics
                .record_visit()
                .map_err(|_| ReductionRunStop::work_counter_overflow())?;
        }
        Ok(())
    }

    /// A full tree has independent child subtrees even when this backend
    /// evaluates them serially. Its charged span is the canonical DAG depth.
    pub(super) fn set_full_span(&mut self, span: Option<u64>) {
        if let Some(accounting) = self.metrics.accounted_mut() {
            accounting.charged_span = span.expect("accounted reducer carries its dependency span");
        }
    }

    pub(super) fn failure(&self, reason: ReductionRunStop<E>) -> ReductionRunFailure<E> {
        ReductionRunFailure {
            reason,
            metrics: self.metrics,
        }
    }

    pub(super) fn validate(&self, value: &T) -> Result<(), ReductionRunStop<E>> {
        self.storage.validate(value)
    }

    pub(super) fn admit(&self, value: T) -> Result<Admitted<T>, ReductionRunStop<E>> {
        let canonical = Arc::new(self.storage.canonical(&value)?);
        Ok(Admitted {
            value,
            canonical: Some(canonical),
        })
    }
    pub(super) fn same(
        &self,
        left: &Admitted<T>,
        right: &Admitted<T>,
    ) -> Result<bool, ReductionRunStop<E>> {
        let a = self.proof(left, false)?;
        let b = self.proof(right, false)?;
        Ok(a == b)
    }
    fn proof<'v>(
        &self,
        value: &'v Admitted<T>,
        payload: bool,
    ) -> Result<&'v super::value_storage::CanonicalBytes, ReductionRunStop<E>> {
        let proof = value.canonical.as_deref().ok_or(ReductionRunStop::Denial(
            super::ReductionDenial::InvalidCanonicalEncoding,
        ))?;
        self.storage.check_proof(&value.value, proof, payload)?;
        Ok(proof)
    }

    fn call(&mut self, left: &T, right: &T) -> Result<Admitted<T>, ReductionRunStop<E>> {
        (self.before_combine)().map_err(ReductionRunStop::Hook)?;
        if self.metrics.has_accounting() {
            self.metrics
                .record_combine()
                .map_err(|_| ReductionRunStop::work_counter_overflow())?;
        }
        let value = catch_unwind(AssertUnwindSafe(|| (self.combine)(left, right)))
            .map_err(|_| ReductionRunStop::Panic)?;
        self.admit(value)
    }

    pub(super) fn build(
        &mut self,
        identity: PartitionIdentity,
        value: T,
        left: Link<T>,
        right: Link<T>,
    ) -> Result<Arc<Node<T>>, ReductionRunStop<E>> {
        let value = self.admit(value)?;
        self.build_proven(identity, value, left, right)
    }
    pub(super) fn build_proven(
        &mut self,
        identity: PartitionIdentity,
        value: Admitted<T>,
        left: Link<T>,
        right: Link<T>,
    ) -> Result<Arc<Node<T>>, ReductionRunStop<E>> {
        self.proof(&value, true)?;
        let left_aggregate = left
            .as_ref()
            .map_or(self.identity, |node| &node.aggregate.value);
        let right_aggregate = right
            .as_ref()
            .map_or(self.identity, |node| &node.aggregate.value);
        let middle = self.call(left_aggregate, &value)?;
        let aggregate = self.call(&middle, right_aggregate)?;
        if self.metrics.has_accounting() {
            self.metrics
                .record_node()
                .map_err(|_| ReductionRunStop::work_counter_overflow())?;
        }
        make_node(&self.storage, identity, value, aggregate, left, right)
    }

    pub(super) fn carry(
        &self,
        old: &Arc<Node<T>>,
        value: Admitted<T>,
        left: Link<T>,
        right: Link<T>,
    ) -> Result<Arc<Node<T>>, ReductionRunStop<E>> {
        make_node(
            &self.storage,
            old.identity,
            value,
            old.aggregate.clone(),
            left,
            right,
        )
    }
}

fn make_node<T, E>(
    storage: &impl ValueStorage<T>,
    identity: PartitionIdentity,
    value: Admitted<T>,
    aggregate: Admitted<T>,
    left: Link<T>,
    right: Link<T>,
) -> Result<Arc<Node<T>>, ReductionRunStop<E>> {
    let (retained_bytes, memory) = storage.node(&value.value, &aggregate.value, &left, &right)?;
    let retained_bytes = retained_bytes
        .map(|bytes| {
            let proof_bytes =
                |value: &Admitted<T>| value.canonical.as_ref().map_or(0, |p| p.retained_bytes());
            bytes
                .checked_add(proof_bytes(&value))
                .and_then(|n| n.checked_add(proof_bytes(&aggregate)))
                .ok_or(ReductionRunStop::ResultCapacityExceeded)
        })
        .transpose()?;
    let full_subtree_span = if storage.has_accounting() {
        let span = |link: &Link<T>| match link {
            None => Ok(0),
            Some(node) => node.full_subtree_span.ok_or(ReductionRunStop::Denial(
                super::ReductionDenial::AccountingUnavailable,
            )),
        };
        Some(
            span(&left)?
                .max(span(&right)?)
                .checked_add(4)
                .ok_or_else(ReductionRunStop::work_counter_overflow)?,
        )
    } else {
        None
    };
    Ok(Arc::new(Node {
        identity,
        value,
        aggregate,
        left,
        right,
        retained_bytes,
        full_subtree_span,
        _memory: memory,
    }))
}
