use std::collections::HashMap;

use worth_ui_host_contract::{
    UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
    UiMountedPaintCommandIdentity, UiMountedPresentationSampleInput,
    UiMountedPresentationTransform,
};

use super::command_motion_layers::{UiCommandMotionLayer, UiCommandMotionLayerKind as Layer};
use super::command_motion_ticks::{UiCommandMotionTicks, UiMotionTickDamage as Repaint};
use super::motion_sample_command::UiMotionSampleCommand;
use super::{production_cost, LocalWorkCost, RetainedTraversalCost, UiMountedPresentationState};
use crate::runtime::motion::UiMotionTargetScope;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiMountedMotionSampleWorkDenial {
    PresentationBasisMismatch,
    UnknownTargetCommands,
    AmbiguousTargetCommands,
    InvalidGeometry,
    /// A command's displayed base was read by another bind than the group
    /// standing it is moved from.
    DisplayedBaseFromAnotherBind,
}

impl UiMountedPresentationState {
    /// One change per command the tick moves. A command several Motions move
    /// shows them composed, and every layer the tick does not sample holds
    /// where the host shows it: a Portal closing over a settling Scroll plays
    /// both.
    pub(in crate::mounting::presentation) fn prepare_motion_sample(
        &self,
        sampling: &crate::mounting::presentation::motion_sampling::UiPresentationMotionSamplingReceipt,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        lease: &crate::mounting::presentation::UiMountedPresentationLease,
    ) -> Result<
        (
            crate::mounting::presentation::UiMountedPresentationWork,
            super::motion_evidence::UiPreparedCommandMotionAcceptance,
        ),
        UiMountedMotionSampleWorkDenial,
    > {
        if self.frame != presentation.frame()
            || self.requirement.host_surface() != presentation.host_surface()
            || self.requirement.binding() != presentation.binding()
        {
            return Err(UiMountedMotionSampleWorkDenial::PresentationBasisMismatch);
        }
        let mut ticks = UiCommandMotionTicks::default();
        let mut scroll_acceptance = Vec::new();
        for (change, sample) in self.scroll_sample_changes(sampling.samples(), presentation)? {
            let layer = change
                .transform()
                .zip(change.clip())
                .map(|(transform, clip)| {
                    UiCommandMotionLayer::scrolled(transform, clip, sample.opacity_units())
                })
                .ok_or(UiMountedMotionSampleWorkDenial::InvalidGeometry)?;
            ticks.sample(change.command(), Layer::Scroll, layer, sample)?;
            ticks.repaint(Repaint::Scrolled(change.command()));
        }
        for sample in sampling.samples() {
            if sample.presentation_basis() != presentation
                || sample.target().semantic_surface() != self.requirement.semantic_surface()
            {
                return Err(UiMountedMotionSampleWorkDenial::PresentationBasisMismatch);
            }
            let portal_group = self.portal_motion_group(sample.target());
            match sample.target().scope() {
                UiMotionTargetScope::ScrollContents => {
                    scroll_acceptance.push(
                        super::scroll_motion_groups::UiScrollGroupMotionUpdate::prepare(
                            self, *sample,
                        )
                        .ok_or(UiMountedMotionSampleWorkDenial::UnknownTargetCommands)?,
                    );
                    continue;
                }
                UiMotionTargetScope::PortalContents if portal_group.is_none() => {
                    return Err(UiMountedMotionSampleWorkDenial::UnknownTargetCommands);
                }
                UiMotionTargetScope::Ordinary | UiMotionTargetScope::PortalContents => {}
            }
            let (kind, portal_clip) = portal_group.as_ref().map_or((Layer::Own, None), |group| {
                (Layer::Portal, group.viewport_clip())
            });
            let instance = sample.target().mounted_instance();
            let identities = portal_group.map_or_else(
                || {
                    self.command_identities_for_instance(instance)
                        .filter(|identity| {
                            !matches!(
                                self.command_option(*identity),
                                Some(worth_ui_host_contract::UiMountedPaintCommand::PortalOverlay { .. })
                            )
                        })
                        .collect::<Vec<_>>()
                },
                |group| {
                    group
                        .commands()
                        .chain(self.scroll_motion_groups.portal_chrome(sample.target()))
                        .collect::<Vec<_>>()
                },
            );
            let mut targets = identities
                .into_iter()
                .map(|identity| Ok((identity, self.motion_sample_command(identity)?)))
                .collect::<Result<Vec<_>, UiMountedMotionSampleWorkDenial>>()?;
            if targets.is_empty() && portal_clip.is_none() {
                // An appearance-only instance is sampled through its painted surface.
                let identity = UiMountedPaintCommandIdentity::appearance_surface(instance);
                if let Ok(target) = self.motion_sample_command(identity) {
                    targets.push((identity, target));
                }
            }
            if targets.is_empty() {
                return Err(UiMountedMotionSampleWorkDenial::UnknownTargetCommands);
            }
            for (identity, UiMotionSampleCommand { clip, visible }) in targets {
                let transform = sample_transform(*sample, clip.coordinate_space())?;
                let layer = UiCommandMotionLayer::moved(transform, sample.opacity_units());
                ticks.sample(identity, kind, layer, *sample)?;
                if sample.geometry().is_none() {
                    if let Some(visible) = visible {
                        ticks.repaint(Repaint::Faded(identity, visible));
                    }
                } else if portal_clip.is_none() {
                    ticks.repaint(Repaint::Moved(
                        identity,
                        kind,
                        clipped_damage(*sample, clip)?,
                    ));
                }
            }
            if let Some(portal_clip) = portal_clip {
                ticks.repaint(Repaint::Shown(clipped_damage(*sample, portal_clip)?));
            }
        }
        // Text image coverage may remain visible outside its logical allocation.
        // The host derives physical damage from retained admitted images.
        // A Scroll group can contain hit-only/empty content. Its exact group
        // update still crosses the host boundary; no paint is manufactured to
        // make the sample nonempty, and an unknown ordinary target still fails.
        if ticks.is_empty() && scroll_acceptance.is_empty() {
            return Err(UiMountedMotionSampleWorkDenial::UnknownTargetCommands);
        }
        let (ticks, planned) = ticks.into_parts();
        let mut changes = Vec::with_capacity(ticks.len());
        let mut acceptance = Vec::with_capacity(ticks.len());
        let mut shown = HashMap::with_capacity(ticks.len());
        for tick in ticks {
            let layers = tick.layers.over(self.accepted_motion_layers(tick.command));
            let change = layers.change(tick.command, self.resting_opacity(tick.command)?)?;
            changes.push(change);
            acceptance.push(self.prepare_command_motion_update_with_change(
                tick.command,
                tick.sample(),
                change,
                layers,
            ));
            shown.insert(tick.command, (layers, change));
        }
        let mut damage = Vec::new();
        for repaint in planned {
            repaint.resolve(&shown, &mut damage)?;
        }
        let work = lease.issue_sample(UiMountedPresentationSampleInput {
            frame: self.frame,
            surface: self.requirement.semantic_surface(),
            binding: self.requirement.binding(),
            content: self.content,
            baseline: self.requirement.baseline(),
            production_cost: production_cost(
                LocalWorkCost {
                    source_instances: sampling.samples().len(),
                    commands_considered: changes.len(),
                    // Target selection plus command and acceptance-slot lookups.
                    command_index_lookups: sampling.samples().len() + 2 * changes.len(),
                    order_lookups: 0,
                },
                RetainedTraversalCost::default(),
                0,
            ),
            changes,
            damage,
        });
        Ok((
            work,
            super::motion_evidence::UiPreparedCommandMotionAcceptance::new(acceptance)
                .with_scroll_groups(scroll_acceptance),
        ))
    }

