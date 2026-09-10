use worth_ui_host_contract::{
    UiHostSurfacePresentationDenial, UiMountedFrameIdentity, UiMountedPaintCommandIdentity,
    UiMountedProjectionView, UiSurfaceBindingGeneration,
};

use super::super::work_producer::{UiMountedPresentationCandidates, UiMountedPresentationState};
/// One structural inheritance decision, retained from admission through issuance.
/// Unchanged command records keep their live acceptance slots, not copied samples.
pub(in crate::mounting::presentation) struct UiPreparedFrameCandidates {
    pub(super) surfaces: Vec<UiPreparedSurfaceCandidate>,
}

#[derive(Default)]
pub(crate) struct UiAcceptedAppearanceMotion {
    by_command: std::collections::HashMap<UiMountedPaintCommandIdentity, Option<u16>>,
    by_instance: std::collections::HashMap<
        (worth_ui_host_contract::UiMountedInstanceIdentity, bool),
        UiAcceptedInstanceMotion,
    >,
    text_fallback_by_instance: std::collections::HashMap<
        worth_ui_host_contract::UiMountedInstanceIdentity,
        UiAcceptedInstanceMotion,
    >,
    commands_visited: usize,
}

#[derive(Clone, Copy)]
enum UiAcceptedInstanceMotion {
    Uniform(Option<u16>),
    Ambiguous,
}

impl UiAcceptedAppearanceMotion {
    #[cfg(test)]
    pub(crate) fn from_text_commands_for_test(
        commands: &[(UiMountedPaintCommandIdentity, Option<u16>)],
    ) -> Self {
        let mut motion = Self::default();
        for (command, opacity) in commands {
            motion.by_command.insert(*command, *opacity);
            insert_instance_motion(
                &mut motion.text_fallback_by_instance,
                command.mounted_instance(),
                *opacity,
            );
        }
        motion.commands_visited = commands.len();
        motion
    }

    pub(crate) fn opacity_for(
        &self,
        command: UiMountedPaintCommandIdentity,
    ) -> Option<Option<u16>> {
        self.by_command.get(&command).copied()
    }

    pub(crate) fn opacity_for_instance(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        portal_surface: bool,
    ) -> Result<Option<Option<u16>>, ()> {
        match self.by_instance.get(&(instance, portal_surface)).copied() {
            None if !portal_surface => match self.text_fallback_by_instance.get(&instance).copied()
            {
                Some(UiAcceptedInstanceMotion::Uniform(opacity)) => Ok(Some(opacity)),
                Some(UiAcceptedInstanceMotion::Ambiguous) => Err(()),
                None => Ok(None),
            },
            None => Ok(None),
            Some(UiAcceptedInstanceMotion::Uniform(opacity)) => Ok(Some(opacity)),
            Some(UiAcceptedInstanceMotion::Ambiguous) => Err(()),
        }
    }

    pub(crate) const fn commands_visited(&self) -> usize {
        self.commands_visited
    }
}

pub(super) struct UiPreparedSurfaceCandidate {
    pub(super) state: UiMountedPresentationState,
    pub(super) predecessor: Option<UiMountedFrameIdentity>,
    pub(super) reconstruction_required: bool,
    pub(super) origin: CandidateOrigin,
}

pub(super) enum CandidateOrigin {
    Initial,
    Successor,
    Reconstruction {
        predecessor: UiMountedFrameIdentity,
        projection: UiMountedProjectionView,
    },
}

impl UiPreparedFrameCandidates {
    pub(in crate::mounting) fn bind_appearance_opacity(
        &mut self,
        frame: &crate::mounting::UiPreparedMountedFrame,
    ) {
        for surface in &mut self.surfaces {
            surface.state.bind_appearance_opacity(frame);
        }
    }

    pub(in crate::mounting) fn requires_complete_appearance_projection(&self) -> bool {
        self.surfaces.iter().any(|surface| {
            matches!(
                surface.origin,
                CandidateOrigin::Initial | CandidateOrigin::Reconstruction { .. }
            )
        })
    }

    #[cfg(test)]
    pub(in crate::mounting::presentation) fn from_single_state_for_test(
        state: UiMountedPresentationState,
    ) -> Self {
        Self {
            surfaces: vec![UiPreparedSurfaceCandidate {
                predecessor: None,
                reconstruction_required: false,
                origin: CandidateOrigin::Initial,
                state,
            }],
        }
    }

    pub(in crate::mounting) fn accepted_appearance_motion(
        &self,
        targets: &[(
            worth_ui_host_contract::UiSemanticSurfaceIdentity,
            worth_ui_host_contract::UiMountedInstanceIdentity,
        )],
    ) -> UiAcceptedAppearanceMotion {
        let mut by_command = std::collections::HashMap::new();
        let mut by_instance = std::collections::HashMap::new();
        let mut text_fallback_by_instance = std::collections::HashMap::new();
        let mut commands_visited = 0;
        let mut targets_by_surface = std::collections::BTreeMap::<_, Vec<_>>::new();
        for (surface, instance) in targets {
            targets_by_surface
                .entry(*surface)
                .or_default()
                .push(*instance);
        }
        for surface in &self.surfaces {
            let semantic_surface = surface.state.semantic_surface();
            let Some(instances) = targets_by_surface.get(&semantic_surface) else {
                continue;
            };
            for instance in instances {
                for (identity, portal_surface, instance_composable, sample) in surface
                    .state
                    .accepted_appearance_motion_for_instance(*instance)
                {
                    commands_visited += 1;
                    let opacity = sample.map(|sample| sample.opacity_units());
                    by_command.insert(identity, opacity);
                    if !instance_composable {
                        insert_instance_motion(
                            &mut text_fallback_by_instance,
                            identity.mounted_instance(),
                            opacity,
                        );
                        continue;
                    }
                    insert_instance_motion(
                        &mut by_instance,
                        (identity.mounted_instance(), portal_surface),
                        opacity,
                    );
                }
            }
        }
        UiAcceptedAppearanceMotion {
            by_command,
            by_instance,
            text_fallback_by_instance,
            commands_visited,
        }
    }

