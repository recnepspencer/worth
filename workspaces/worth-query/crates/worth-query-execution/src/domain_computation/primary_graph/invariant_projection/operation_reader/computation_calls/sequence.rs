//! Measured reader calls in canonical order, with checked range charges.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
use crate::domain_computation::primary_graph::application_contribution::{
    QueryMemoryReservation, QueryRequestExecution,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Range,
};
mod funding;
mod issue;
#[cfg(test)]
mod tests;
use funding::shape_bound;
mod reached;
use reached::ReachedCalls;
#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) enum CallSequenceDenial {
    Resource(Resource),
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

pub(in crate::domain_computation::primary_graph) struct CallShape {
    ordinals: BTreeMap<ComputationRead, usize>,
    prefix: usize,
    bytes: u64,
    _memory: QueryMemoryReservation,
}
impl CallShape {
    pub(in crate::domain_computation::primary_graph) fn calls(
        &self,
    ) -> impl Iterator<Item = ComputationRead> + '_ {
        self.ordinals.keys().copied()
    }
    pub(in crate::domain_computation::primary_graph) fn ordinal(
        &self,
        read: ComputationRead,
    ) -> Option<usize> {
        self.ordinals.get(&read).copied()
    }
    pub(in crate::domain_computation::primary_graph) fn len(&self) -> usize {
        self.ordinals.len()
    }
}
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
    start: usize,
    end: usize,
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
        self.reached.contains_in(entity, range)
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
    pub(in crate::domain_computation::primary_graph) fn patched<'a, I>(
        self: &Arc<Self>,
        changes: I,
        execution: &QueryRequestExecution<'_>,
    ) -> Result<Arc<Self>, CallSequenceDenial>
    where
        I: Iterator<Item = (ComputationRead, &'a ComputationCallCharge, u64)>,
    {
        let mut changes = changes.peekable();
        if changes.peek().is_none() {
            return Ok(Arc::clone(self));
        }
        let mut root = self.root.clone();
        let mut reached = self.reached.clone();
        let (mut pairs, mut payload) = (self.pairs, self.payload);
        let policy = execution.allocation_policy()?;
        for (read, charge, declared) in changes {
            policy
                .check_live()
                .map_err(worth_execution::ExecutionOrderedMapDenial::from)?;
            let ordinal = self.shape.ordinal(read).ok_or(Resource::CapacityOverflow)?;
            let old = leaf(root.as_deref().expect("known call root"), ordinal);
            for entity in old.0.reached.iter() {
                observed::computation_call_reach();
                reached.remove(*entity, ordinal, policy)?;
            }
            for entity in charge.reached.iter() {
                observed::computation_call_reach();
                reached.insert(*entity, ordinal, policy)?;
            }
            pairs = pairs
                .checked_sub(old.0.reached.len())
                .and_then(|v| v.checked_add(charge.reached.len()))
                .ok_or(Resource::CapacityOverflow)?;
            payload = payload
                .checked_sub(old.0.additional_bytes().ok_or(Resource::CapacityOverflow)?)
                .and_then(|v| v.checked_add(charge.additional_bytes()?))
                .ok_or(Resource::CapacityOverflow)?;
            root = Some(patch(
                root.as_ref().unwrap(),
                ordinal,
                &(charge.clone(), declared),
                execution,
            )?);
        }
        let memory = execution.reserve((size_of::<Self>() + 2 * size_of::<usize>()) as u64)?;
        let bytes = memory
            .bytes()
            .checked_add(self.shape.bytes)
            .and_then(|v| v.checked_add(root.as_ref().map_or(0, |n| n.bytes)))
            .and_then(|v| v.checked_add(reached.bytes()))
            .ok_or(Resource::CapacityOverflow)?;
        Ok(Arc::new(Self {
            shape: Arc::clone(&self.shape),
            root,
            reached,
            pairs,
            payload,
            bytes,
            _memory: memory,
        }))
    }
}

