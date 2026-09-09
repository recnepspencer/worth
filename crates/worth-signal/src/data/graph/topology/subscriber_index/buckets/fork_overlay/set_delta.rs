use std::collections::BTreeSet;
use std::ops::Bound::{Excluded, Unbounded};

use crate::data::handle::NodeId;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in super::super) struct SetDelta {
    pub(super) added: im::OrdSet<NodeId>,
    pub(super) removed: im::OrdSet<NodeId>,
    pub(super) retired_base_intervals: im::OrdMap<NodeId, NodeId>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in super::super) struct SetMergeTraversal {
    pub(super) base_members: usize,
    pub(super) range_seeks: usize,
}

impl SetDelta {
    pub(super) fn contains(&self, base_contains: bool, node: &NodeId) -> bool {
        self.added.contains(node) || (base_contains && !self.removed.contains(node))
    }

    pub(super) fn insert(&mut self, base: Option<&BTreeSet<NodeId>>, node: NodeId) {
        let base_contains = base.is_some_and(|base| base.contains(&node));
        if self.contains(base_contains, &node) {
            return;
        }
        if self.removed.remove(&node).is_some() {
            record_base_readmission(
                base.expect("retired member must belong to the immutable base"),
                &mut self.retired_base_intervals,
                node,
            );
        } else {
            self.added.insert(node);
        }
    }

    pub(super) fn remove(&mut self, base: Option<&BTreeSet<NodeId>>, node: NodeId) {
        let base_contains = base.is_some_and(|base| base.contains(&node));
        if !self.contains(base_contains, &node) {
            return;
        }
        if self.added.remove(&node).is_none() {
            self.removed.insert(node);
            record_base_retirement(
                base.expect("removed inherited member must belong to the immutable base"),
                &mut self.retired_base_intervals,
                node,
            );
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.retired_base_intervals.is_empty()
    }
}

fn record_base_retirement(
    base: &BTreeSet<NodeId>,
    intervals: &mut im::OrdMap<NodeId, NodeId>,
    node: NodeId,
) {
    let left_start = base
        .range(..node)
        .next_back()
        .and_then(|predecessor| containing_interval(intervals, *predecessor))
        .map(|(start, _)| start);
    let right_start = base
        .range((Excluded(node), Unbounded))
        .next()
        .and_then(|successor| intervals.get(successor).map(|_| *successor));
    let start = left_start.unwrap_or(node);
    let end = right_start
        .and_then(|right| intervals.get(&right).copied())
        .unwrap_or(node);
    if let Some(left) = left_start {
        intervals.remove(&left);
    }
    if let Some(right) = right_start {
        intervals.remove(&right);
    }
    intervals.insert(start, end);
}

fn record_base_readmission(
    base: &BTreeSet<NodeId>,
    intervals: &mut im::OrdMap<NodeId, NodeId>,
    node: NodeId,
) {
    let Some((start, end)) = containing_interval(intervals, node) else {
        return;
    };
    intervals.remove(&start);
    if start < node {
        let predecessor = *base
            .range(..node)
            .next_back()
            .expect("non-start interval member has a predecessor");
        intervals.insert(start, predecessor);
    }
    if node < end {
        let successor = *base
            .range((Excluded(node), Unbounded))
            .next()
            .expect("non-end interval member has a successor");
        intervals.insert(successor, end);
    }
}

fn containing_interval(
    intervals: &im::OrdMap<NodeId, NodeId>,
    node: NodeId,
) -> Option<(NodeId, NodeId)> {
    intervals.get(&node).map(|end| (node, *end)).or_else(|| {
        intervals
            .get_prev(&node)
            .filter(|(_, end)| **end >= node)
            .map(|(start, end)| (*start, *end))
    })
}
