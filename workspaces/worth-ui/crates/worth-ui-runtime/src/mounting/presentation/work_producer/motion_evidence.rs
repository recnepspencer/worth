use std::{cell::Cell, rc::Rc};
use worth_ui_host_contract::{UiHostObservationPresentationBasis, UiMountedPaintCommandIdentity};

use super::super::motion_sampling::UiPresentationMotionSampleReceipt;
use super::UiMountedPresentationState;

/// Live physical evidence, shared only by versions of one unchanged command.
/// It is never exposed as an immutable historical frame snapshot.
#[derive(Clone, Default)]
pub(super) struct UiCommandMotionAcceptance(Rc<Cell<Option<UiPresentationMotionSampleReceipt>>>);

impl UiCommandMotionAcceptance {
    pub(super) fn sample(&self) -> Option<UiPresentationMotionSampleReceipt> {
        self.0.get()
    }
}

pub(super) struct UiCommandMotionUpdate {
    command: UiMountedPaintCommandIdentity,
    slot: UiCommandMotionAcceptance,
    sample: UiPresentationMotionSampleReceipt,
}

/// Prepared before host effects; dropping it changes no accepted evidence.
#[derive(Default)]
pub(in crate::mounting::presentation) struct UiPreparedCommandMotionAcceptance {
    updates: Box<[UiCommandMotionUpdate]>,
}

#[derive(Debug)]
pub(in crate::mounting::presentation) enum UiCommandMotionAcceptanceDenial {
    PresentationChanged,
    CommandReplaced,
    SampleBasis(super::super::motion_sampling::UiPresentationGeometrySamplingDenial),
}

impl UiPreparedCommandMotionAcceptance {
    pub(super) fn new(updates: Vec<UiCommandMotionUpdate>) -> Self {
        Self {
            updates: updates.into_boxed_slice(),
        }
    }

    pub(in crate::mounting::presentation) fn accept(
        mut self,
        current: &UiMountedPresentationState,
        presentation: UiHostObservationPresentationBasis,
    ) -> Result<(), UiCommandMotionAcceptanceDenial> {
        let requirement = current.motion_sample_requirement();
        if current.frame() != presentation.frame()
            || requirement.binding() != presentation.binding()
            || requirement.host_surface() != presentation.host_surface()
        {
            return Err(UiCommandMotionAcceptanceDenial::PresentationChanged);
        }
        // Validate every command and reattribute every receipt before any write.
        for update in &mut self.updates {
            let slot = current
                .motion_slot(update.command)
                .ok_or(UiCommandMotionAcceptanceDenial::CommandReplaced)?;
            if !Rc::ptr_eq(&slot.0, &update.slot.0) {
                return Err(UiCommandMotionAcceptanceDenial::CommandReplaced);
            }
            update.sample = update
                .sample
                .with_presentation_basis(presentation)
                .map_err(UiCommandMotionAcceptanceDenial::SampleBasis)?;
        }
        for update in self.updates {
            update.slot.0.set(Some(update.sample));
        }
        Ok(())
    }
}

impl UiMountedPresentationState {
    pub(in crate::mounting::presentation) fn appearance_motion_overrides(
        &self,
        instances: &[worth_ui_host_contract::UiMountedInstanceIdentity],
    ) -> Vec<worth_ui_host_contract::UiMountedPresentationSampleChange> {
        instances
            .iter()
            .flat_map(|instance| self.command_identities_for_instance(*instance))
            .filter_map(|identity| {
                let sample = self.motion_for_command(identity)??;
                let command = self.command_option(identity)?;
                let transform = super::motion_sample::sample_transform(
                    sample,
                    command.clip_bounds().coordinate_space(),
                )
                .ok()?;
                Some(
                    worth_ui_host_contract::UiMountedPresentationSampleChange::from_runtime_sampling(
                        identity,
                        transform,
                        super::super::compose_opacity(
                            self.appearance_opacity_for_command(identity),
                            sample.opacity_units(),
                        ),
                    ),
                )
            })
            .collect()
    }

