//! Compose all affecting Scroll groups before issuing one change per command.
use super::super::motion_sample::UiMountedMotionSampleWorkDenial as Denial;
use super::group_offset::{
    UiAcceptedCommandTranslation, UiAcceptedGroupOffset, UiDisplayedCommandTranslation,
    UiDisplayedGroupOffset, UiDisplayedToAcceptedTranslation, UiGroupStanding,
    UiPublishedGroupOffset, UiPublishedToAcceptedTranslation,
};
use super::*;
use crate::mounting::presentation::motion_sampling::UiPresentationMotionSampleReceipt;
use crate::runtime::scroll::{
    chrome::{UiScrollChromeAxis, UiScrollChromeFacts},
    snap_to_device_grid, UiScrollBounds,
};
use worth_ui_host_contract::{
    UiMountedCanonicalBoxInput, UiMountedCoordinateSpace, UiMountedPresentationSampleChange,
    UiMountedPresentationTransform,
};

type ActiveSamples = BTreeMap<UiMountedInstanceIdentity, UiPresentationMotionSampleReceipt>;

impl UiMountedPresentationState {
    pub(in crate::mounting::presentation::work_producer) fn scroll_sample_changes(
        &self,
        samples: &[UiPresentationMotionSampleReceipt],
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<
        Vec<(
            UiMountedPresentationSampleChange,
            UiPresentationMotionSampleReceipt,
        )>,
        Denial,
    > {
        let mut active = ActiveSamples::new();
        let mut commands = Vec::new();
        let mut selected = std::collections::HashSet::new();
        for sample in samples.iter().filter(|sample| {
            sample.target().scope() == crate::runtime::motion::UiMotionTargetScope::ScrollContents
        }) {
            let group = self
                .scroll_motion_groups
                .groups
                .get(&sample.target())
                .ok_or(Denial::UnknownTargetCommands)?;
            if active.insert(group.input.owner, *sample).is_some() {
                return Err(Denial::AmbiguousTargetCommands);
            }
            for command in group.commands.iter() {
                if selected.insert(command.identity) {
                    commands.push((command, *sample));
                }
            }
        }
        let mut changes = Vec::with_capacity(commands.len());
        for (command, receipt) in commands {
            let identity = command.identity;
            let contributors = self
                .scroll_motion_groups
                .memberships
                .get(&identity)
                .ok_or(Denial::UnknownTargetCommands)?;
            let groups = contributors
                .iter()
                .map(|target| {
                    let group = self
                        .scroll_motion_groups
                        .groups
                        .get(target)
                        .ok_or(Denial::UnknownTargetCommands)?;
                    Ok((group, group.published_move(&active, &receipt)?))
                })
                .collect::<Result<Vec<_>, Denial>>()?;
            let mut clip = None;
            for part in command.clips.iter() {
                let delta = displacement_of(part.owner, &groups, presentation)?;
                add_clip(&mut clip, translate(part.bounds, delta.components())?)?;
            }
            for (group, _) in &groups {
                let delta = displacement_of(Some(group.input.owner), &groups, presentation)?;
                add_clip(
                    &mut clip,
                    translate(group.input.viewport, delta.components())?,
                )?;
            }
            let clip = clip.ok_or(Denial::InvalidGeometry)?;
            let (transform, opacity) = if let Some(chrome) = identity.scroll_chrome_identity() {
                let target = self
                    .scroll_motion_groups
                    .chrome
                    .get(&chrome)
                    .ok_or(Denial::UnknownTargetCommands)?;
                let mut placed = target.bounds;
                if chrome.part() == worth_ui_host_contract::UiMountedScrollChromePart::Thumb {
                    let owner_target = self
                        .scroll_motion_groups
                        .owners
                        .get(&chrome.owner_instance())
                        .ok_or(Denial::UnknownTargetCommands)?;
                    let group = self
                        .scroll_motion_groups
                        .groups
                        .get(owner_target)
                        .ok_or(Denial::UnknownTargetCommands)?;
                    placed = sampled_thumb(group, chrome.axis(), &active, &receipt)?;
                }
                placed = translate(
                    placed,
                    displacement_of(Some(chrome.owner_instance()), &groups, presentation)?
                        .components(),
                )?;
                (
                    UiMountedPresentationTransform::from_runtime_sampling(target.bounds, placed)
                        .map_err(|_| Denial::InvalidGeometry)?,
                    target.opacity,
                )
            } else {
                // A displayed base already shows every group where it stands,
                // so it moves from there; a published command moves from its
                // groups' published offsets, as its clips do.
                let translation = match command.base_translation {
                    Some(base) => UiAcceptedCommandTranslation::from_displayed(
                        presentation,
                        base,
                        groups
                            .iter()
                            .map(|(group, _)| group.standing_move(base, &active, &receipt))
                            .collect::<Result<Vec<_>, _>>()?,
                    )?,
                    None => UiAcceptedCommandTranslation::from_published(
                        presentation,
                        groups.iter().map(|(_, delta)| *delta),
                    )?,
                };
                let source = bounds([0.0, 0.0, 1.0, 1.0], UiMountedCoordinateSpace::Viewport)?;
                (
                    UiMountedPresentationTransform::from_runtime_sampling(
                        source,
                        translate(source, translation.components())?,
                    )
                    .map_err(|_| Denial::InvalidGeometry)?,
                    self.appearance_opacity_for_command(identity),
                )
            };
            let change = UiMountedPresentationSampleChange::from_runtime_scroll_sampling(
                identity,
                transform,
                crate::mounting::presentation::compose_opacity(opacity, receipt.opacity_units()),
                clip,
            )
            .map_err(|_| Denial::InvalidGeometry)?;
            changes.push((change, receipt));
        }
        Ok(changes)
    }
}

impl UiMountedScrollMotionGroup {
    /// Where the host's retained commands show this group: at its last
    /// displayed sample, or, before one, where it stood when it was bound.
    pub(super) fn standing(&self) -> UiGroupStanding {
        self.displayed_sample
            .get()
            .map_or(self.bound_standing.standing(), |sample| {
                UiGroupStanding::Displayed(UiDisplayedGroupOffset::of_sample(
                    self.input.content,
                    sample,
                ))
            })
    }

