//! Connectivity and Source bijection follow actual retained structure and Native deltas.
use super::*;
impl SourceTree {
    pub(super) fn admit(
        &self,
        root: Option<usize>,
        node_count: usize,
        source_count: usize,
        edits: &[Edit],
        control: &Control<'_>,
    ) -> Result<(), Stop> {
        if self.region_count != node_count
            || self.sources.len() != source_count
            || self.parent_count != node_count.saturating_sub(1)
            || root.is_none() != (node_count == 0)
        {
            return Err(control.refuse(Stop::SourceStructureIncomplete));
        }
        if let Some(root) = root {
            if self.parent(root, control)?.is_some() || self.region_at(root, control)?.is_none() {
                return Err(control.refuse(Stop::SourceStructureIncomplete));
            }
        }
        for edit in edits {
            if edit.region.is_none() {
                // Region retirement never erases this independent incoming posting.
                if self.parent(edit.ordinal, control)?.is_some() {
                    return Err(control.refuse(Stop::SourceStructureIncomplete));
                }
                continue;
            }
            for child in edit.region.unwrap().children.into_iter().flatten() {
                // Every new cycle contains a new edge. Retained edges already
                // belong to the exact qualified predecessor's acyclic graph.
                if edit
                    .before
                    .is_some_and(|before| before.children.contains(&Some(child)))
                {
                    continue;
                }
                let mut current = Some(edit.ordinal);
                let mut visited = 0;
                while let Some(parent) = current {
                    if parent == child || visited >= node_count {
                        return Err(control.refuse(Stop::SourceStructureCycle));
                    }
                    visited += 1;
                    current = self.parent(parent, control)?;
                }
            }
        }
        control.poll()
    }
    pub(super) fn admit_delta(
        &self,
        before: &SourceTree,
        changes: &[Change],
        execution: &ExecutionRequest<'_, '_>,
        control: &Control<'_>,
    ) -> Result<(), Stop> {
        let mut after = Map::default();
        for change in changes {
            if let Some(slot) = change.before {
                maps::lookup(control, before.sources.len(), size_of::<Slot>())?;
                if !before.sources.contains_key(&slot) {
                    return Err(control.refuse(Stop::SourceStructureIncomplete));
                }
            }
            if let Some(slot) = change.after {
                maps::insert(&mut after, &slot, &(), execution, control)?;
                maps::lookup(control, self.sources.len(), size_of::<Slot>())?;
                if !self.sources.contains_key(&slot) {
                    return Err(control.refuse(Stop::SourceStructureIncomplete));
                }
            }
        }
        for change in changes {
            if let Some(slot) = change.before {
                maps::lookup(control, after.len(), size_of::<Slot>())?;
                maps::lookup(control, self.sources.len(), size_of::<Slot>())?;
                if !after.contains_key(&slot) && self.sources.contains_key(&slot) {
                    return Err(control.refuse(Stop::SourceStructureIncomplete));
                }
            }
        }
        Ok(())
    }
}
