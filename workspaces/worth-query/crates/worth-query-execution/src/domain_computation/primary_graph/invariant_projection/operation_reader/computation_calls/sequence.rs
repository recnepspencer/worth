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
use funding::{bound, shape_bound};
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
}
enum Kind {
    Leaf(ComputationRead, Call),
    Branch(Arc<Node>, Arc<Node>),
}
pub(in crate::domain_computation::primary_graph) enum CallVisit<'a> {
    Range(Range<usize>, &'a CallSummary),
    Leaf(usize, ComputationRead, &'a ComputationCallCharge, u64),
}
/// No request borrow or parent sequence is stored. Each root funds its whole
/// representation; unchanged nodes and the exact ordinal shape may be shared.
pub(in crate::domain_computation::primary_graph) struct ComputationCallSequence {
    shape: Arc<CallShape>,
    root: Option<Arc<Node>>,
    reached: im::OrdMap<EntityId, im::OrdSet<usize>>,
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
        self._memory
            .bytes()
            .saturating_add(self.shape._memory.bytes())
            .min(self.bytes)
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
        self.reached
            .get(&entity)
            .is_some_and(|ordinals| ordinals.range(range.clone()).next().is_some())
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
    ) -> Result<Arc<Self>, Resource>
    where
        I: Iterator<Item = (ComputationRead, &'a ComputationCallCharge, u64)>,
    {
        let mut changes = changes.peekable();
        if changes.peek().is_none() {
            return Ok(Arc::clone(self));
        }
        let mut memory = execution.reserve(
            self.bytes
                .checked_mul(2)
                .ok_or(Resource::CapacityOverflow)?,
        )?;
        let (mut added_pairs, mut added_payload) = (0_usize, 0_u64);
        let (mut root, mut reached, mut pairs, mut payload) = (
            self.root.clone(),
            self.reached.clone(),
            self.pairs,
            self.payload,
        );
        for (read, charge, declared) in changes {
            added_pairs = added_pairs
                .checked_add(charge.reached.len())
                .ok_or(Resource::CapacityOverflow)?;
            added_payload = added_payload
                .checked_add(
                    charge
                        .additional_bytes()
                        .ok_or(Resource::CapacityOverflow)?,
                )
                .ok_or(Resource::CapacityOverflow)?;
            let peak = bound(
                self.shape.len(),
                self.reached
                    .len()
                    .checked_add(added_pairs)
                    .ok_or(Resource::CapacityOverflow)?,
                self.pairs
                    .checked_add(added_pairs)
                    .ok_or(Resource::CapacityOverflow)?,
                self.payload
                    .checked_add(added_payload)
                    .ok_or(Resource::CapacityOverflow)?,
            )
            .and_then(|bytes| bytes.checked_add(self.bytes))
            .ok_or(Resource::CapacityOverflow)?;
            if peak > memory.bytes() {
                memory.resize(peak)?;
            }
            let call = (charge.clone(), declared);
            let ordinal = self.shape.ordinal(read).ok_or(Resource::CapacityOverflow)?;
            let old = leaf(root.as_deref().expect("a known call has a root"), ordinal);
            for entity in old.0.reached.iter() {
                observed::computation_call_reach();
                let set = reached.get_mut(entity).expect("recorded reached identity");
                set.remove(&ordinal);
                if set.is_empty() {
                    reached.remove(entity);
                }
            }
            for entity in call.0.reached.iter() {
                observed::computation_call_reach();
                reached.entry(*entity).or_default().insert(ordinal);
            }
            pairs = pairs
                .checked_sub(old.0.reached.len())
                .and_then(|n| n.checked_add(call.0.reached.len()))
                .ok_or(Resource::CapacityOverflow)?;
            payload = payload
                .checked_sub(old.0.additional_bytes().ok_or(Resource::CapacityOverflow)?)
                .and_then(|n| n.checked_add(call.0.additional_bytes()?))
                .ok_or(Resource::CapacityOverflow)?;
            root = Some(patch(root.as_ref().unwrap(), ordinal, &call));
        }
        let bytes = bound(self.shape.len(), reached.len(), pairs, payload)
            .ok_or(Resource::CapacityOverflow)?;
        let bytes = bytes
            .checked_add(self.shape.bytes)
            .ok_or(Resource::CapacityOverflow)?;
        memory.resize(bytes)?;
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
fn build(start: usize, entries: &[(ComputationRead, Call)]) -> Option<Arc<Node>> {
    if entries.is_empty() {
        return None;
    }
    observed::computation_call_copy();
    let end = start + entries.len();
    let (kind, summary) = if entries.len() == 1 {
        let (read, call) = &entries[0];
        (
            Kind::Leaf(*read, call.clone()),
            Some(CallSummary::leaf(&call.0, call.1)),
        )
    } else {
        let split = entries.len() / 2;
        let left = build(start, &entries[..split]).unwrap();
        let right = build(start + split, &entries[split..]).unwrap();
        let summary = left.summary.zip(right.summary).and_then(|(a, b)| a.then(b));
        (Kind::Branch(left, right), summary)
    };
    Some(Arc::new(Node {
        start,
        end,
        summary,
        kind,
    }))
}
fn leaf(node: &Node, ordinal: usize) -> &Call {
    match &node.kind {
        Kind::Leaf(_, call) => call,
        Kind::Branch(a, b) => leaf(if ordinal < a.end { a } else { b }, ordinal),
    }
}
fn patch(node: &Arc<Node>, ordinal: usize, call: &Call) -> Arc<Node> {
    observed::computation_call_copy();
    let (kind, summary) = match &node.kind {
        Kind::Leaf(read, _) => (
            Kind::Leaf(*read, call.clone()),
            Some(CallSummary::leaf(&call.0, call.1)),
        ),
        Kind::Branch(a, b) => {
            let (a, b) = if ordinal < a.end {
                (patch(a, ordinal, call), Arc::clone(b))
            } else {
                (Arc::clone(a), patch(b, ordinal, call))
            };
            let summary = a.summary.zip(b.summary).and_then(|(a, b)| a.then(b));
            (Kind::Branch(a, b), summary)
        }
    };
    Arc::new(Node {
        start: node.start,
        end: node.end,
        summary,
        kind,
    })
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
