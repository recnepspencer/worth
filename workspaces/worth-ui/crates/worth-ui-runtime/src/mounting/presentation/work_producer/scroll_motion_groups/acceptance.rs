//! Physical group evidence commits atomically with the commands it moved.
use super::super::motion_evidence::UiCommandMotionAcceptanceDenial as Denial;
use super::*;
use crate::mounting::presentation::motion_sampling::UiPresentationMotionSampleReceipt;
use std::{cell::Cell, rc::Rc};

pub(in crate::mounting::presentation::work_producer) struct UiScrollGroupMotionUpdate {
    target: UiMotionTargetIdentity,
    slot: Rc<Cell<Option<UiPresentationMotionSampleReceipt>>>,
    sample: UiPresentationMotionSampleReceipt,
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
                .accepted
                .clone(),
        })
    }

    pub(in crate::mounting::presentation::work_producer) fn validate(
        &mut self,
        state: &UiMountedPresentationState,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<(), Denial> {
        let group = state
            .scroll_motion_groups
            .groups
            .get(&self.target)
            .ok_or(Denial::CommandReplaced)?;
        if !Rc::ptr_eq(&group.accepted, &self.slot) {
            return Err(Denial::CommandReplaced);
        }
        self.sample = self
            .sample
            .with_presentation_basis(presentation)
            .map_err(|_| Denial::SampleBasis)?;
        Ok(())
    }

    pub(in crate::mounting::presentation::work_producer) fn commit(self) {
        self.slot.set(Some(self.sample));
    }
}
