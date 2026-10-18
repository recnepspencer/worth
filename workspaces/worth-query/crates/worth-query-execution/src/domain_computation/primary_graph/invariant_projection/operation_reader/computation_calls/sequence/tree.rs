//! The existing measured-call tree, with relative counts and funded AVL joins.
use super::*;
pub(super) fn build(
    start: usize,
    entries: &[(ComputationRead, Call)],
    execution: &QueryRequestExecution<'_>,
) -> Result<Option<Arc<Node>>, Resource> {
    assert_eq!(start, 0);
    if entries.is_empty() {
        return Ok(None);
    }
    if entries.len() == 1 {
        return fresh(entries[0].0, &entries[0].1, execution).map(Some);
    }
    let split = entries.len() / 2;
    branch(
        build(0, &entries[..split], execution)?.unwrap(),
        build(0, &entries[split..], execution)?.unwrap(),
        execution,
    )
    .map(Some)
}
pub(super) fn fresh(
    read: ComputationRead,
    call: &Call,
    execution: &QueryRequestExecution<'_>,
) -> Result<Arc<Node>, Resource> {
    let held = (size_of::<Node>() + 2 * size_of::<usize>()) as u64;
    let bytes = held
        .checked_add(
            call.0
                .additional_bytes()
                .ok_or(Resource::CapacityOverflow)?,
        )
        .ok_or(Resource::CapacityOverflow)?;
    let memory = execution.reserve(bytes)?;
    observed::computation_call_copy();
    Ok(Arc::new(Node {
        count: 1,
        height: 1,
        summary: Some(CallSummary::leaf(&call.0, call.1)),
        kind: Kind::Leaf(read, call.clone()),
        bytes,
        _memory: memory,
    }))
}
fn branch(
    a: Arc<Node>,
    b: Arc<Node>,
    execution: &QueryRequestExecution<'_>,
) -> Result<Arc<Node>, Resource> {
    let count = a
        .count
        .checked_add(b.count)
        .ok_or(Resource::CapacityOverflow)?;
    let height = a
        .height
        .max(b.height)
        .checked_add(1)
        .ok_or(Resource::CapacityOverflow)?;
    let summary = a.summary.zip(b.summary).and_then(|(a, b)| a.then(b));
    let held = (size_of::<Node>() + 2 * size_of::<usize>()) as u64;
    let bytes = held
        .checked_add(a.bytes)
        .and_then(|v| v.checked_add(b.bytes))
        .ok_or(Resource::CapacityOverflow)?;
    let memory = execution.reserve(held)?;
    observed::computation_call_copy();
    Ok(Arc::new(Node {
        count,
        height,
        summary,
        kind: Kind::Branch(a, b),
        bytes,
        _memory: memory,
    }))
}
fn balance(
    a: Arc<Node>,
    b: Arc<Node>,
    execution: &QueryRequestExecution<'_>,
) -> Result<Arc<Node>, Resource> {
    if a.height > b.height.saturating_add(1) {
        let Kind::Branch(x, y) = &a.kind else {
            unreachable!("heavy branch")
        };
        if x.height >= y.height {
            return branch(
                Arc::clone(x),
                branch(Arc::clone(y), b, execution)?,
                execution,
            );
        }
        let Kind::Branch(u, v) = &y.kind else {
            unreachable!("inner branch")
        };
        return branch(
            branch(Arc::clone(x), Arc::clone(u), execution)?,
            branch(Arc::clone(v), b, execution)?,
            execution,
        );
    }
    if b.height > a.height.saturating_add(1) {
        let Kind::Branch(x, y) = &b.kind else {
            unreachable!("heavy branch")
        };
        if y.height >= x.height {
            return branch(
                branch(a, Arc::clone(x), execution)?,
                Arc::clone(y),
                execution,
            );
        }
        let Kind::Branch(u, v) = &x.kind else {
            unreachable!("inner branch")
        };
        return branch(
            branch(a, Arc::clone(u), execution)?,
            branch(Arc::clone(v), Arc::clone(y), execution)?,
            execution,
        );
    }
    branch(a, b, execution)
}
fn join(
    a: Arc<Node>,
    b: Arc<Node>,
    execution: &QueryRequestExecution<'_>,
) -> Result<Arc<Node>, Resource> {
    if a.height > b.height.saturating_add(1) {
        let Kind::Branch(x, y) = &a.kind else {
            unreachable!("heavy branch")
        };
        return balance(Arc::clone(x), join(Arc::clone(y), b, execution)?, execution);
    }
    if b.height > a.height.saturating_add(1) {
        let Kind::Branch(x, y) = &b.kind else {
            unreachable!("heavy branch")
        };
        return balance(join(a, Arc::clone(x), execution)?, Arc::clone(y), execution);
    }
    branch(a, b, execution)
}
pub(super) fn leaf(node: &Node, ordinal: usize) -> &Call {
    match &node.kind {
        Kind::Leaf(_, call) => {
            assert_eq!(ordinal, 0);
            call
        }
        Kind::Branch(a, b) => {
            if ordinal < a.count {
                leaf(a, ordinal)
            } else {
                leaf(b, ordinal - a.count)
            }
        }
    }
}
pub(super) fn patch(
    node: &Arc<Node>,
    ordinal: usize,
    call: &Call,
    execution: &QueryRequestExecution<'_>,
) -> Result<Arc<Node>, Resource> {
    match &node.kind {
        Kind::Leaf(read, _) => fresh(*read, call, execution),
        Kind::Branch(a, b) => {
            if ordinal < a.count {
                branch(
                    patch(a, ordinal, call, execution)?,
                    Arc::clone(b),
                    execution,
                )
            } else {
                branch(
                    Arc::clone(a),
                    patch(b, ordinal - a.count, call, execution)?,
                    execution,
                )
            }
        }
    }
}
pub(super) fn insert(
    node: Option<&Arc<Node>>,
    ordinal: usize,
    read: ComputationRead,
    call: &Call,
    execution: &QueryRequestExecution<'_>,
) -> Result<Arc<Node>, Resource> {
    let Some(node) = node else {
        assert_eq!(ordinal, 0);
        return fresh(read, call, execution);
    };
    match &node.kind {
        Kind::Leaf(..) => {
            let new = fresh(read, call, execution)?;
            if ordinal == 0 {
                join(new, Arc::clone(node), execution)
            } else {
                assert_eq!(ordinal, 1);
                join(Arc::clone(node), new, execution)
            }
        }
        Kind::Branch(a, b) => {
            if ordinal <= a.count {
                join(
                    insert(Some(a), ordinal, read, call, execution)?,
                    Arc::clone(b),
                    execution,
                )
            } else {
                join(
                    Arc::clone(a),
                    insert(Some(b), ordinal - a.count, read, call, execution)?,
                    execution,
                )
            }
        }
    }
}
pub(super) fn remove(
    node: &Arc<Node>,
    ordinal: usize,
    execution: &QueryRequestExecution<'_>,
) -> Result<Option<Arc<Node>>, Resource> {
    match &node.kind {
        Kind::Leaf(..) => {
            assert_eq!(ordinal, 0);
            Ok(None)
        }
        Kind::Branch(a, b) => {
            if ordinal < a.count {
                match remove(a, ordinal, execution)? {
                    Some(a) => join(a, Arc::clone(b), execution).map(Some),
                    None => Ok(Some(Arc::clone(b))),
                }
            } else {
                match remove(b, ordinal - a.count, execution)? {
                    Some(b) => join(Arc::clone(a), b, execution).map(Some),
                    None => Ok(Some(Arc::clone(a))),
                }
            }
        }
    }
}
pub(super) fn summary(node: &Node, range: &Range<usize>) -> Option<CallSummary> {
    summary_at(node, 0, range)
}
fn summary_at(node: &Node, start: usize, range: &Range<usize>) -> Option<CallSummary> {
    observed::computation_call_range();
    let end = start.checked_add(node.count)?;
    if range.start <= start && end <= range.end {
        return node.summary;
    }
    match &node.kind {
        Kind::Leaf(..) => None,
        Kind::Branch(a, b) => {
            let split = start + a.count;
            if range.end <= split {
                summary_at(a, start, range)
            } else if range.start >= split {
                summary_at(b, split, range)
            } else {
                summary_at(a, start, range)?.then(summary_at(b, split, range)?)
            }
        }
    }
}
pub(super) fn visit<E>(
    node: &Node,
    range: &Range<usize>,
    marked: &BTreeSet<usize>,
    visitor: &mut impl FnMut(CallVisit<'_>) -> Result<bool, E>,
) -> Result<(), E> {
    visit_at(node, 0, range, marked, visitor)
}
fn visit_at<E>(
    node: &Node,
    start: usize,
    range: &Range<usize>,
    marked: &BTreeSet<usize>,
    visitor: &mut impl FnMut(CallVisit<'_>) -> Result<bool, E>,
) -> Result<(), E> {
    let end = start + node.count;
    if range.end <= start || end <= range.start {
        return Ok(());
    }
    observed::computation_call_range();
    if range.start <= start
        && end <= range.end
        && marked.range(start..end).next().is_none()
        && let Some(summary) = &node.summary
        && visitor(CallVisit::Range(start..end, summary))?
    {
        return Ok(());
    }
    match &node.kind {
        Kind::Leaf(read, (charge, declared)) => {
            visitor(CallVisit::Leaf(start, *read, charge, *declared))?;
        }
        Kind::Branch(a, b) => {
            visit_at(a, start, range, marked, visitor)?;
            visit_at(b, start + a.count, range, marked, visitor)?;
        }
    }
    Ok(())
}
