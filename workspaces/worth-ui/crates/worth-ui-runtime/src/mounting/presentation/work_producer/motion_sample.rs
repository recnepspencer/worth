use worth_ui_host_contract::{
    UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
    UiMountedLogicalDamage, UiMountedPresentationSampleChange, UiMountedPresentationSampleInput,
    UiMountedPresentationTransform,
};

use super::{production_cost, LocalWorkCost, RetainedTraversalCost, UiMountedPresentationState};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiMountedMotionSampleWorkDenial {
    PresentationBasisMismatch,
    UnknownTargetCommands,
    AmbiguousTargetCommands,
    InvalidGeometry,
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
        for sample in sampling.samples() {
            if sample.presentation_basis() != presentation
                || sample.target().semantic_surface() != self.requirement.semantic_surface()
            {
                return Err(UiMountedMotionSampleWorkDenial::PresentationBasisMismatch);
            }
            let portal_group = self.portal_motion_group(sample.target());
            let portal_clip = portal_group
                .as_ref()
                .and_then(|group| group.viewport_clip());
            let identities = portal_group.map_or_else(
                || {
                    self.command_identities_for_instance(sample.target().mounted_instance())
                        .collect::<Vec<_>>()
                },
                |group| group.commands().collect::<Vec<_>>(),
            );
            if identities.is_empty() {
                return Err(UiMountedMotionSampleWorkDenial::UnknownTargetCommands);
            }
            for identity in identities {
                if !selected.insert(identity) {
                    return Err(UiMountedMotionSampleWorkDenial::AmbiguousTargetCommands);
                }
                let command = self
                    .command_option(identity)
                    .ok_or(UiMountedMotionSampleWorkDenial::UnknownTargetCommands)?;
                let transform =
                    sample_transform(*sample, command.clip_bounds().coordinate_space())?;
                let opacity = super::super::compose_opacity(
                    self.appearance_opacity_for_command(identity),
                    sample.opacity_units(),
                );
                changes.push(UiMountedPresentationSampleChange::from_runtime_sampling(
                    identity, transform, opacity,
                ));
                acceptance.push(self.prepare_command_motion_update(identity, *sample));
                if sample.geometry().is_none() {
                    damage.extend(
                        super::command_visible_bounds(command)
                            .map(UiMountedLogicalDamage::from_runtime_mounting),
                    );
                } else if portal_clip.is_none() {
                    append_clipped_damage(&mut damage, *sample, command.clip_bounds())?;
                }
            }
            if let Some(portal_clip) = portal_clip {
                append_clipped_damage(&mut damage, *sample, portal_clip)?;
            }
        }
        // Text image coverage may remain visible outside its logical allocation.
        // The host derives physical damage from retained admitted images.
        if changes.is_empty() {
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
            super::motion_evidence::UiPreparedCommandMotionAcceptance::new(acceptance),
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
