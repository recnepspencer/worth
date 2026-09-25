//! What one Motion tick does to the commands it moves.
//!
//! A tick gathers every layer it samples for each command before it shows
//! any: a command two Motions move in one tick gets one change composing
//! both. Where the tick repaints can depend on that composition, so damage is
//! planned in sample order and resolved once every change is known.

use std::collections::HashMap;

use worth_ui_host_contract::{
    UiMountedCanonicalBox, UiMountedLogicalDamage, UiMountedPaintCommandIdentity,
    UiMountedPresentationSampleChange,
};

use super::command_motion_layers::{
    UiCommandMotionLayer, UiCommandMotionLayerKind, UiCommandMotionLayers,
};
use super::motion_sample::UiMountedMotionSampleWorkDenial as Denial;
use crate::mounting::presentation::motion_sampling::UiPresentationMotionSampleReceipt;
use crate::mounting::presentation::truth_geometry::carried_box;

/// The layers one tick samples for one command.
pub(super) struct UiCommandMotionTick {
    pub(super) command: UiMountedPaintCommandIdentity,
    pub(super) layers: UiCommandMotionLayers,
    /// The outermost sample the tick moves the command by: the one its
    /// accepted evidence records.
    sample: (UiCommandMotionLayerKind, UiPresentationMotionSampleReceipt),
}

impl UiCommandMotionTick {
    pub(super) const fn sample(&self) -> UiPresentationMotionSampleReceipt {
        self.sample.1
    }
}

/// Where a tick repaints, before every change it makes is known.
pub(super) enum UiMotionTickDamage {
    /// Exactly these, drawn where the host shows them.
    Shown(Vec<UiMountedCanonicalBox>),
    /// All the command's Scroll regions show of it.
    Scrolled(UiMountedPaintCommandIdentity),
    /// Where the command shows now.
    Faded(UiMountedPaintCommandIdentity, UiMountedCanonicalBox),
    /// These, drawn in the space the command's `kind` layer moves.
    Moved(
        UiMountedPaintCommandIdentity,
        UiCommandMotionLayerKind,
        Vec<UiMountedCanonicalBox>,
    ),
}

#[derive(Default)]
pub(super) struct UiCommandMotionTicks {
    ticks: Vec<UiCommandMotionTick>,
    index: HashMap<UiMountedPaintCommandIdentity, usize>,
    damage: Vec<UiMotionTickDamage>,
}

impl UiCommandMotionTicks {
    /// Records that `sample` moves `command` by the `kind` layer `layer`.
    pub(super) fn sample(
        &mut self,
        command: UiMountedPaintCommandIdentity,
        kind: UiCommandMotionLayerKind,
        layer: UiCommandMotionLayer,
        sample: UiPresentationMotionSampleReceipt,
    ) -> Result<(), Denial> {
        let at = *self.index.entry(command).or_insert_with(|| {
            self.ticks.push(UiCommandMotionTick {
                command,
                layers: UiCommandMotionLayers::default(),
                sample: (kind, sample),
            });
            self.ticks.len() - 1
        });
        let tick = &mut self.ticks[at];
        tick.layers.sample(kind, layer)?;
        if kind >= tick.sample.0 {
            tick.sample = (kind, sample);
        }
        Ok(())
    }

    pub(super) fn repaint(&mut self, damage: UiMotionTickDamage) {
        self.damage.push(damage);
    }

    pub(super) fn is_empty(&self) -> bool {
        self.ticks.is_empty()
    }

    /// The commands the tick moves, in the order it first moved them, and
    /// its planned damage.
    pub(super) fn into_parts(self) -> (Vec<UiCommandMotionTick>, Vec<UiMotionTickDamage>) {
        (self.ticks, self.damage)
    }
}

impl UiMotionTickDamage {
    /// This damage where the host shows it, given how the tick shows each
    /// command it moves.
    pub(super) fn resolve(
        self,
        shown: &HashMap<
            UiMountedPaintCommandIdentity,
            (UiCommandMotionLayers, UiMountedPresentationSampleChange),
        >,
        damage: &mut Vec<UiMountedLogicalDamage>,
    ) -> Result<(), Denial> {
        let shown_as = |command| shown.get(&command).ok_or(Denial::UnknownTargetCommands);
        let boxes = match self {
            Self::Shown(boxes) => boxes,
            Self::Scrolled(command) => shown_as(command)?.1.clip().into_iter().collect(),
            Self::Faded(command, visible) => {
                let change = shown_as(command)?.1;
                let visible = match change.transform() {
                    Some(transform) => {
                        carried_box(visible, transform).ok_or(Denial::InvalidGeometry)?
                    }
                    None => visible,
                };
                match change.clip() {
                    Some(clip) => visible.intersection(clip).into_iter().collect(),
                    None => vec![visible],
                }
            }
            Self::Moved(command, kind, boxes) => {
                let layers = shown_as(command)?.0;
                boxes
                    .into_iter()
                    .map(|bounds| layers.carried_outside(kind, bounds))
                    .collect::<Result<_, _>>()?
            }
        };
        damage.extend(
            boxes
                .into_iter()
                .map(UiMountedLogicalDamage::from_runtime_mounting),
        );
        Ok(())
    }
}

#[cfg(test)]
#[path = "command_motion_ticks_tests.rs"]
mod tests;