use crate::domain_computation::primary_graph::fact_verification_observation as observed;
fn build(
    start: usize,
    entries: &[(ComputationRead, Call)],
    execution: &QueryRequestExecution<'_>,
) -> Result<Option<Arc<Node>>, Resource> {
    if entries.is_empty() {
        return Ok(None);
    }
    let end = start + entries.len();
    let (kind, summary) = if entries.len() == 1 {
        let (read, call) = &entries[0];
        (
            Kind::Leaf(*read, call.clone()),
            Some(CallSummary::leaf(&call.0, call.1)),
        )
    } else {
        let split = entries.len() / 2;
        let a = build(start, &entries[..split], execution)?.unwrap();
        let b = build(start + split, &entries[split..], execution)?.unwrap();
        let summary = a.summary.zip(b.summary).and_then(|(a, b)| a.then(b));
        (Kind::Branch(a, b), summary)
    };
    funded_node(start, end, kind, summary, execution).map(Some)
}
fn funded_node(
    start: usize,
    end: usize,
    kind: Kind,
    summary: Option<CallSummary>,
    execution: &QueryRequestExecution<'_>,
) -> Result<Arc<Node>, Resource> {
    let payload = match &kind {
        Kind::Leaf(_, call) => call.0.additional_bytes(),
        Kind::Branch(a, b) => a.bytes.checked_add(b.bytes),
    }
    .ok_or(Resource::CapacityOverflow)?;
    let allocation = (size_of::<Node>() + 2 * size_of::<usize>()) as u64;
    let retained = allocation
        .checked_add(payload)
        .ok_or(Resource::CapacityOverflow)?;
    let held = if matches!(&kind, Kind::Leaf(..)) {
        retained
    } else {
        allocation
    };
    let memory = execution.reserve(held)?;
    observed::computation_call_copy();
    Ok(Arc::new(Node {
        start,
        end,
        kind,
        summary,
        bytes: retained,
        _memory: memory,
    }))
}
fn leaf(node: &Node, ordinal: usize) -> &Call {
    match &node.kind {
        Kind::Leaf(_, call) => call,
        Kind::Branch(a, b) => leaf(if ordinal < a.end { a } else { b }, ordinal),
    }
}
fn patch(
    node: &Arc<Node>,
    ordinal: usize,
    call: &Call,
    execution: &QueryRequestExecution<'_>,
) -> Result<Arc<Node>, Resource> {
    let (kind, summary) = match &node.kind {
        Kind::Leaf(read, _) => (
            Kind::Leaf(*read, call.clone()),
            Some(CallSummary::leaf(&call.0, call.1)),
        ),
        Kind::Branch(a, b) => {
            let (a, b) = if ordinal < a.end {
                (patch(a, ordinal, call, execution)?, Arc::clone(b))
            } else {
                (Arc::clone(a), patch(b, ordinal, call, execution)?)
            };
            let summary = a.summary.zip(b.summary).and_then(|(a, b)| a.then(b));
            (Kind::Branch(a, b), summary)
        }
    };
    funded_node(node.start, node.end, kind, summary, execution)
}
fn summary(node: &Node, range: &Range<usize>) -> Option<CallSummary> {
    observed::computation_call_range();
    if range.start <= node.start && node.end <= range.end {
        return node.summary;
    }
    match &node.kind {
        Kind::Leaf(..) => None,
        Kind::Branch(a, b) => {
            if range.end <= a.end {
                summary(a, range)
            } else if range.start >= b.start {
                summary(b, range)
            } else {
                summary(a, range)?.then(summary(b, range)?)
            }
        }
    }
}
fn visit<E>(
    node: &Node,
    range: &Range<usize>,
    marked: &BTreeSet<usize>,
    visitor: &mut impl FnMut(CallVisit<'_>) -> Result<bool, E>,
) -> Result<(), E> {
    if range.end <= node.start || node.end <= range.start {
        return Ok(());
    }
    observed::computation_call_range();
    if range.start <= node.start
        && node.end <= range.end
        && marked.range(node.start..node.end).next().is_none()
        && let Some(summary) = &node.summary
        && visitor(CallVisit::Range(node.start..node.end, summary))?
    {
        return Ok(());
    }
    match &node.kind {
        Kind::Leaf(read, (charge, declared)) => {
            visitor(CallVisit::Leaf(node.start, *read, charge, *declared))?;
        }
        Kind::Branch(a, b) => {
            visit(a, range, marked, visitor)?;
            visit(b, range, marked, visitor)?;
        }
    }
    Ok(())
}
