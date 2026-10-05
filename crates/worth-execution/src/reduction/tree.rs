use std::sync::Arc;

use worth_foundational::PartitionIdentity;

use crate::{oracle::CanonicalBits, report::ChargedBytes};

use super::plan::{ReductionDenial, ReductionPlan};

mod construction;
mod edit;
mod memory;
mod recombine;
mod scheduled;
mod staged;

use construction::Shape;
use edit::{contains, delete, insert, update};
use memory::node_bytes;
use recombine::Engine;
pub(crate) use scheduled::ScheduledReductionError;

pub(super) type Link<T> = Option<Arc<Node<T>>>;

pub(super) struct Node<T> {
    pub(super) identity: PartitionIdentity,
    pub(super) value: T,
    pub(super) aggregate: T,
    pub(super) left: Link<T>,
    pub(super) right: Link<T>,
    pub(super) retained_bytes: u64,
}

/// Completed node recombinations and entered combine calls for one operation.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ReductionMetrics {
    pub structural_visits: u64,
    pub recombined_nodes: u64,
    pub combine_calls: u64,
    pub charged_work: u64,
    pub charged_span: u64,
}

impl ReductionMetrics {
    pub(crate) fn add(self, other: Self) -> Result<Self, ReductionRunStop<()>> {
        Ok(Self {
            structural_visits: self
                .structural_visits
                .checked_add(other.structural_visits)
                .ok_or(ReductionRunStop::WorkCounterOverflow)?,
            recombined_nodes: self
                .recombined_nodes
                .checked_add(other.recombined_nodes)
                .ok_or(ReductionRunStop::WorkCounterOverflow)?,
            combine_calls: self
                .combine_calls
                .checked_add(other.combine_calls)
                .ok_or(ReductionRunStop::WorkCounterOverflow)?,
            charged_work: self
                .charged_work
                .checked_add(other.charged_work)
                .ok_or(ReductionRunStop::WorkCounterOverflow)?,
            charged_span: self
                .charged_span
                .checked_add(other.charged_span)
                .ok_or(ReductionRunStop::WorkCounterOverflow)?,
        })
    }

    pub(super) fn record_visit(&mut self) -> Result<(), ReductionRunStop<()>> {
        self.structural_visits = self
            .structural_visits
            .checked_add(1)
            .ok_or(ReductionRunStop::WorkCounterOverflow)?;
        self.charged_work = self
            .charged_work
            .checked_add(1)
            .ok_or(ReductionRunStop::WorkCounterOverflow)?;
        self.charged_span = self
            .charged_span
            .checked_add(1)
            .ok_or(ReductionRunStop::WorkCounterOverflow)?;
        Ok(())
    }

    pub(super) fn record_combine(&mut self) -> Result<(), ReductionRunStop<()>> {
        let calls = self
            .combine_calls
            .checked_add(1)
            .ok_or(ReductionRunStop::WorkCounterOverflow)?;
        let work = self
            .charged_work
            .checked_add(1)
            .ok_or(ReductionRunStop::WorkCounterOverflow)?;
        let span = self
            .charged_span
            .checked_add(1)
            .ok_or(ReductionRunStop::WorkCounterOverflow)?;
        self.combine_calls = calls;
        self.charged_work = work;
        self.charged_span = span;
        Ok(())
    }

    pub(super) fn record_node(&mut self) -> Result<(), ReductionRunStop<()>> {
        self.recombined_nodes = self
            .recombined_nodes
            .checked_add(1)
            .ok_or(ReductionRunStop::WorkCounterOverflow)?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ReductionRunStop<E> {
    Hook(E),
    Panic,
    Denial(ReductionDenial),
    ResultCapacityExceeded,
    WorkCounterOverflow,
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
pub struct ReductionTree<T, F> {
    root: Link<T>,
    identity: T,
    combine: F,
    len: usize,
}

impl<T: Clone, F: Clone> Clone for ReductionTree<T, F> {
    fn clone(&self) -> Self {
        Self {
            root: self.root.clone(),
            identity: self.identity.clone(),
            combine: self.combine.clone(),
            len: self.len,
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
        if plan.identities().len() != values.len() {
            return Err(failure(ReductionRunStop::Denial(
                ReductionDenial::ValueCountMismatch,
            )));
        }
        let mut tree = Self {
            root: None,
            identity,
            combine,
            len: values.len(),
        };
        let mut engine = Engine::new(
            &tree.identity,
            &tree.combine,
            max_value_bytes,
            before_combine,
        );
        engine
            .validate(&tree.identity)
            .map_err(|reason| engine.failure(reason))?;
        let shape = Shape::build(plan.identities(), values, &mut engine)
            .map_err(|reason| engine.failure(reason))?;
        let shape_span = engine.metrics().charged_span;
        if let Some(spec) = shape.full_spec() {
            let (candidate, dependency_span) = shape
                .evaluate(spec, &mut engine)
                .map_err(|reason| engine.failure(reason))?;
            tree.root = candidate;
            let span = shape_span
                .checked_add(dependency_span)
                .ok_or_else(|| engine.failure(ReductionRunStop::WorkCounterOverflow))?;
            engine.set_full_span(span);
        }
        let metrics = engine.metrics();
        Ok((tree, metrics))
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

    pub fn result(&self) -> &T {
        self.root
            .as_ref()
            .map_or(&self.identity, |node| &node.aggregate)
    }

    pub fn partition_count(&self) -> usize {
        self.len
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

fn failure<E>(reason: ReductionRunStop<E>) -> ReductionRunFailure<E> {
    ReductionRunFailure {
        reason,
        metrics: ReductionMetrics::default(),
    }
}

fn pure_denial(failure: ReductionRunFailure<()>) -> ReductionDenial {
    match failure.reason {
        ReductionRunStop::Denial(denial) => denial,
        ReductionRunStop::Panic => ReductionDenial::ReducerPanic,
        ReductionRunStop::Hook(()) => unreachable!("infallible structural hook"),
        ReductionRunStop::ResultCapacityExceeded => ReductionDenial::ResultCapacityExceeded,
        ReductionRunStop::WorkCounterOverflow => ReductionDenial::WorkCounterOverflow,
    }
}