    pub(super) fn prepare_command_motion_update(
        &self,
        command: UiMountedPaintCommandIdentity,
        sample: UiPresentationMotionSampleReceipt,
    ) -> UiCommandMotionUpdate {
        UiCommandMotionUpdate {
            command,
            slot: self
                .motion_slot(command)
                .expect("prepared command is admitted")
                .clone(),
            sample,
        }
    }

    fn motion_slot(
        &self,
        command: UiMountedPaintCommandIdentity,
    ) -> Option<&UiCommandMotionAcceptance> {
        self.commands_by_instance
            .get(&command.mounted_instance())?
            .motion_slot(command)
    }

    pub(in crate::mounting::presentation) fn motion_for_command(
        &self,
        command: UiMountedPaintCommandIdentity,
    ) -> Option<Option<UiPresentationMotionSampleReceipt>> {
        self.motion_slot(command)
            .map(UiCommandMotionAcceptance::sample)
    }

    pub(super) fn inherit_unchanged_motion(
        &mut self,
        predecessor: &Self,
        changed: &[worth_ui_host_contract::UiMountedInstanceIdentity],
    ) {
        for instance in changed {
            let Some(mut bundle) = self.commands_by_instance.get(instance).cloned() else {
                continue;
            };
            let Some(previous) = predecessor.commands_by_instance.get(instance) else {
                continue;
            };
            bundle.inherit_unchanged_motion(previous);
            self.commands_by_instance.insert(*instance, bundle);
        }
    }

    pub(in crate::mounting::presentation) fn inherit_reconstruction_motion(
        &mut self,
        predecessor: &Self,
    ) {
        if self.requirement.semantic_surface() != predecessor.requirement.semantic_surface() {
            return;
        }
        let affected = predecessor
            .rebound_from_binding
            .unwrap_or_else(|| predecessor.requirement.binding());
        let replacement = self.requirement.binding();
        let instances = self
            .commands_by_instance
            .iter()
            .map(|(instance, _)| *instance)
            .collect::<Vec<_>>();
        if affected == replacement {
            self.inherit_unchanged_motion(predecessor, &instances);
            return;
        }
        for instance in instances {
            let Some(mut bundle) = self.commands_by_instance.get(&instance).cloned() else {
                continue;
            };
            let Some(previous) = predecessor.commands_by_instance.get(&instance) else {
                continue;
            };
            bundle.inherit_rebound_motion(previous, affected, replacement);
            self.commands_by_instance.insert(instance, bundle);
        }
    }

    pub(super) fn reconstruction_motion_overrides(
        &self,
    ) -> Vec<worth_ui_host_contract::UiMountedPresentationSampleChange> {
        self.reconstruction_appearance_motion()
            .filter_map(|(identity, sample)| {
                let sample = sample?;
                let command = self.command_option(identity)?;
                let transform = super::motion_sample::sample_transform(
                    sample,
                    command.clip_bounds().coordinate_space(),
                )
                .expect("accepted Motion geometry remains valid for an equivalent command");
                let opacity = super::super::compose_opacity(
                    self.appearance_opacity_for_command(identity),
                    sample.opacity_units(),
                );
                Some(
                    worth_ui_host_contract::UiMountedPresentationSampleChange::from_runtime_sampling(
                        identity, transform, opacity,
                    ),
                )
            })
            .collect()
    }
}

/// Structural reservation includes a fixed live slot, its Rc control words,
/// and pending boxed updates. Preparation seals their exact length before effects.
/// The command count is an admitted upper bound, including clipped commands.
pub(in crate::mounting) fn motion_acceptance_reserved_bytes(commands: usize) -> Option<usize> {
    let per_command = std::mem::size_of::<UiCommandMotionAcceptance>()
        .checked_add(std::mem::size_of::<Option<UiPresentationMotionSampleReceipt>>())?
        .checked_add(2 * std::mem::size_of::<usize>())?;
    commands.checked_mul(per_command.checked_add(std::mem::size_of::<UiCommandMotionUpdate>())?)
}

#[cfg(test)]
#[path = "motion_evidence_tests.rs"]
mod tests;
