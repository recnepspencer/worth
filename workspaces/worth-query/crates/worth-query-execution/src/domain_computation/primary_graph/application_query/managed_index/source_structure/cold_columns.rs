//! A cold physical roster has no predecessor pages or warm edit directory.
use super::*;
use std::cell::Cell;
use worth_execution::{ExecutionArray, ExecutionRequestArrayBuilder};

impl SourceTree {
    pub(super) fn from_cold(
        edits: &[Edit],
        image: SourceImageView<'_>,
        execution: &ExecutionRequest<'_, '_>,
        control: &Control<'_>,
    ) -> Result<Self, Stop> {
        let regions = column(edits.len(), execution, control)?;
        let parents = column(edits.len(), execution, control)?;
        let (mut region_count, mut parent_count) = (0, 0);
        for (ordinal, edit) in edits.iter().enumerate() {
            control.checkpoint(1)?;
            if edit.ordinal != ordinal || edit.before.is_some() {
                return Err(control.refuse(Stop::SourceStructureIncomplete));
            }
            let Some(region) = edit.region else {
                continue;
            };
            if region.children[0].is_some() && region.children[0] == region.children[1] {
                return Err(control.refuse(Stop::DuplicateNodeParent));
            }
            for child in region.children.into_iter().flatten() {
                control.checkpoint(1)?;
                if !edits.get(child).is_some_and(|entry| entry.region.is_some()) {
                    return Err(control.refuse(Stop::SourceStructureIncomplete));
                }
                let parent = &parents.elements()[child];
                if parent.get().is_some() {
                    return Err(control.refuse(Stop::DuplicateNodeParent));
                }
                parent.set(Some(ordinal));
                parent_count += 1;
            }
            if let Some(slot) = region.source {
                maps::lookup(control, image.len(), size_of::<Slot>())?;
                image.validate_slot(slot, control)?;
            }
            regions.elements()[ordinal].set(Some(region));
            region_count += 1;
        }
        let sources = super::cold_postings::capture(edits, execution, control)?;
        let regions = NodePages::from_scratch(execution, regions.elements(), control)?;
        let parents = NodePages::from_scratch(execution, parents.elements(), control)?;
        control.poll()?;
        Ok(Self {
            root: None,
            owner: image.owner,
            regions,
            parents,
            region_count,
            parent_count,
            sources,
        })
    }
}

pub(super) fn column<T>(
    count: usize,
    execution: &ExecutionRequest<'_, '_>,
    control: &Control<'_>,
) -> Result<ExecutionArray<Cell<Option<T>>>, Stop> {
    let mut column = ExecutionRequestArrayBuilder::allocate(*execution, count)
        .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
    for _ in 0..count {
        control.checkpoint(1)?;
        column
            .push(Cell::new(None))
            .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
    }
    column
        .seal()
        .map_err(|cause| control.refuse(Stop::Storage(cause)))
}
