//! Admission-only structural ceiling for the retained Scroll sample index.
//! Shared allocations may be charged more than once across groups/frames;
//! no sample tick recounts membership or performs reservation work.
use super::*;
use crate::mounting::presentation::motion_sampling::UiPresentationMotionSampleReceipt;
use std::mem::size_of;

fn arc_slice_bytes<T>(len: usize) -> Option<usize> {
    len.checked_mul(size_of::<T>())?
        .checked_add(2 * size_of::<usize>())
}

impl UiMountedPresentationState {
    pub(in crate::mounting) fn indexed_motion_reserved_bytes(&self) -> Option<usize> {
        let groups = &self.scroll_motion_groups;
        let appearance_slots = self.bound_appearance_surface_instances().count();
        let additional_slots = appearance_slots.checked_add(groups.chrome.len())?;
        let mut bytes =
            super::super::motion_evidence::motion_acceptance_reserved_bytes(additional_slots)?
                .checked_add(groups.geometry_index_reserved_bytes)?;
        if groups.groups.is_empty() && groups.chrome.is_empty() {
            return Some(bytes);
        }
        bytes = bytes
            .checked_add(size_of::<UiMountedScrollMotionGroups>())?
            .checked_add(size_of::<
                BTreeMap<UiMountedInstanceIdentity, UiMotionTargetIdentity>,
            >())?
            .checked_add(arc_slice_bytes::<(
                UiMountedInstanceIdentity,
                UiMotionTargetIdentity,
            )>(groups.owners.len())?)?
            .checked_add(size_of::<
                std::collections::HashMap<
                    UiMountedPaintCommandIdentity,
                    Arc<[UiMotionTargetIdentity]>,
                >,
            >())?
            .checked_add(arc_slice_bytes::<(
                UiMountedPaintCommandIdentity,
                Arc<[UiMotionTargetIdentity]>,
            )>(groups.memberships.capacity())?)?
            .checked_add(
                groups
                    .memberships
                    .capacity()
                    .checked_mul(2 * size_of::<usize>())?,
            )?
            .checked_add(size_of::<
                BTreeMap<UiMotionTargetIdentity, UiMountedScrollMotionGroup>,
            >())?
            .checked_add(size_of::<
                BTreeMap<UiMountedScrollChromeIdentity, UiMountedScrollChromeSampleTarget>,
            >())?
            .checked_add(arc_slice_bytes::<(
                UiMotionTargetIdentity,
                UiMountedScrollMotionGroup,
            )>(groups.groups.len())?)?
            .checked_add(arc_slice_bytes::<(
                UiMountedScrollChromeIdentity,
                UiMountedScrollChromeSampleTarget,
            )>(groups.chrome.len())?)?;
        for group in groups.groups.values() {
            bytes = bytes
                .checked_add(size_of::<Option<UiPresentationMotionSampleReceipt>>())?
                .checked_add(2 * size_of::<usize>())?
                .checked_add(size_of::<super::acceptance::UiScrollGroupMotionUpdate>())?
                .checked_add(
                    super::super::motion_evidence::motion_acceptance_reserved_bytes(
                        group.commands.len(),
                    )?,
                )?
                .checked_add(arc_slice_bytes::<UiMountedScrollMotionMember>(
                    group.input.members.len(),
                )?)?
                .checked_add(arc_slice_bytes::<UiMountedScrollMotionCommand>(
                    group.commands.len(),
                )?)?
                .checked_add(arc_slice_bytes::<UiMountedScrollChromeIdentity>(
                    group.thumbs.len(),
                )?)?;
            for member in group.input.members.iter() {
                bytes = bytes.checked_add(arc_slice_bytes::<UiMountedScrollMotionClip>(
                    member.clips.len(),
                )?)?;
            }
            // Projected command clips are separately allocated after coordinate
            // conversion; retain both them and the prepared input provenance.
            for command in group.commands.iter() {
                bytes = bytes.checked_add(arc_slice_bytes::<UiMountedScrollMotionClip>(
                    command.clips.len(),
                )?)?;
            }
        }
        for memberships in groups.memberships.values() {
            bytes = bytes.checked_add(arc_slice_bytes::<UiMotionTargetIdentity>(
                memberships.len(),
            )?)?;
        }
        Some(bytes)
    }
}

#[cfg(test)]
#[path = "retention_tests.rs"]
mod tests;
