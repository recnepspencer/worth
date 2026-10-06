use std::{collections::VecDeque, sync::Arc};

use worth_foundational::PartitionIdentity;

use crate::{oracle::CanonicalBits, report::ChargedBytes};

use super::{
    super::plan::priority, recombine::Engine, Link, Node, ReductionMetrics, ReductionRunStop,
};

pub(super) struct ShapeNode<T> {
    identity: PartitionIdentity,
    value: T,
    left: Option<usize>,
    right: Option<usize>,
}

/// The canonical Cartesian shape is constructed once, in linear time.
pub(crate) struct Shape<T> {
    nodes: Vec<ShapeNode<T>>,
    root: Option<usize>,
    split_order: Vec<SubtreeSpec>,
    split_flags: Vec<bool>,
}

pub(crate) struct FrontierPlan {
    pub(crate) tasks: Vec<SubtreeSpec>,
    pub(crate) events: Vec<FrontierEvent>,
}

#[derive(Clone, Copy)]
pub(crate) enum FrontierEvent {
    Enter(SubtreeSpec),
    Task(SubtreeSpec),
    Finish(SubtreeSpec),
}

#[derive(Clone, Copy)]
pub(crate) struct SubtreeSpec {
    pub(crate) identity: PartitionIdentity,
    root: usize,
    first: usize,
    len: usize,
}

impl SubtreeSpec {
    pub(crate) fn len(self) -> usize {
        self.len
    }

    pub(crate) fn index(self) -> usize {
        self.root
    }
}

impl ChargedBytes for SubtreeSpec {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

pub(crate) struct SubtreeResult<T> {
    pub(super) root: Link<T>,
    pub(crate) metrics: ReductionMetrics,
}

impl<T: ChargedBytes> ChargedBytes for SubtreeResult<T> {
    fn additional_charged_bytes(&self) -> u64 {
        self.root.as_ref().map_or(0, |node| node.retained_bytes)
    }
}

pub(super) struct Built<T> {
    root: Arc<Node<T>>,
    span: u64,
}

impl<T> Shape<T> {
    pub(crate) fn len(&self) -> usize {
        self.nodes.len()
    }

    pub(crate) fn node_value(&self, spec: SubtreeSpec) -> (PartitionIdentity, &T) {
        let node = &self.nodes[spec.root];
        (node.identity, &node.value)
    }

    pub(crate) fn children_of(&self, spec: SubtreeSpec) -> [Option<SubtreeSpec>; 2] {
        let node = &self.nodes[spec.root];
        [
            node.left.map(|root| SubtreeSpec {
                identity: self.nodes[root].identity,
                root,
                first: spec.first,
                len: spec.root - spec.first,
            }),
            node.right.map(|root| SubtreeSpec {
                identity: self.nodes[root].identity,
                root,
                first: spec.root + 1,
                len: spec.first + spec.len - spec.root - 1,
            }),
        ]
    }

    /// The split decisions were all checkpointed in `build`, so width only
    /// selects a prefix of an already metered, deterministic BFS plan.
    pub(crate) fn frontier(&mut self, target_tasks: usize) -> FrontierPlan {
        let Some(root) = self.full_spec() else {
            return FrontierPlan {
                tasks: Vec::new(),
                events: Vec::new(),
            };
        };
        let mut count = 1_usize;
        for &spec in &self.split_order {
            if count >= target_tasks.max(1) {
                break;
            }
            self.split_flags[spec.root] = true;
            count += self.children_of(spec).into_iter().flatten().count() - 1;
        }
        let mut tasks = Vec::with_capacity(count);
        let capacity = self.nodes.len().saturating_mul(2);
        let mut events = Vec::with_capacity(capacity);
        let mut pending = Vec::with_capacity(capacity);
        pending.push((root, false));
        while let Some((spec, ready)) = pending.pop() {
            if ready {
                events.push(FrontierEvent::Finish(spec));
                continue;
            }
            if self.split_flags[spec.root] {
                events.push(FrontierEvent::Enter(spec));
                let [left, right] = self.children_of(spec);
                pending.push((spec, true));
                if let Some(right) = right {
                    pending.push((right, false));
                }
                if let Some(left) = left {
                    pending.push((left, false));
                }
            } else {
                tasks.push(spec);
                events.push(FrontierEvent::Task(spec));
            }
        }
        FrontierPlan { tasks, events }
    }

    pub(crate) fn full_spec(&self) -> Option<SubtreeSpec> {
        self.root.map(|root| SubtreeSpec {
            identity: self.nodes[root].identity,
            root,
            first: 0,
            len: self.nodes.len(),
        })
    }