    pub(super) fn prepare(
        frame: &crate::mounting::UiPreparedMountedFrame,
        retained: &UiMountedPresentationCandidates,
        reconstruction_bindings: &std::collections::BTreeSet<UiSurfaceBindingGeneration>,
    ) -> Result<Self, UiHostSurfacePresentationDenial> {
        let source = frame.presentation_delta_source();
        let mut surfaces = Vec::with_capacity(frame.surfaces().len());
        for surface in frame.surfaces() {
            let predecessor = retained.get(&surface.requirement().binding());
            let reconstruction_required =
                reconstruction_bindings.contains(&surface.requirement().binding());
            let (state, origin) = match (source.predecessor(), predecessor) {
                (Some(source_frame), Some(previous))
                    if reconstruction_required && source_frame == previous.frame() =>
                {
                    reconstruct(surface, source_frame, Some(previous))?
                }
                (Some(source_frame), Some(previous)) if source_frame == previous.frame() => {
                    let projection = UiMountedPresentationState::successor_projection_required(
                        previous,
                        source,
                        surface.requirement(),
                    )
                    .then(|| surface.projection());
                    (
                        UiMountedPresentationState::successor_from_source(
                            previous,
                            source,
                            projection,
                            surface.requirement(),
                        ),
                        CandidateOrigin::Successor,
                    )
                }
                (None, None) => (
                    UiMountedPresentationState::from_projection(
                        surface.projection(),
                        surface.requirement(),
                        None,
                    ),
                    CandidateOrigin::Initial,
                ),
                (Some(source_frame), Some(previous)) if source_frame > previous.frame() => {
                    // A partially advanced frame reconstructs this exact older surface.
                    reconstruct(surface, previous.frame(), Some(previous))?
                }
                (Some(source_frame), None) => reconstruct(surface, source_frame, None)?,
                _ => return Err(UiHostSurfacePresentationDenial::StalePredecessor),
            };
            surfaces.push(UiPreparedSurfaceCandidate {
                state,
                predecessor: predecessor.map(UiMountedPresentationState::frame),
                reconstruction_required,
                origin,
            });
        }
        Ok(Self { surfaces })
    }
}

fn insert_instance_motion<K: std::hash::Hash + Eq>(
    entries: &mut std::collections::HashMap<K, UiAcceptedInstanceMotion>,
    key: K,
    opacity: Option<u16>,
) {
    entries
        .entry(key)
        .and_modify(|current| {
            if !matches!(current, UiAcceptedInstanceMotion::Uniform(value) if *value == opacity) {
                *current = UiAcceptedInstanceMotion::Ambiguous;
            }
        })
        .or_insert(UiAcceptedInstanceMotion::Uniform(opacity));
}

fn reconstruct(
    surface: &crate::mounting::UiMountedSurfaceReceipt,
    predecessor: UiMountedFrameIdentity,
    retained: Option<&UiMountedPresentationState>,
) -> Result<(UiMountedPresentationState, CandidateOrigin), UiHostSurfacePresentationDenial> {
    let projection =
        worth_ui_host_contract::UiMountedPresentationAuxiliaryState::from_runtime_mounting(
            surface.projection(),
        )
        .reconstruct_authored()
        .map_err(|_| UiHostSurfacePresentationDenial::MalformedProjection)?;
    let mut state = UiMountedPresentationState::from_projection(
        &projection,
        surface.requirement(),
        Some(predecessor),
    );
    if let Some(retained) = retained {
        state.inherit_reconstruction_motion(retained);
    }
    Ok((
        state,
        CandidateOrigin::Reconstruction {
            predecessor,
            projection,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_ui_host_contract::UiMountedInstanceIdentity;

    #[test]
    fn replaced_text_candidate_keeps_exact_motion_without_blocking_its_sibling() {
        let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
        let retained =
            UiMountedPaintCommandIdentity::semantic_text_from_correspondence(instance, 0, None);
        let replacement = UiMountedPaintCommandIdentity::semantic_text_from_correspondence(
            instance,
            u16::MAX,
            None,
        );
        let mut motion = UiAcceptedAppearanceMotion::default();
        motion.by_command.insert(retained, Some(32_768));
        motion.by_command.insert(replacement, None);
        insert_instance_motion(
            &mut motion.text_fallback_by_instance,
            instance,
            Some(32_768),
        );
        insert_instance_motion(&mut motion.text_fallback_by_instance, instance, None);

        assert_eq!(motion.opacity_for(retained), Some(Some(32_768)));
        assert_eq!(motion.opacity_for(replacement), Some(None));
        assert_eq!(motion.opacity_for_instance(instance, false), Err(()));
    }
}
