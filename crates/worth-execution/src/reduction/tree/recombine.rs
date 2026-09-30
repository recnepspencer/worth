use std::{
    mem::size_of,
    panic::{catch_unwind, AssertUnwindSafe},
    sync::Arc,
};

use worth_foundational::PartitionIdentity;

use crate::{oracle::CanonicalBits, report::ChargedBytes};

use super::{node_bytes, Link, Node, ReductionMetrics, ReductionRunFailure, ReductionRunStop};

pub(super) struct Engine<'a, T, F, H> {
    identity: &'a T,
    combine: &'a F,
    max_value_bytes: u64,
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
        Self {
            identity,
            combine,
            max_value_bytes,
            before_combine,
            metrics: ReductionMetrics::default(),
        }
    }

    pub(super) fn metrics(&self) -> ReductionMetrics {
        self.metrics
    }

    pub(super) fn visit(&mut self) -> Result<(), ReductionRunStop<E>> {
        (self.before_combine)().map_err(ReductionRunStop::Hook)?;
        self.metrics
            .record_visit()
            .map_err(|_| ReductionRunStop::WorkCounterOverflow)
    }

    /// A full tree has independent child subtrees even when this backend
    /// evaluates them serially. Its charged span is the canonical DAG depth.
    pub(super) fn set_full_span(&mut self, span: u64) {
        self.metrics.charged_span = span;
    }

    pub(super) fn failure(&self, reason: ReductionRunStop<E>) -> ReductionRunFailure<E> {
        ReductionRunFailure {
            reason,
            metrics: self.metrics,
        }
    }

    pub(super) fn validate(&self, value: &T) -> Result<(), ReductionRunStop<E>> {
        let inline =
            u64::try_from(size_of::<T>()).map_err(|_| ReductionRunStop::ResultCapacityExceeded)?;
        let bytes = inline
            .checked_add(value.additional_charged_bytes())
            .ok_or(ReductionRunStop::ResultCapacityExceeded)?;
        if bytes > self.max_value_bytes {
            return Err(ReductionRunStop::ResultCapacityExceeded);
        }
        canonical_bytes(value, self.max_value_bytes).map(|_| ())
    }

    pub(super) fn same(&self, left: &T, right: &T) -> Result<bool, ReductionRunStop<E>> {
        Ok(canonical_bytes(left, self.max_value_bytes)?
            == canonical_bytes(right, self.max_value_bytes)?)
    }

    fn call(&mut self, left: &T, right: &T) -> Result<T, ReductionRunStop<E>> {
        (self.before_combine)().map_err(ReductionRunStop::Hook)?;
        self.metrics
            .record_combine()
            .map_err(|_| ReductionRunStop::WorkCounterOverflow)?;
        let value = catch_unwind(AssertUnwindSafe(|| (self.combine)(left, right)))
            .map_err(|_| ReductionRunStop::Panic)?;
        self.validate(&value)?;
        Ok(value)
    }

    pub(super) fn build(
        &mut self,
        identity: PartitionIdentity,
        value: T,
        left: Link<T>,
        right: Link<T>,
    ) -> Result<Arc<Node<T>>, ReductionRunStop<E>> {
        self.validate(&value)?;
        let left_aggregate = left.as_ref().map_or(self.identity, |node| &node.aggregate);
        let right_aggregate = right.as_ref().map_or(self.identity, |node| &node.aggregate);
        let middle = self.call(left_aggregate, &value)?;
        let aggregate = self.call(&middle, right_aggregate)?;
        self.metrics
            .record_node()
            .map_err(|_| ReductionRunStop::WorkCounterOverflow)?;
        make_node(identity, value, aggregate, left, right)
    }

    pub(super) fn carry(
        &self,
        old: &Arc<Node<T>>,
        value: T,
        left: Link<T>,
        right: Link<T>,
    ) -> Result<Arc<Node<T>>, ReductionRunStop<E>> {
        make_node(old.identity, value, old.aggregate.clone(), left, right)
    }
}

fn make_node<T: ChargedBytes, E>(
    identity: PartitionIdentity,
    value: T,
    aggregate: T,
    left: Link<T>,
    right: Link<T>,
) -> Result<Arc<Node<T>>, ReductionRunStop<E>> {
    let retained_bytes = node_bytes(&value, &aggregate, &left, &right)
        .ok_or(ReductionRunStop::ResultCapacityExceeded)?;
    Ok(Arc::new(Node {
        identity,
        value,
        aggregate,
        left,
        right,
        retained_bytes,
    }))
}

fn canonical_bytes<T: CanonicalBits, E>(
    value: &T,
    max_bytes: u64,
) -> Result<Vec<u8>, ReductionRunStop<E>> {
    let length = value.canonical_len().ok_or(ReductionRunStop::Denial(
        super::super::ReductionDenial::InvalidCanonicalEncoding,
    ))?;
    if u64::try_from(length)
        .ok()
        .is_none_or(|bytes| bytes > max_bytes)
    {
        return Err(ReductionRunStop::ResultCapacityExceeded);
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(length)
        .map_err(|_| ReductionRunStop::ResultCapacityExceeded)?;
    let mut valid = true;
    let complete = value.visit_canonical_bits(&mut |chunk| {
        if chunk.len() > length.saturating_sub(bytes.len()) {
            valid = false;
            return false;
        }
        bytes.extend_from_slice(chunk);
        true
    });
    if valid && complete && bytes.len() == length {
        Ok(bytes)
    } else {
        Err(ReductionRunStop::Denial(
            super::super::ReductionDenial::InvalidCanonicalEncoding,
        ))
    }
}
