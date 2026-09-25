use worth_ui_host_contract::{
    UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
    UiMountedLogicalDamage, UiMountedPaintCommandIdentity, UiMountedPresentationSampleChange,
    UiMountedPresentationSampleInput, UiMountedPresentationTransform,
};

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
        let mut changes = Vec::new();
        let mut damage = Vec::new();
        let mut acceptance = Vec::new();
        let mut selected = std::collections::HashSet::new();
        let mut scroll_acceptance = Vec::new();
        for (change, sample) in self.scroll_sample_changes(sampling.samples(), presentation)? {
            selected.insert(change.command());
            damage.extend(
                change
                    .clip()
                    .map(UiMountedLogicalDamage::from_runtime_mounting),
            );
            acceptance.push(self.prepare_command_motion_update_with_change(
                change.command(),
                sample,
                change,
            ));
            changes.push(change);
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
            let portal_clip = portal_group
                .as_ref()
                .and_then(|group| group.viewport_clip());
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
                |group| group.commands().collect::<Vec<_>>(),
            );
            let mut targets = Vec::with_capacity(identities.len().max(1));
            for identity in identities {
                if identity.is_appearance_surface() {
                    let target = self
                        .appearance_surface_sample_target(identity.mounted_instance())
                        .ok_or(UiMountedMotionSampleWorkDenial::UnknownTargetCommands)?;
                    targets.push((
                        identity,
                        target.geometry().clip(),
                        Some(target.geometry().bounds()),
                    ));
                    continue;
                }
                let command = self
                    .command_option(identity)
                    .ok_or(UiMountedMotionSampleWorkDenial::UnknownTargetCommands)?;
                targets.push((
                    identity,
                    command.clip_bounds(),
                    super::command_visible_bounds(command),
                ));
            }
            if targets.is_empty() && portal_clip.is_none() {
                // An appearance-only instance is sampled through its painted surface.
                if let Some(target) = self.appearance_surface_sample_target(instance) {
                    let geometry = target.geometry();
                    targets.push((
                        UiMountedPaintCommandIdentity::appearance_surface(instance),
                        geometry.clip(),
                        Some(geometry.bounds()),
                    ));
                }
            }
            if targets.is_empty() {
                return Err(UiMountedMotionSampleWorkDenial::UnknownTargetCommands);
            }
            for (identity, clip, visible) in targets {
                if !selected.insert(identity) {
                    return Err(UiMountedMotionSampleWorkDenial::AmbiguousTargetCommands);
                }
                let transform = sample_transform(*sample, clip.coordinate_space())?;
                let opacity = super::super::compose_opacity(
                    self.appearance_opacity_for_command(identity),
                    sample.opacity_units(),
                );
                changes.push(UiMountedPresentationSampleChange::from_runtime_sampling(
                    identity, transform, opacity,
                ));
                acceptance.push(self.prepare_command_motion_update(identity, *sample));
                if sample.geometry().is_none() {
                    damage.extend(visible.map(UiMountedLogicalDamage::from_runtime_mounting));
                } else if portal_clip.is_none() {
                    append_clipped_damage(&mut damage, *sample, clip)?;
                }
            }
            if let Some(portal_clip) = portal_clip {
                append_clipped_damage(&mut damage, *sample, portal_clip)?;
            }
        }
        // Text image coverage may remain visible outside its logical allocation.
        // The host derives physical damage from retained admitted images.
        // A Scroll group can contain hit-only/empty content. Its exact group
        // update still crosses the host boundary; no paint is manufactured to
        // make the sample nonempty, and an unknown ordinary target still fails.
        if changes.is_empty() && scroll_acceptance.is_empty() {
            return Err(UiMountedMotionSampleWorkDenial::UnknownTargetCommands);
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

fn append_clipped_damage(
    damage: &mut Vec<UiMountedLogicalDamage>,
    sample: crate::mounting::presentation::motion_sampling::UiPresentationMotionSampleReceipt,
    clip: UiMountedCanonicalBox,
) -> Result<(), UiMountedMotionSampleWorkDenial> {
    let sampled_clip = clip_geometry(clip)?;
    damage.extend(
        sample
            .damage()
            .clipped_to(sampled_clip)
            .into_iter()
            .flatten()
            .map(|region| {
                logical_damage(region.components(), clip.coordinate_space())
                    .map_err(|_| UiMountedMotionSampleWorkDenial::InvalidGeometry)
            })
            .collect::<Result<Vec<_>, _>>()?,
    );
    Ok(())
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

fn logical_damage(
    components: [f32; 4],
    coordinate_space: UiMountedCoordinateSpace,
) -> Result<UiMountedLogicalDamage, worth_ui_host_contract::UiMountedGeometryDenial> {
    canonical_box(components, coordinate_space).map(UiMountedLogicalDamage::from_runtime_mounting)
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
