//! The Motion layers one command's displayed change composes.
//!
//! A command can move under three Motions at once: its own, the Scroll
//! regions that carry it, and the Portal that presents it. The host shows one
//! change per command, so the change composes them, innermost first. A tick
//! samples some of them; every layer it does not sample holds where the host
//! shows it now, so moving one Motion never drops another.

use worth_ui_host_contract::{
    UiMountedAppearanceOpacity, UiMountedCanonicalBox, UiMountedPaintCommandIdentity,
    UiMountedPresentationSampleChange, UiMountedPresentationTransform,
};

use super::motion_sample::UiMountedMotionSampleWorkDenial as Denial;
use crate::mounting::presentation::compose_opacity;
use crate::mounting::presentation::truth_geometry::{carried_box, carried_transform};

/// Which Motion a layer carries, innermost first.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum UiCommandMotionLayerKind {
    /// The command's own target moving.
    Own,
    /// The Scroll regions that carry the command.
    Scroll,
    /// The Portal that presents the command.
    Portal,
}

/// One Motion's part of what the host shows for a command.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct UiCommandMotionLayer {
    transform: Option<UiMountedPresentationTransform>,
    /// Scroll clips what it moves, after moving it.
    clip: Option<UiMountedCanonicalBox>,
    motion_units: u16,
}

impl UiCommandMotionLayer {
    /// A layer that moves and fades what it carries.
    pub(super) const fn moved(
        transform: Option<UiMountedPresentationTransform>,
        motion_units: u16,
    ) -> Self {
        Self {
            transform,
            clip: None,
            motion_units,
        }
    }

    /// A Scroll layer: it moves what it carries and clips it where the moved
    /// content shows.
    pub(super) const fn scrolled(
        transform: UiMountedPresentationTransform,
        clip: UiMountedCanonicalBox,
        motion_units: u16,
    ) -> Self {
        Self {
            transform: Some(transform),
            clip: Some(clip),
            motion_units,
        }
    }

    /// Whether the layer neither moves nor fades what it carries; it may
    /// still clip it.
    pub(super) fn moves_nothing(self) -> bool {
        self.motion_units == u16::MAX
            && self
                .transform
                .is_none_or(|transform| transform.source() == transform.sampled())
    }
}

/// The layers the host shows one command through.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct UiCommandMotionLayers {
    own: Option<UiCommandMotionLayer>,
    scroll: Option<UiCommandMotionLayer>,
    portal: Option<UiCommandMotionLayer>,
}

impl UiCommandMotionLayers {
    fn slot(&mut self, kind: UiCommandMotionLayerKind) -> &mut Option<UiCommandMotionLayer> {
        match kind {
            UiCommandMotionLayerKind::Own => &mut self.own,
            UiCommandMotionLayerKind::Scroll => &mut self.scroll,
            UiCommandMotionLayerKind::Portal => &mut self.portal,
        }
    }

    /// Records the layer one tick sampled. Two Motions of one kind cannot
    /// both move one command in one tick, and only Scroll clips.
    pub(super) fn sample(
        &mut self,
        kind: UiCommandMotionLayerKind,
        layer: UiCommandMotionLayer,
    ) -> Result<(), Denial> {
        if kind != UiCommandMotionLayerKind::Scroll && layer.clip.is_some() {
            return Err(Denial::InvalidGeometry);
        }
        let slot = self.slot(kind);
        if slot.is_some() {
            return Err(Denial::AmbiguousTargetCommands);
        }
        *slot = Some(layer);
        Ok(())
    }

    /// The layers a tick leaves the host showing: those it sampled, over
    /// those the host already shows.
    pub(super) fn over(self, held: Self) -> Self {
        Self {
            own: self.own.or(held.own),
            scroll: self.scroll.or(held.scroll),
            portal: self.portal.or(held.portal),
        }
    }

    /// The Portal layer alone: what a command the Portal presents keeps
    /// showing through when a frame replaces it.
    pub(super) fn portal_only(self) -> Option<UiPortalMotionLayer> {
        self.portal.map(|portal| UiPortalMotionLayer {
            transform: portal.transform,
            motion_units: portal.motion_units,
        })
    }

    /// The Scroll layer, when the host shows the command's regions moving it.
    pub(super) const fn scroll_layer(self) -> Option<UiCommandMotionLayer> {
        self.scroll
    }