    /// Metrics for the first admitted checkpoints of this subtree's fixed
    /// enter/left/right/finish/two-combine traversal. Used only when the
    /// enclosing ceiling stops before a speculative task can be accepted.
    pub(crate) fn prefix_metrics(
        &self,
        spec: SubtreeSpec,
        accepted_work: u64,
    ) -> Option<ReductionMetrics> {
        let mut remaining = accepted_work;
        let mut metrics = ReductionMetrics::default();
        let mut traversal = Vec::with_capacity(spec.len.saturating_mul(2));
        traversal.push((spec.root, false));
        while let Some((index, ready)) = traversal.pop() {
            if remaining == 0 {
                break;
            }
            let node = &self.nodes[index];
            metrics.record_visit().ok()?;
            remaining -= 1;
            if !ready {
                traversal.push((index, true));
                if let Some(right) = node.right {
                    traversal.push((right, false));
                }
                if let Some(left) = node.left {
                    traversal.push((left, false));
                }
                continue;
            }
            for _ in 0..2 {
                if remaining == 0 {
                    return Some(metrics);
                }
                metrics.record_combine().ok()?;
                remaining -= 1;
            }
            metrics.record_node().ok()?;
        }
        (remaining == 0).then_some(metrics)
    }
}

impl<T: Clone + ChargedBytes + CanonicalBits> Shape<T> {
    pub(super) fn build<F, H, E>(
        identities: &[PartitionIdentity],
        values: Vec<T>,
        engine: &mut Engine<'_, T, F, H>,
    ) -> Result<Self, ReductionRunStop<E>>
    where
        F: Fn(&T, &T) -> T,
        H: FnMut() -> Result<(), E>,
    {
        let mut nodes = Vec::with_capacity(values.len());
        let mut split_flags = Vec::with_capacity(values.len());
        for (identity, value) in identities.iter().copied().zip(values) {
            engine.visit()?;
            nodes.push(ShapeNode {
                identity,
                value,
                left: None,
                right: None,
            });
            split_flags.push(false);
        }
        let mut stack: Vec<usize> = Vec::with_capacity(nodes.len());
        for index in 0..nodes.len() {
            engine.visit()?;
            let mut popped = None;
            while let Some(&top) = stack.last() {
                engine.visit()?;
                if priority(nodes[index].identity) >= priority(nodes[top].identity) {
                    break;
                }
                popped = stack.pop();
            }
            if let Some(&parent) = stack.last() {
                nodes[parent].right = Some(index);
            }
            nodes[index].left = popped;
            stack.push(index);
        }
        let root = stack.first().copied();
        let mut shape = Self {
            nodes,
            root,
            split_order: Vec::with_capacity(identities.len()),
            split_flags,
        };
        if let Some(root) = shape.full_spec() {
            let mut queue = VecDeque::with_capacity(shape.nodes.len());
            queue.push_back(root);
            while let Some(spec) = queue.pop_front() {
                engine.visit()?;
                let children = shape.children_of(spec);
                if children.iter().any(Option::is_some) {
                    shape.split_order.push(spec);
                }
                for child in children.into_iter().flatten() {
                    queue.push_back(child);
                }
            }
        }
        Ok(shape)
    }

    /// Each shape node has an entry and completion visit; child aggregates are joined in the
    /// fixed left/value/right order. The traversal stack avoids depth limits.
    pub(super) fn evaluate<F, H, E>(
        &self,
        spec: SubtreeSpec,
        engine: &mut Engine<'_, T, F, H>,
    ) -> Result<(Link<T>, u64), ReductionRunStop<E>>
    where
        F: Fn(&T, &T) -> T,
        H: FnMut() -> Result<(), E>,
    {
        let mut built: Vec<Option<Built<T>>> =
            std::iter::repeat_with(|| None).take(spec.len).collect();
        let mut traversal = Vec::with_capacity(spec.len.saturating_mul(2));
        traversal.push((spec.root, false));
        while let Some((index, ready)) = traversal.pop() {
            let node = &self.nodes[index];
            if !ready {
                engine.visit()?;
                traversal.push((index, true));
                if let Some(right) = node.right {
                    traversal.push((right, false));
                }
                if let Some(left) = node.left {
                    traversal.push((left, false));
                }
                continue;
            }
            engine.visit()?;
            let left = node.left.and_then(|child| built[child - spec.first].take());
            let right = node
                .right
                .and_then(|child| built[child - spec.first].take());
            let span = left
                .as_ref()
                .map_or(0, |item| item.span)
                .max(right.as_ref().map_or(0, |item| item.span))
                .checked_add(4)
                .ok_or(ReductionRunStop::WorkCounterOverflow)?;
            let root = engine.build(
                node.identity,
                node.value.clone(),
                left.map(|item| item.root),
                right.map(|item| item.root),
            )?;
            built[index - spec.first] = Some(Built { root, span });
        }
        let built = built[spec.root - spec.first]
            .take()
            .expect("root evaluated");
        Ok((Some(built.root), built.span))
    }
}
