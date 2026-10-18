//! Measured reader calls in canonical order, with checked range charges.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
use crate::domain_computation::primary_graph::application_contribution::{
    QueryMemoryReservation, QueryRequestExecution,
};
use std::{collections::BTreeSet, ops::Range};
#[cfg(test)]
mod editing_tests;
mod funding;
mod issue;
#[cfg(test)]
mod tests;

mod reached;
use reached::ReachedCalls;
#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) enum CallSequenceDenial {
    Resource(Resource),
    InvalidEdit,
    Storage(worth_execution::ExecutionOrderedMapDenial),
}
impl From<Resource> for CallSequenceDenial {
    fn from(cause: Resource) -> Self {
        Self::Resource(cause)
    }
}
impl From<worth_execution::ExecutionOrderedMapDenial> for CallSequenceDenial {
    fn from(cause: worth_execution::ExecutionOrderedMapDenial) -> Self {
        Self::Storage(cause)
    }
}
type Call = (ComputationCallCharge, u64);

mod shape;
pub(in crate::domain_computation::primary_graph) use shape::{CallShape, CallShapeTransition};
mod tree;
use tree::{build, leaf, patch, summary, visit};
mod editing;
#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph) struct CallSummary {
    pub(super) work: WorthQueryInvariantProjectionWork,
    pub(super) demand: usize,
    pub(super) consumed: usize,
    pub(in crate::domain_computation::primary_graph) declared: u64,
}
impl CallSummary {
    fn leaf(charge: &ComputationCallCharge, declared: u64) -> Self {
        Self {
            work: charge.work,
            demand: charge.demand,
            consumed: charge.consumed,
            declared,
        }
    }
    fn then(self, next: Self) -> Option<Self> {
        Some(Self {
            work: self.work.with_carried(next.work)?,
            demand: self.demand.max(self.consumed.checked_add(next.demand)?),
            consumed: self.consumed.checked_add(next.consumed)?,
            declared: self.declared.checked_add(next.declared)?,
        })
    }
}
struct Node {
    count: usize,
    height: u8,
    summary: Option<CallSummary>,
    kind: Kind,
    bytes: u64,
    _memory: QueryMemoryReservation,
}
enum Kind {
    Leaf(ComputationRead, Call),
    Branch(Arc<Node>, Arc<Node>),
}
pub(in crate::domain_computation::primary_graph) enum CallVisit<'a> {
    Range(Range<usize>, &'a CallSummary),
    Leaf(usize, ComputationRead, &'a ComputationCallCharge, u64),
}
/// No request borrow or parent sequence is stored. Each cell funds its own
/// representation; unchanged nodes and the exact ordinal shape share grants.
pub(in crate::domain_computation::primary_graph) struct ComputationCallSequence {
    shape: Arc<CallShape>,
    root: Option<Arc<Node>>,
    reached: ReachedCalls,
    pairs: usize,
    payload: u64,
    bytes: u64,
    _memory: QueryMemoryReservation,
}
impl ComputationCallSequence {
    pub(in crate::domain_computation::primary_graph) fn shape(&self) -> &Arc<CallShape> {
        &self.shape
    }
    pub(in crate::domain_computation::primary_graph) fn prefix(&self) -> usize {
        self.shape.prefix
    }
    pub(in crate::domain_computation::primary_graph) fn bytes(&self) -> u64 {
        self.bytes
    }
    /// Already funded backing travelling with this immutable value. This is
    /// transport custody only; retention still charges the complete capacity.
    pub(in crate::domain_computation::primary_graph) fn funded_bytes(&self) -> u64 {
        self.bytes
    }
    pub(in crate::domain_computation::primary_graph) fn summary(
        &self,
        range: Range<usize>,
    ) -> Option<CallSummary> {
        summary(self.root.as_deref()?, &range)
    }
    pub(in crate::domain_computation::primary_graph) fn contains_in(
        &self,
        entity: EntityId,
        range: &Range<usize>,
    ) -> bool {
        observed::computation_call_scope();
        let Some(first) = self.shape.call_at(range.start) else {
            return false;
        };
        if range.start >= range.end || range.end > self.shape.len() {
            return false;
        }
        self.reached
            .contains_in(entity, first, self.shape.call_at(range.end))
    }
    pub(in crate::domain_computation::primary_graph) fn visit<E>(
        &self,
        range: Range<usize>,
        marked: &BTreeSet<usize>,
        visitor: &mut impl FnMut(CallVisit<'_>) -> Result<bool, E>,
    ) -> Result<(), E> {
        if let Some(root) = &self.root {
            visit(root, &range, marked, visitor)?;
        }
        Ok(())
    }
}

use crate::domain_computation::primary_graph::fact_verification_observation as observed;
