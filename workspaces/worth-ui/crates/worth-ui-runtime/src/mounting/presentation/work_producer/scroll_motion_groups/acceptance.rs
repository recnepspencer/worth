//! Physical group evidence commits atomically with the commands it moved, and
//! only as what the admitted witness displayed.
use super::super::motion_evidence::UiCommandMotionAcceptanceDenial as Denial;
use super::*;
use crate::mounting::presentation::motion_sampling::UiPresentationMotionSampleReceipt;
use crate::mounting::presentation::{UiDisplayedRect, UiDisplayedSurfaceBasis};
use std::{cell::Cell, rc::Rc};

pub(in crate::mounting::presentation::work_producer) struct UiScrollGroupMotionUpdate {
    target: UiMotionTargetIdentity,
    slot: Rc<Cell<Option<UiDisplayedRect>>>,
    sample: UiPresentationMotionSampleReceipt,
}

/// A group update the admitted witness displayed, ready to commit.
pub(in crate::mounting::presentation::work_producer) struct UiDisplayedScrollGroupMotion {
    slot: Rc<Cell<Option<UiDisplayedRect>>>,
    displayed: UiDisplayedRect,
}

impl UiScrollGroupMotionUpdate {
    pub(in crate::mounting::presentation::work_producer) fn prepare(
        state: &UiMountedPresentationState,
        sample: UiPresentationMotionSampleReceipt,
    ) -> Option<Self> {
        Some(Self {
            target: sample.target(),
            sample,
            slot: state
                .scroll_motion_groups
                .groups
                .get(&sample.target())?
                .displayed_sample
                .clone(),
        })
    }

    /// What `displayed` shows of this update, if the group it was prepared
    /// for is still the one `state` holds.
    pub(in crate::mounting::presentation::work_producer) fn validate(
        &self,
        state: &UiMountedPresentationState,
        displayed: UiDisplayedSurfaceBasis,
    ) -> Result<UiDisplayedScrollGroupMotion, Denial> {
        let group = state
            .scroll_motion_groups
            .groups
            .get(&self.target)
            .ok_or(Denial::CommandReplaced)?;
        if !Rc::ptr_eq(&group.displayed_sample, &self.slot) {
            return Err(Denial::CommandReplaced);
        }
        let sample = self
            .sample
            .with_presentation_basis(displayed.basis())
            .map_err(|_| Denial::SampleBasis)?;
        let geometry = sample.geometry().ok_or(Denial::SampleBasis)?;
        Ok(UiDisplayedScrollGroupMotion {
            slot: self.slot.clone(),
            displayed: UiDisplayedRect::displayed(geometry, displayed)
                .map_err(|_| Denial::SampleBasis)?,
        })
    }
}

impl UiDisplayedScrollGroupMotion {
    pub(in crate::mounting::presentation::work_producer) fn commit(self) {
        self.slot.set(Some(self.displayed));
    }
}
