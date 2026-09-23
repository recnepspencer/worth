//! Compose all affecting Scroll groups before issuing one change per command.
use super::super::motion_sample::UiMountedMotionSampleWorkDenial as Denial;
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
                    Ok((group, group_delta(group, &active)?))
                })
                .collect::<Result<Vec<_>, Denial>>()?;
            let mut clip = None;
            for part in command.clips.iter() {
                let delta = displacement_of(part.owner, &groups);
                add_clip(&mut clip, translate(part.bounds, delta)?)?;
            }
            for (group, _) in &groups {
                let delta = displacement_of(Some(group.input.owner), &groups);
                add_clip(&mut clip, translate(group.input.viewport, delta)?)?;
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
                    placed = sampled_thumb(group, chrome.axis(), &active)?;
                }
                placed = translate(
                    placed,
                    displacement_of(Some(chrome.owner_instance()), &groups),
                )?;
                (
                    UiMountedPresentationTransform::from_runtime_sampling(target.bounds, placed)
                        .map_err(|_| Denial::InvalidGeometry)?,
                    target.opacity,
                )
            } else {
                let translation = groups
                    .iter()
                    .fold(command.base_translation, |sum, (_, delta)| add(sum, *delta));
                let source = bounds([0.0, 0.0, 1.0, 1.0], UiMountedCoordinateSpace::Viewport)?;
                (
                    UiMountedPresentationTransform::from_runtime_sampling(
                        source,
                        translate(source, translation)?,
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

fn sampled_offset(
    group: &UiMountedScrollMotionGroup,
    active: &ActiveSamples,
) -> Result<[f64; 2], Denial> {
    let input = &group.input;
    let sample = active
        .get(&input.owner)
        .copied()
        .or_else(|| group.accepted.get());
    match sample {
        Some(sample) => {
            let sampled = sample
                .geometry()
                .ok_or(Denial::InvalidGeometry)?
                .components();
            Ok([
                f64::from(input.content.x() - sampled[0]),
                f64::from(input.content.y() - sampled[1]),
            ])
        }
        None => Ok(offset_points(input.offset)),
    }
}

fn offset_points(offset: UiScrollOffset) -> [f64; 2] {
    let unit = worth_ui_host_contract::UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f64;
    [
        offset.inline_subpixels() as f64 / unit,
        offset.block_subpixels() as f64 / unit,
    ]
}

fn group_delta(
    group: &UiMountedScrollMotionGroup,
    active: &ActiveSamples,
) -> Result<[f32; 2], Denial> {
    let desired = sampled_offset(group, active)?;
    let previous = offset_points(group.input.offset);
    let snap = |value| value - group.input.scale.grid_residue(value);
    Ok([
        (snap(previous[0]) - snap(desired[0])) as f32,
        (snap(previous[1]) - snap(desired[1])) as f32,
    ])
}

fn displacement_of(
    owner: Option<UiMountedInstanceIdentity>,
    groups: &[(&UiMountedScrollMotionGroup, [f32; 2])],
) -> [f32; 2] {
    let Some(owner) = owner else {
        return [0.0; 2];
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
        .fold([0.0; 2], |sum, (_, delta)| add(sum, *delta))
}

fn sampled_thumb(
    group: &UiMountedScrollMotionGroup,
    axis: worth_ui_host_contract::UiMountedScrollChromeAxis,
    active: &ActiveSamples,
) -> Result<UiMountedCanonicalBox, Denial> {
    let input = &group.input;
    let desired = sampled_offset(group, active)?;
    let unit = worth_ui_host_contract::UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f64;
    let offset = UiScrollOffset::new(
        (desired[0].max(0.0) * unit).round() as i64,
        (desired[1].max(0.0) * unit).round() as i64,
    )
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

fn add(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] + b[0], a[1] + b[1]]
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
