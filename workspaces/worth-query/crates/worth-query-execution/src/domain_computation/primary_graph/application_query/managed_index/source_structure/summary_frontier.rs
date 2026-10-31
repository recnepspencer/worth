//! Only actual changed support and its complete final ancestor closure is requalified.
use super::super::WorthQueryManagedIndexWorkDenial as Denial;
use super::*;

impl SourceTree {
    pub(super) fn affected(
        &self,
        before: &SourceTree,
        changes: &[Change],
        edits: &[Edit],
        mut affected: Map<usize, ()>,
        execution: &ExecutionRequest<'_, '_>,
        control: &Control<'_>,
    ) -> Result<Map<usize, ()>, Stop> {
        // Every live seed schedules its own final parent. Another seed may
        // stop this walk before its turn because that turn still closes its
        // parent chain; seed membership alone is not a completed closure.
        for edit in edits {
            if edit.region.is_some() {
                if let Some(parent) = self.parent(edit.ordinal, control)? {
                    self.add_ancestors(parent, &mut affected, execution, control)?;
                }
            }
        }
        for change in changes {
            for (tree, slot) in [(before, change.before), (self, change.after)] {
                if let Some(slot) = slot {
                    maps::lookup(control, tree.sources.len(), size_of::<Slot>())?;
                    if let Some(node) = tree.sources.get(&slot).copied() {
                        self.add_ancestors(node, &mut affected, execution, control)?;
                    }
                }
            }
        }
        Ok(affected)
    }
    fn add_ancestors(
        &self,
        ordinal: usize,
        affected: &mut Map<usize, ()>,
        execution: &ExecutionRequest<'_, '_>,
        control: &Control<'_>,
    ) -> Result<(), Stop> {
        if self.region_at(ordinal, control)?.is_none() {
            return Ok(());
        }
        let mut current = Some(ordinal);
        while let Some(node) = current {
            maps::lookup(control, affected.len(), size_of::<usize>())?;
            if affected.contains_key(&node) {
                break;
            }
            maps::insert(affected, &node, &(), execution, control)?;
            current = self.parent(node, control)?;
        }
        Ok(())
    }
}

pub(super) fn postorder(
    tree: &SourceTree,
    affected: Option<&Map<usize, ()>>,
    control: &Control<'_>,
    qualify: &mut dyn FnMut(usize) -> Result<(), Denial>,
) -> Result<(), Denial> {
    let (mut previous, mut current) = (None, tree.root());
    while let Some(ordinal) = current {
        control.checkpoint(1)?;
        let parent = tree.parent(ordinal, control)?;
        let region = tree.region(ordinal, control)?;
        let [left, right] = region.children;
        let next = if previous == parent {
            let skip = if let Some(affected) = affected {
                maps::lookup(control, affected.len(), size_of::<usize>())?;
                !affected.contains_key(&ordinal)
            } else {
                false
            };
            if skip {
                parent
            } else if let Some(child) = left.or(right) {
                Some(child)
            } else {
                qualify(ordinal)?;
                parent
            }
        } else if previous == left && right.is_some() {
            right
        } else if previous == left || previous == right {
            qualify(ordinal)?;
            parent
        } else {
            return Err(control.refuse(Stop::IncompleteTraversal).into());
        };
        previous = current;
        current = next;
    }
    control.poll().map_err(Into::into)
}
