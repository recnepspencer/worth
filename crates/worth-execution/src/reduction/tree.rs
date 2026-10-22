use std::sync::Arc;

use worth_foundational::PartitionIdentity;

use crate::{oracle::CanonicalBits, report::ChargedBytes};

use super::plan::{ReductionDenial, ReductionPlan};

mod construction;
mod denial;
use denial::{failure, pure_denial};
mod edit;
mod full_cost;
mod shape_cost;
pub use shape_cost::ReductionShapeCostDenial;
mod memory;
mod native;
pub use native::NativeReductionTree;
mod recombine;
mod retained_run;
mod scheduled;
mod staged;
mod structural_span;
mod value_storage;
pub(crate) use retained_run::{RunRange, RunTree};

use edit::{contains, delete, insert, update};
pub use memory::{ReductionEditMemoryBoundError, ReductionInsertMemoryBoundError};
use recombine::Engine;
pub(crate) use scheduled::ScheduledReductionError;

pub(super) type Link<T> = Option<Arc<Node<T>>>;

pub(super) struct Node<T> {
    pub(super) identity: PartitionIdentity,
    pub(super) value: T,
    pub(super) aggregate: T,
    pub(super) left: Link<T>,
    pub(super) right: Link<T>,
    pub(super) retained_bytes: Option<u64>,
    pub(super) full_subtree_span: Option<u64>,
    // Native retained slots fund each copied node independently. Ordinary
    // reductions retain their existing enclosing tree reservation.
    pub(super) _memory: Option<crate::ExecutionMemoryReservation>,
}

mod metrics;
pub use metrics::{ReductionAccounting, ReductionMetrics};

#[derive(Debug, PartialEq, Eq)]
pub enum ReductionRunStop<E> {
    Hook(E),
    Panic,
    Denial(ReductionDenial),
    ResultCapacityExceeded,
    WorkCounterOverflow,
}

