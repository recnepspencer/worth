//! Isolated ordinal columns remove lost postings before adding any replacement.
use super::*;
impl SourceTree {
    pub(super) fn apply_edits(
        self,
        edits: &[Edit],
        image: SourceImageView<'_>,
        execution: &ExecutionRequest<'_, '_>,
        control: &Control<'_>,
    ) -> Result<Self, Stop> {
        let mut regions = PageEdit::new(self.regions, *execution);
        let mut parents = PageEdit::new(self.parents, *execution);
        let (mut region_count, mut parent_count) = (self.region_count, self.parent_count);
        let mut sources = self.sources;
        for edit in edits {
            let Some(old) = edit.before else {
                continue;
            };
            for child in old.children.into_iter().flatten() {
                if !edit
                    .region
                    .is_some_and(|region| region.children.contains(&Some(child)))
                {
                    write(&mut parents, &mut parent_count, child, None, control)?;
                }
            }
            if let Some(slot) = old.source {
                if edit.region.and_then(|region| region.source) != Some(slot) {
                    maps::remove(&mut sources, &slot, execution, control)?;
                }
            }
            if edit.region.is_none() {
                // Keep parents[ordinal] independent for the final retired-edge check.
                write(&mut regions, &mut region_count, edit.ordinal, None, control)?;
            }
        }
        for edit in edits {
            let Some(region) = edit.region else {
                continue;
            };
            if region.children[0].is_some() && region.children[0] == region.children[1] {
                return Err(control.refuse(Stop::DuplicateNodeParent));
            }
            for child in region.children.into_iter().flatten() {
                if edit
                    .before
                    .is_some_and(|before| before.children.contains(&Some(child)))
                {
                    continue;
                }
                if parents.get(child, control)?.is_some() {
                    return Err(control.refuse(Stop::DuplicateNodeParent));
                }
                write(
                    &mut parents,
                    &mut parent_count,
                    child,
                    Some(edit.ordinal),
                    control,
                )?;
            }
            if let Some(slot) = region.source {
                maps::lookup(control, image.len(), size_of::<Slot>())?;
                image.validate_slot(slot, control)?;
                if edit.before.and_then(|before| before.source) != Some(slot) {
                    maps::lookup(control, sources.len(), size_of::<Slot>())?;
                    if sources.contains_key(&slot) {
                        return Err(control.refuse(Stop::DuplicateSourceSlot));
                    }
                    maps::insert(&mut sources, &slot, &edit.ordinal, execution, control)?;
                }
            }
            if edit.before != Some(region) {
                write(
                    &mut regions,
                    &mut region_count,
                    edit.ordinal,
                    Some(region),
                    control,
                )?;
            }
        }
        // A refusal in either consuming seal drops both candidates; capture
        // admits the complete final pair before any qualified root can escape.
        let regions = regions.seal(control)?;
        let parents = parents.seal(control)?;
        Ok(Self {
            root: self.root,
            owner: self.owner,
            regions,
            parents,
            sources,
            region_count,
            parent_count,
        })
    }
}
fn write<T: Copy>(
    pages: &mut PageEdit<'_, T>,
    count: &mut usize,
    ordinal: usize,
    value: Option<T>,
    control: &Control<'_>,
) -> Result<(), Stop> {
    control.checkpoint(1)?;
    let occupied = pages.get(ordinal, control)?.is_some();
    let next = match (occupied, value.is_some()) {
        (false, true) => count
            .checked_add(1)
            .ok_or_else(|| control.refuse(Stop::CapacityOverflow))?,
        (true, false) => count
            .checked_sub(1)
            .ok_or_else(|| control.refuse(Stop::SourceStructureIncomplete))?,
        _ => *count,
    };
    pages.replace(ordinal, value, control)?;
    *count = next;
    Ok(())
}
