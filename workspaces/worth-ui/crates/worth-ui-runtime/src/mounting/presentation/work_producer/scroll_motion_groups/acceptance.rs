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
    displayed: Option<UiDisplayedRect>,
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
            displayed: None,
        })
    }

    pub(in crate::mounting::presentation::work_producer) fn validate(
        &mut self,
        state: &UiMountedPresentationState,
        displayed: UiDisplayedSurfaceBasis,
    ) -> Result<(), Denial> {
        let group = state
            .scroll_motion_groups
            .groups
            .get(&self.target)
            .ok_or(Denial::CommandReplaced)?;
        if !Rc::ptr_eq(&group.displayed_sample, &self.slot) {
            return Err(Denial::CommandReplaced);
        }
        self.sample = self
            .sample
            .with_presentation_basis(displayed.basis())
            .map_err(|_| Denial::SampleBasis)?;
        let geometry = self.sample.geometry().ok_or(Denial::SampleBasis)?;
        self.displayed =
            Some(UiDisplayedRect::displayed(geometry, displayed).map_err(|_| Denial::SampleBasis)?);
        Ok(())
    }

    pub(in crate::mounting::presentation::work_producer) fn commit(self) {
        self.slot.set(self.displayed);
    }
}