impl<E> ReductionRunStop<E> {
    pub(crate) fn work_counter_overflow() -> Self {
        crate::backend::request_stop(Self::WorkCounterOverflow)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ReductionRunFailure<E> {
    pub reason: ReductionRunStop<E>,
    pub metrics: ReductionMetrics,
}

impl<E: ChargedBytes> ChargedBytes for ReductionRunFailure<E> {
    fn additional_charged_bytes(&self) -> u64 {
        match &self.reason {
            ReductionRunStop::Hook(value) => value.additional_charged_bytes(),
            _ => 0,
        }
    }
}

/// Retained Cartesian reduction tree. Edits stage immutable path copies and
/// publish one root only after every combine and validation succeeds.
///
/// Cutoff is decided by canonical bits. The tree's value bound does not
/// permit substituting `PartialEq`: comparing its values is error E0369.
///
/// ```compile_fail,E0369
/// use worth_execution::{CanonicalBits, ChargedBytes};
/// fn cutoff<T: Clone + ChargedBytes + CanonicalBits>(left: &T, right: &T) -> bool {
///     left == right
/// }
/// ```
pub struct ReductionTree<T, F> {
    root: Link<T>,
    identity: T,
    combine: F,
    len: usize,
    accounted: bool,
}

impl<T: Clone, F: Clone> Clone for ReductionTree<T, F> {
    fn clone(&self) -> Self {
        Self {
            root: self.root.clone(),
            identity: self.identity.clone(),
            combine: self.combine.clone(),
            len: self.len,
            accounted: self.accounted,
        }
    }
}

impl<T: Clone + ChargedBytes + CanonicalBits, F: Fn(&T, &T) -> T> ReductionTree<T, F> {
    pub(crate) fn from_plan_checked<E>(
        plan: ReductionPlan,
        values: Vec<T>,
        identity: T,
        combine: F,
        max_value_bytes: u64,
        before_combine: impl FnMut() -> Result<(), E>,
    ) -> Result<(Self, ReductionMetrics), ReductionRunFailure<E>> {
        Self::from_plan_with_storage(
            plan,
            values,
            identity,
            combine,
            value_storage::BoundedValues(max_value_bytes),
            before_combine,
        )
    }

    /// Structural/unmetered construction. Use `try_from_declared_checked` for
    /// a budgeted stage.
    pub fn try_from_declared(
        plan: ReductionPlan,
        entries: Vec<(PartitionIdentity, T)>,
        identity: T,
        combine: F,
    ) -> Result<(Self, ReductionMetrics), ReductionDenial> {
        Self::try_from_declared_checked(plan, entries, identity, combine, u64::MAX, || {
            Ok::<_, ()>(())
        })
        .map_err(pure_denial)
    }

    pub fn try_from_declared_checked<E>(
        plan: ReductionPlan,
        entries: Vec<(PartitionIdentity, T)>,
        identity: T,
        combine: F,
        max_value_bytes: u64,
        before_combine: impl FnMut() -> Result<(), E>,
    ) -> Result<(Self, ReductionMetrics), ReductionRunFailure<E>> {
        if !plan
            .identities()
            .iter()
            .copied()
            .eq(entries.iter().map(|(partition, _)| *partition))
        {
            return Err(failure(ReductionRunStop::Denial(
                ReductionDenial::CoverageMismatch,
            )));
        }
        Self::from_plan_checked(
            plan,
            entries.into_iter().map(|(_, value)| value).collect(),
            identity,
            combine,
            max_value_bytes,
            before_combine,
        )
    }

    /// The value ceiling includes inline storage, owned allocations, and
    /// canonical encoding length. The caller reserves that ceiling.
    pub fn update_checked<E>(
        &mut self,
        partition: PartitionIdentity,
        value: T,
        max_value_bytes: u64,
        before_combine: impl FnMut() -> Result<(), E>,
    ) -> Result<ReductionMetrics, ReductionRunFailure<E>> {
        let mut engine = Engine::new(
            &self.identity,
            &self.combine,
            max_value_bytes,
            before_combine,
        );
        let candidate = update(&self.root, partition, value, &mut engine)
            .map_err(|reason| engine.failure(reason))?;
        self.root = candidate.link;
        Ok(engine.metrics())
    }

    pub fn insert_checked<E>(
        &mut self,
        partition: PartitionIdentity,
        value: T,
        max_value_bytes: u64,
        before_combine: impl FnMut() -> Result<(), E>,
    ) -> Result<ReductionMetrics, ReductionRunFailure<E>> {
        if contains(&self.root, partition) {
            return Err(failure(ReductionRunStop::Denial(
                ReductionDenial::IdentityAlreadyPresent(partition),
            )));
        }
        let mut engine = Engine::new(
            &self.identity,
            &self.combine,
            max_value_bytes,
            before_combine,
        );
        let candidate = insert(&self.root, partition, value, &mut engine)
            .map_err(|reason| engine.failure(reason))?;
        self.root = candidate;
        self.len += 1;
        Ok(engine.metrics())
    }

    pub fn delete_checked<E>(
        &mut self,
        partition: PartitionIdentity,
        max_value_bytes: u64,
        before_combine: impl FnMut() -> Result<(), E>,
    ) -> Result<ReductionMetrics, ReductionRunFailure<E>> {
        if !contains(&self.root, partition) {
            return Err(failure(ReductionRunStop::Denial(
                ReductionDenial::UnknownIdentity(partition),
            )));
        }
        let mut engine = Engine::new(
            &self.identity,
            &self.combine,
            max_value_bytes,
            before_combine,
        );
        let candidate =
            delete(&self.root, partition, &mut engine).map_err(|reason| engine.failure(reason))?;
        self.root = candidate;
        self.len -= 1;
        Ok(engine.metrics())
    }

    /// Structural/unmetered edit. Checked execution uses `update_checked`.
    pub fn update(
        &mut self,
        partition: PartitionIdentity,
        value: T,
    ) -> Result<ReductionMetrics, ReductionDenial> {
        self.update_checked(partition, value, u64::MAX, || Ok::<_, ()>(()))
            .map_err(pure_denial)
    }

    /// Structural/unmetered edit. Checked execution uses `insert_checked`.
    pub fn insert(
        &mut self,
        partition: PartitionIdentity,
        value: T,
    ) -> Result<ReductionMetrics, ReductionDenial> {
        self.insert_checked(partition, value, u64::MAX, || Ok::<_, ()>(()))
            .map_err(pure_denial)
    }

    /// Structural/unmetered edit. Checked execution uses `delete_checked`.
    pub fn delete(
        &mut self,
        partition: PartitionIdentity,
    ) -> Result<ReductionMetrics, ReductionDenial> {
        self.delete_checked(partition, u64::MAX, || Ok::<_, ()>(()))
            .map_err(pure_denial)
    }
}
