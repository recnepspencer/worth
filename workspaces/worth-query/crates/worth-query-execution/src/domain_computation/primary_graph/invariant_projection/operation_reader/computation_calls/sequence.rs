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
type Call = (ComputationCallCharge, Option<u64>);

mod shape;
pub(in crate::domain_computation::primary_graph) use shape::{CallShape, CallShapeTransition};
mod tree;
use tree::{build, leaf, patch, summary, visit};
mod editing;
#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph) struct CallSummary {
    pub(super) accounting: Option<ComputationCallAccounting>,
    pub(in crate::domain_computation::primary_graph) declared: Option<u64>,
}
impl CallSummary {
    fn native() -> Self {
        Self {
            accounting: None,
            declared: None,
        }
    }
    fn leaf(charge: &ComputationCallCharge, declared: Option<u64>) -> Self {
        Self {
            accounting: charge.accounting,
            declared,
        }
    }
    fn then(self, next: Self) -> Option<Self> {
        let accounting = match self.accounting.zip(next.accounting) {
            Some((a, b)) => Some(ComputationCallAccounting {
                work: a.work.with_carried(b.work)?,
                demand: a.demand.max(a.consumed.checked_add(b.demand)?),
                consumed: a.consumed.checked_add(b.consumed)?,
            }),
            None => None,
        };
        let declared = match self.declared.zip(next.declared) {
            Some((a, b)) => Some(a.checked_add(b)?),
            None => None,
        };
        Some(Self {
            accounting,
            declared,
        })
    }
}
struct Node {
    count: usize,
    height: u8,
    summary: Option<CallSummary>,
    kind: Kind,
    #[cfg(test)]
    bytes: Option<u64>,
    _memory: Option<QueryMemoryReservation>,
}
enum Kind {
    Leaf(ComputationRead, Call),
    Branch(Arc<Node>, Arc<Node>),
}
pub(in crate::domain_computation::primary_graph) enum CallVisit<'a> {
    Range(Range<usize>, &'a CallSummary),
    Leaf(
        usize,
        ComputationRead,
        &'a ComputationCallCharge,
        Option<u64>,
    ),
}
/// No request borrow or parent sequence is stored. Each cell funds its own
/// representation; unchanged nodes and the exact ordinal shape share grants.
pub(in crate::domain_computation::primary_graph) struct ComputationCallSequence {
    shape: Arc<CallShape>,
    root: Option<Arc<Node>>,
    reached: ReachedCalls,
    #[cfg(test)]
    bytes: Option<u64>,
    _memory: Option<QueryMemoryReservation>,
}
impl ComputationCallSequence {
    pub(in crate::domain_computation::primary_graph) fn shape(&self) -> &Arc<CallShape> {
        &self.shape
    }
    pub(in crate::domain_computation::primary_graph) fn prefix(&self) -> usize {
        self.shape.prefix
    }
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn bytes(&self) -> Option<u64> {
        self.bytes
    }
    pub(in crate::domain_computation::primary_graph) fn summary(
        &self,
        range: Range<usize>,
        accounted: bool,
    ) -> Option<CallSummary> {
        if !accounted {
            return (range.start < range.end && range.end <= self.shape.len())
                .then(CallSummary::native);
        }
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