    /// These layers with `scroll` as the Scroll one.
    pub(super) const fn with_scroll(self, scroll: Option<UiCommandMotionLayer>) -> Self {
        Self { scroll, ..self }
    }

    /// The Scroll layer's transform: how far the host shows the command's
    /// regions moving it.
    pub(super) fn scroll_transform(self) -> Option<UiMountedPresentationTransform> {
        self.scroll.and_then(|layer| layer.transform)
    }

    /// Where the host shows the command's regions clipping it, before any
    /// Portal moves it.
    pub(super) fn scroll_clip(self) -> Option<UiMountedCanonicalBox> {
        self.scroll.and_then(|layer| layer.clip)
    }

    /// Where the layers outside `kind` show `bounds`, drawn in the space
    /// `kind`'s layer moves.
    pub(super) fn carried_outside(
        self,
        kind: UiCommandMotionLayerKind,
        bounds: UiMountedCanonicalBox,
    ) -> Result<UiMountedCanonicalBox, Denial> {
        let outside = match kind {
            UiCommandMotionLayerKind::Own => [self.scroll, self.portal],
            UiCommandMotionLayerKind::Scroll => [self.portal, None],
            UiCommandMotionLayerKind::Portal => [None, None],
        };
        outside
            .into_iter()
            .flatten()
            .filter_map(|layer| layer.transform)
            .try_fold(bounds, |bounds, outer| {
                carried_box(bounds, outer).ok_or(Denial::InvalidGeometry)
            })
    }

    /// The change that shows `command`, at resting opacity `resting`,
    /// through every layer.
    pub(super) fn change(
        self,
        command: UiMountedPaintCommandIdentity,
        resting: UiMountedAppearanceOpacity,
    ) -> Result<UiMountedPresentationSampleChange, Denial> {
        let mut transform: Option<UiMountedPresentationTransform> = None;
        let mut clip: Option<UiMountedCanonicalBox> = None;
        let mut motion_units = u16::MAX;
        for layer in [self.own, self.scroll, self.portal].into_iter().flatten() {
            if let Some(outer) = layer.transform {
                transform = Some(match transform {
                    Some(inner) => {
                        carried_transform(inner, outer).ok_or(Denial::InvalidGeometry)?
                    }
                    None => outer,
                });
                clip = clip
                    .map(|clip| carried_box(clip, outer).ok_or(Denial::InvalidGeometry))
                    .transpose()?;
            }
            clip = layer.clip.or(clip);
            motion_units = compose_opacity(
                UiMountedAppearanceOpacity::from_units(motion_units),
                layer.motion_units,
            )
            .units();
        }
        let opacity = compose_opacity(resting, motion_units);
        match clip {
            Some(clip) => UiMountedPresentationSampleChange::from_runtime_scroll_sampling(
                command,
                transform.ok_or(Denial::InvalidGeometry)?,
                opacity,
                clip,
            )
            .map_err(|_| Denial::InvalidGeometry),
            None => Ok(UiMountedPresentationSampleChange::from_runtime_sampling(
                command, transform, opacity,
            )),
        }
    }
}

#[cfg(test)]
#[path = "command_motion_layers_tests.rs"]
pub(super) mod tests;

/// A Portal layer alone. Only Scroll clips, so it moves and fades what it
/// carries, and composing it with nothing else cannot fail.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct UiPortalMotionLayer {
    transform: Option<UiMountedPresentationTransform>,
    motion_units: u16,
}

impl UiPortalMotionLayer {
    /// The layers the host shows a command through this layer alone.
    pub(super) fn layers(self) -> UiCommandMotionLayers {
        UiCommandMotionLayers {
            portal: Some(UiCommandMotionLayer::moved(
                self.transform,
                self.motion_units,
            )),
            ..UiCommandMotionLayers::default()
        }
    }

    /// The change that shows `command`, at resting opacity `resting`,
    /// through this layer alone.
    pub(super) fn change(
        self,
        command: UiMountedPaintCommandIdentity,
        resting: UiMountedAppearanceOpacity,
    ) -> UiMountedPresentationSampleChange {
        UiMountedPresentationSampleChange::from_runtime_sampling(
            command,
            self.transform,
            compose_opacity(resting, self.motion_units),
        )
    }
}