    /// Where the candidate `tick` accepts puts the group: at its active
    /// sample, or, when the tick does not move it, where it stands.
    fn candidate_offset(
        &self,
        active: &ActiveSamples,
        tick: &UiPresentationMotionSampleReceipt,
    ) -> Result<UiAcceptedGroupOffset, Denial> {
        match active.get(&self.input.owner) {
            Some(sample) => sample
                .geometry()
                .map(|geometry| UiAcceptedGroupOffset::of_sample(self.input.content, geometry))
                .ok_or(Denial::InvalidGeometry),
            None => Ok(UiAcceptedGroupOffset::held_by(tick, self.standing())),
        }
    }

    /// How far the candidate moves content at the group's published offset.
    fn published_move(
        &self,
        active: &ActiveSamples,
        tick: &UiPresentationMotionSampleReceipt,
    ) -> Result<UiPublishedToAcceptedTranslation, Denial> {
        Ok(UiPublishedGroupOffset::of(self.input.offset)
            .move_to(self.candidate_offset(active, tick)?, self.input.scale))
    }

    /// How far the candidate moves content where `base`, bound with this
    /// group, shows it.
    fn standing_move(
        &self,
        base: UiDisplayedCommandTranslation,
        active: &ActiveSamples,
        tick: &UiPresentationMotionSampleReceipt,
    ) -> Result<UiDisplayedToAcceptedTranslation, Denial> {
        Ok(self
            .bound_standing
            .shown_by(base)?
            .move_to(self.candidate_offset(active, tick)?, self.input.scale))
    }
}

fn displacement_of(
    owner: Option<UiMountedInstanceIdentity>,
    groups: &[(
        &UiMountedScrollMotionGroup,
        UiPublishedToAcceptedTranslation,
    )],
    presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
) -> Result<UiPublishedToAcceptedTranslation, Denial> {
    let none = UiPublishedToAcceptedTranslation::none(presentation);
    let Some(owner) = owner else {
        return Ok(none);
    };
    groups
        .iter()
        .filter(|(group, _)| {
            group
                .input
                .members
                .binary_search_by_key(&owner, |member| member.instance)
                .is_ok()
        })
        .try_fold(none, |sum, (_, delta)| sum.then(*delta))
}

fn sampled_thumb(
    group: &UiMountedScrollMotionGroup,
    axis: worth_ui_host_contract::UiMountedScrollChromeAxis,
    active: &ActiveSamples,
    tick: &UiPresentationMotionSampleReceipt,
) -> Result<UiMountedCanonicalBox, Denial> {
    let input = &group.input;
    let offset = group
        .candidate_offset(active, tick)?
        .chrome_derivation_offset()
        .ok_or(Denial::InvalidGeometry)?;
    let facts = UiScrollChromeFacts::derive(
        input.viewport,
        UiScrollBounds::from_mounted_region(input.content, input.viewport)
            .ok_or(Denial::InvalidGeometry)?,
        offset,
        input.chrome.as_ref().ok_or(Denial::UnknownTargetCommands)?,
    )
    .ok_or(Denial::InvalidGeometry)?;
    let axis = match axis {
        worth_ui_host_contract::UiMountedScrollChromeAxis::Inline => UiScrollChromeAxis::Inline,
        worth_ui_host_contract::UiMountedScrollChromeAxis::Block => UiScrollChromeAxis::Block,
    };
    snap_to_device_grid(
        facts.axis(axis).ok_or(Denial::InvalidGeometry)?.thumb(),
        input.scale,
    )
    .map_err(|_| Denial::InvalidGeometry)
}

fn translate(
    rect: UiMountedCanonicalBox,
    delta: [f32; 2],
) -> Result<UiMountedCanonicalBox, Denial> {
    bounds(
        [
            rect.x() + delta[0],
            rect.y() + delta[1],
            rect.width(),
            rect.height(),
        ],
        rect.coordinate_space(),
    )
}

fn add_clip(
    clip: &mut Option<UiMountedCanonicalBox>,
    next: UiMountedCanonicalBox,
) -> Result<(), Denial> {
    *clip = Some(match *clip {
        Some(old) => intersect(old, next)?,
        None => next,
    });
    Ok(())
}

fn intersect(
    a: UiMountedCanonicalBox,
    b: UiMountedCanonicalBox,
) -> Result<UiMountedCanonicalBox, Denial> {
    if a.coordinate_space() != b.coordinate_space() {
        return Err(Denial::InvalidGeometry);
    }
    let x = a.x().max(b.x());
    let y = a.y().max(b.y());
    bounds(
        [
            x,
            y,
            ((a.x() + a.width()).min(b.x() + b.width()) - x).max(0.0),
            ((a.y() + a.height()).min(b.y() + b.height()) - y).max(0.0),
        ],
        a.coordinate_space(),
    )
}

fn bounds(
    value: [f32; 4],
    coordinate_space: UiMountedCoordinateSpace,
) -> Result<UiMountedCanonicalBox, Denial> {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: value[0],
        y: value[1],
        width: value[2],
        height: value[3],
        coordinate_space,
    })
    .map_err(|_| Denial::InvalidGeometry)
}