    pub(crate) const fn motion_sample_requirement(
        &self,
    ) -> worth_ui_host_contract::UiMountedSurfaceBindingRequirement {
        self.requirement
    }
}

/// What `sample` repaints within `clip`.
fn clipped_damage(
    sample: crate::mounting::presentation::motion_sampling::UiPresentationMotionSampleReceipt,
    clip: UiMountedCanonicalBox,
) -> Result<Vec<UiMountedCanonicalBox>, UiMountedMotionSampleWorkDenial> {
    sample
        .damage()
        .clipped_to(clip_geometry(clip)?)
        .into_iter()
        .flatten()
        .map(|region| {
            canonical_box(region.components(), clip.coordinate_space())
                .map_err(|_| UiMountedMotionSampleWorkDenial::InvalidGeometry)
        })
        .collect()
}

pub(super) fn sample_transform(
    sample: crate::mounting::presentation::motion_sampling::UiPresentationMotionSampleReceipt,
    coordinate_space: UiMountedCoordinateSpace,
) -> Result<Option<UiMountedPresentationTransform>, UiMountedMotionSampleWorkDenial> {
    match (sample.base_geometry(), sample.geometry()) {
        (Some(source), Some(sampled)) => {
            let source = canonical_box(source.components(), coordinate_space)
                .map_err(|_| UiMountedMotionSampleWorkDenial::InvalidGeometry)?;
            let sampled = canonical_box(sampled.components(), coordinate_space)
                .map_err(|_| UiMountedMotionSampleWorkDenial::InvalidGeometry)?;
            UiMountedPresentationTransform::from_runtime_sampling(source, sampled)
                .map(Some)
                .map_err(|_| UiMountedMotionSampleWorkDenial::InvalidGeometry)
        }
        (None, None) => Ok(None),
        _ => Err(UiMountedMotionSampleWorkDenial::InvalidGeometry),
    }
}

fn clip_geometry(
    bounds: UiMountedCanonicalBox,
) -> Result<
    crate::mounting::presentation::motion_sampling::UiPresentationSampledClipGeometry,
    UiMountedMotionSampleWorkDenial,
> {
    crate::mounting::presentation::motion_sampling::UiPresentationSampledClipGeometry::from_presented_components([
        bounds.x(), bounds.y(), bounds.width(), bounds.height(),
    ])
    .map_err(|_| UiMountedMotionSampleWorkDenial::InvalidGeometry)
}

fn canonical_box(
    components: [f32; 4],
    coordinate_space: UiMountedCoordinateSpace,
) -> Result<UiMountedCanonicalBox, worth_ui_host_contract::UiMountedGeometryDenial> {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: components[0],
        y: components[1],
        width: components[2],
        height: components[3],
        coordinate_space,
    })
}
