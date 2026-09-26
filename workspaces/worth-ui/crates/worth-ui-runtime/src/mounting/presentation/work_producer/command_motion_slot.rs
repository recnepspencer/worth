//! The live slot holding what the host displays one command through.

use std::{cell::Cell, rc::Rc};
use worth_ui_host_contract::UiMountedPresentationSampleChange;

use super::super::motion_sampling::UiPresentationMotionSampleReceipt;
use super::command_motion_layers::UiCommandMotionLayers;

/// Live physical evidence, shared only by versions of one unchanged command.
/// It is never exposed as an immutable historical frame snapshot.
#[derive(Clone, Default)]
pub(super) struct UiCommandMotionAcceptance(Rc<Cell<Option<UiDisplayedCommandMotion>>>);

/// The sample and change an admitted witness displayed for one command, and
/// the Motion layers that change composes. Only acceptance at a witness's
/// displayed basis writes it, or a successor frame carrying what the host
/// already displays into a command it replaces.
#[derive(Clone, Copy)]
pub(super) struct UiDisplayedCommandMotion {
    pub(super) sample: UiPresentationMotionSampleReceipt,
    pub(super) change: UiMountedPresentationSampleChange,
    pub(super) layers: UiCommandMotionLayers,
}

impl UiCommandMotionAcceptance {
    pub(super) fn sample(&self) -> Option<UiPresentationMotionSampleReceipt> {
        self.0.get().map(|accepted| accepted.sample)
    }

    /// The opacity every Motion showing the command scales it by, composed.
    pub(super) fn motion_units(&self) -> Option<u16> {
        self.0
            .get()
            .map(|accepted| accepted.change.opacity().motion_units())
    }

    pub(super) fn displayed(&self) -> Option<UiDisplayedCommandMotion> {
        self.0.get()
    }

    pub(super) fn display(&self, displayed: UiDisplayedCommandMotion) {
        self.0.set(Some(displayed));
    }

    /// Whether both name one live slot.
    pub(super) fn is(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
