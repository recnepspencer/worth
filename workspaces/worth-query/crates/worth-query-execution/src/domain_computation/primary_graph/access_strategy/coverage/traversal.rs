//! Query discharges the actual checked root through visits or qualified subtree summaries.
use crate::domain_computation::primary_graph::application_query::managed_index::{
    SourceTree, WorthQueryManagedIndexControl as Control, WorthQueryManagedIndexSourceSlot as Slot,
    WorthQueryManagedIndexStop as Stop, WorthQueryManagedIndexWorkDenial as Denial,
};

pub(in crate::domain_computation::primary_graph) fn visit(
    tree: &SourceTree,
    root: Option<usize>,
    control: &Control<'_>,
    apply: &mut dyn FnMut(usize, Option<Slot>) -> Result<bool, Denial>,
) -> Result<(), Denial> {
    if root != tree.root() {
        return Err(control.refuse(Stop::IncompleteTraversal).into());
    }
    let (mut previous, mut current) = (None, root);
    let (mut entered, mut sources, mut steps) = (0usize, 0usize, 0usize);
    let mut pruned = false;
    let maximum_steps = tree
        .len()
        .checked_mul(3)
        .ok_or_else(|| control.refuse(Stop::CapacityOverflow))?;
    while let Some(ordinal) = current {
        control.checkpoint(1)?;
        if steps >= maximum_steps {
            return Err(control.refuse(Stop::IncompleteTraversal).into());
        }
        steps += 1;
        let region = tree.region(ordinal, control)?;
        let parent = tree.parent(ordinal, control)?;
        let [left, right] = region.children;
        let next = if previous == parent {
            entered += 1;
            if apply(ordinal, region.source)? {
                sources += usize::from(region.source.is_some());
                left.or(right).or(parent)
            } else {
                pruned = true;
                parent
            }
        } else if previous == left {
            right.or(parent)
        } else if previous == right {
            parent
        } else {
            return Err(control.refuse(Stop::IncompleteTraversal).into());
        };
        previous = current;
        current = next;
    }
    if !pruned && (entered != tree.len() || sources != tree.source_len()) {
        return Err(control.refuse(Stop::IncompleteTraversal).into());
    }
    control.poll().map_err(Into::into)
}
