//! The live slot holding what the host displays one command through.
//!
//! A frame can be issued while a tick is still in flight on its predecessor.
//! The host lands the tick first, so by the time the frame shows, each command
//! the tick moves stands where the tick put it, and the frame states that
//! rather than what its predecessor showed before the tick. It restates each
//! such command at the tick's layer whether or not the tick lands: a tick that
//! never lands leaves its Motion running, and the next tick moves each command
//! again.
//!
//! A command a Scroll group moves is left sharing its slot. The group's bound
//! standing and each displayed base bind together at what the host showed
//! before the tick, and a tick's translation composes from both, so holding
//! the tick's layer on the command alone would move it twice. The tick's
//! acceptance reaches the group and its commands through the slots they
//! share, and whatever next moves or places the group restates the command
//! where it stands.

use std::{cell::Cell, rc::Rc};
use worth_ui_host_contract::{UiMountedPaintCommandIdentity, UiMountedPresentationSampleChange};

use super::super::motion_sampling::UiPresentationMotionSampleReceipt;
use super::command_motion_layers::UiCommandMotionLayers;
use super::UiMountedPresentationState;

/// Live physical evidence, shared only by versions of one unchanged command.
/// It is never exposed as an immutable historical frame snapshot.
#[derive(Clone, Default)]
pub(super) struct UiCommandMotionAcceptance(Rc<Cell<Option<UiDisplayedCommandMotion>>>);

/// The sample and change an admitted witness displayed for one command, and
/// the Motion layers that change composes. Only acceptance at a witness's
/// displayed basis writes it, or a successor frame stating what the host
/// displays by the time it shows: what the host already displays, carried
/// into a command it replaces, or what a tick issued ahead of it displays.
#[derive(Clone, Copy)]
pub(super) struct UiDisplayedCommandMotion {
    pub(super) sample: UiPresentationMotionSampleReceipt,
    pub(super) change: UiMountedPresentationSampleChange,
    pub(super) layers: UiCommandMotionLayers,
}

/// What a tick still in flight displays each command it moves once it lands,
/// with the slot it was prepared against: nothing when no tick is in flight
/// ahead of a frame.
#[derive(Clone, Default)]
pub(in crate::mounting::presentation) struct UiIssuedCommandMotion(
    Vec<(
        UiMountedPaintCommandIdentity,
        UiCommandMotionAcceptance,
        UiDisplayedCommandMotion,
    )>,
);

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

impl UiIssuedCommandMotion {
    pub(super) fn landing(
        displayed: impl IntoIterator<
            Item = (
                UiMountedPaintCommandIdentity,
                UiCommandMotionAcceptance,
                UiDisplayedCommandMotion,
            ),
        >,
    ) -> Self {
        Self(displayed.into_iter().collect())
    }

    pub(super) fn displayed(&self) -> impl Iterator<Item = UiDisplayedCommandMotion> + '_ {
        self.0.iter().map(|(_, _, displayed)| *displayed)
    }
}

impl UiMountedPresentationState {
    /// Give each command this successor shares with the predecessor the
    /// `issued` tick was prepared against, and no Scroll group moves, a live
    /// slot of its own holding what the tick displays it at, and tell the host.
    pub(in crate::mounting::presentation) fn hold_issued_motion(
        &mut self,
        issued: &UiIssuedCommandMotion,
    ) {
        for (command, prepared, displayed) in &issued.0 {
            if displayed.layers.scroll_layer().is_some()
                || !self
                    .motion_slot(*command)
                    .is_some_and(|shared| shared.is(prepared))
            {
                continue;
            }
            let Some(own) = self.own_motion_slot(*command) else {
                continue;
            };
            own.display(*displayed);
            if !self.carried_motion.contains(command) {
                self.carried_motion.push(*command);
            }
        }
    }

    /// Give `command` a live slot its predecessors do not share.
    pub(super) fn own_motion_slot(
        &mut self,
        command: UiMountedPaintCommandIdentity,
    ) -> Option<UiCommandMotionAcceptance> {
        if let Some(identity) = command.scroll_chrome_identity() {
            return self.scroll_motion_groups.own_motion_slot(identity);
        }
        if command.is_appearance_surface() {
            return self.own_appearance_surface_slot(command.mounted_instance());
        }
        let instance = command.mounted_instance();
        let mut bundle = self.commands_by_instance.get(&instance)?.clone();
        let motion = bundle.own_motion_slot(command)?;
        self.commands_by_instance.insert(instance, bundle);
        Some(motion)
    }
}
