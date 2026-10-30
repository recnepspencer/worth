//! Isolated topology edits remove old associations before adding any replacement.
use super::*;
impl SourceTree {
    pub(super) fn remove_edits(
        &mut self,
        edits: &[Edit],
        execution: &ExecutionRequest<'_, '_>,
        control: &Control<'_>,
    ) -> Result<(), Stop> {
        for edit in edits {
            maps::lookup(control, self.regions.len(), size_of::<usize>())?;
            let Some(old) = self.regions.get(&edit.ordinal).copied() else {
                continue;
            };
            for child in old.children.into_iter().flatten() {
                maps::remove(&mut self.parents, &child, execution, control)?;
            }
            if let Some(slot) = old.source {
                maps::remove(&mut self.sources, &slot, execution, control)?;
            }
            maps::remove(&mut self.regions, &edit.ordinal, execution, control)?;
        }
        Ok(())
    }
    pub(super) fn add_edits(
        &mut self,
        edits: &[Edit],
        image: SourceImageView<'_>,
        execution: &ExecutionRequest<'_, '_>,
        control: &Control<'_>,
    ) -> Result<(), Stop> {
        for edit in edits {
            let Some(region) = edit.region else {
                continue;
            };
            for child in region.children.into_iter().flatten() {
                maps::lookup(control, self.parents.len(), size_of::<usize>())?;
                if self.parents.contains_key(&child) {
                    return Err(control.refuse(Stop::DuplicateNodeParent));
                }
                maps::insert(&mut self.parents, &child, &edit.ordinal, execution, control)?;
            }
            if let Some(slot) = region.source {
                maps::lookup(control, image.len(), size_of::<Slot>())?;
                image.validate_slot(slot, control)?;
                maps::lookup(control, self.sources.len(), size_of::<Slot>())?;
                if self.sources.contains_key(&slot) {
                    return Err(control.refuse(Stop::DuplicateSourceSlot));
                }
                maps::insert(&mut self.sources, &slot, &edit.ordinal, execution, control)?;
            }
            maps::insert(
                &mut self.regions,
                &edit.ordinal,
                &region,
                execution,
                control,
            )?;
        }
        Ok(())
    }
}
