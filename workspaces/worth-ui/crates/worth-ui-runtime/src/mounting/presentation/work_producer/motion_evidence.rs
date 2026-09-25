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
/// displayed basis writes it.
#[derive(Clone, Copy)]
struct UiDisplayedCommandMotion {
    sample: UiPresentationMotionSampleReceipt,
    change: UiMountedPresentationSampleChange,
    layers: UiCommandMotionLayers,
}

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
}

pub(super) struct UiCommandMotionUpdate {
    command: UiMountedPaintCommandIdentity,
    slot: UiCommandMotionAcceptance,
    sample: UiPresentationMotionSampleReceipt,
    change: UiMountedPresentationSampleChange,
    layers: UiCommandMotionLayers,
}

#[derive(Clone)]
pub(super) struct UiPreparedEntranceAcceptance {
    entrance: crate::runtime::motion::UiPreparedMotionEntrance,
    commands: Box<
        [(
            UiMountedPaintCommandIdentity,
            UiCommandMotionAcceptance,
            UiMountedPresentationSampleChange,
            UiCommandMotionLayers,
        )],
    >,
}

impl UiMountedPresentationState {
    pub(super) fn retain_entrance_acceptance(
        &mut self,
        entrance: crate::runtime::motion::UiPreparedMotionEntrance,
        changes: &[(UiMountedPresentationSampleChange, UiCommandMotionLayers)],
    ) -> Result<(), worth_ui_host_contract::UiHostSurfacePresentationDenial> {
        let commands = changes
            .iter()
            .map(|(change, layers)| {
                self.motion_slot(change.command()).cloned().map(|slot| (change.command(), slot, *change, *layers))
                .ok_or(worth_ui_host_contract::UiHostSurfacePresentationDenial::MalformedProjection)
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.entrance_acceptance = Some(UiPreparedEntranceAcceptance {
            entrance,
            commands: commands.into_boxed_slice(),
        });
        Ok(())
    }

    /// Accept a published entrance as displayed, once the witness that
    /// published it is the one `displayed` names.
    pub(in crate::mounting::presentation) fn accept_entrance(
        &mut self,
        sample: UiPresentationMotionSampleReceipt,
        displayed: crate::mounting::presentation::UiDisplayedSurfaceBasis,
    ) -> Result<bool, UiCommandMotionAcceptanceDenial> {
        let Some(prepared) = self.entrance_acceptance.as_ref() else {
            return Ok(false);
        };
        let entrance = prepared.entrance;
        if entrance.track() != sample.track()
            || entrance.target() != sample.target()
            || entrance.frame() != sample.presentation_basis().frame()
            || displayed.basis() != sample.presentation_basis()
            || sample.opacity_units() != 0
            || entrance.geometry().0 != sample.base_geometry()
            || !match (entrance.geometry().1, sample.geometry()) {
                (Some(initial), Some(sampled)) => sampled.stands_at(initial),
                (None, None) => true,
                _ => false,
            }
        {
            return Err(UiCommandMotionAcceptanceDenial::SampleBasis);
        }
        let updates = prepared
            .commands
            .iter()
            .map(|(command, slot, change, layers)| UiCommandMotionUpdate {
                command: *command,
                slot: slot.clone(),
                sample,
                change: *change,
                layers: *layers,
            })
            .collect();
        UiPreparedCommandMotionAcceptance::new(updates).accept_at(self, displayed)?;
        self.entrance_acceptance = None;
        Ok(true)
    }
}

/// Prepared before host effects; dropping it changes no accepted evidence.
#[derive(Default)]
pub(in crate::mounting::presentation) struct UiPreparedCommandMotionAcceptance {
    updates: Box<[UiCommandMotionUpdate]>,
    scroll: Box<[super::scroll_motion_groups::UiScrollGroupMotionUpdate]>,
}

#[derive(Debug)]
pub(in crate::mounting::presentation) enum UiCommandMotionAcceptanceDenial {
    PresentationChanged,
    CommandReplaced,
    SampleBasis,
}

impl UiPreparedCommandMotionAcceptance {
    pub(super) fn new(updates: Vec<UiCommandMotionUpdate>) -> Self {
        Self {
            updates: updates.into_boxed_slice(),
            scroll: Box::new([]),
        }
    }

    pub(super) fn with_scroll_groups(
        mut self,
        scroll: Vec<super::scroll_motion_groups::UiScrollGroupMotionUpdate>,
    ) -> Self {
        self.scroll = scroll.into_boxed_slice();
        self
    }

    pub(in crate::mounting::presentation) fn accept(
        self,
        current: &UiMountedPresentationState,
        witness: &crate::mounting::presentation::UiPresentedSurfaceWitness,
    ) -> Result<(), UiCommandMotionAcceptanceDenial> {
        self.accept_at(current, witness.displayed_basis())
    }

    fn accept_at(
        mut self,
        current: &UiMountedPresentationState,
        displayed: crate::mounting::presentation::UiDisplayedSurfaceBasis,
    ) -> Result<(), UiCommandMotionAcceptanceDenial> {
        let presentation = displayed.basis();
        let requirement = current.motion_sample_requirement();
        if current.frame() != presentation.frame()
            || requirement.binding() != presentation.binding()
            || requirement.host_surface() != presentation.host_surface()
        {
            return Err(UiCommandMotionAcceptanceDenial::PresentationChanged);
        }
        // Validate every command and reattribute every receipt before any write.
        for update in &mut self.updates {
            if update.change.command() != update.command {
                return Err(UiCommandMotionAcceptanceDenial::SampleBasis);
            }
            let slot = current
                .motion_slot(update.command)
                .ok_or(UiCommandMotionAcceptanceDenial::CommandReplaced)?;
            if !Rc::ptr_eq(&slot.0, &update.slot.0) {
                return Err(UiCommandMotionAcceptanceDenial::CommandReplaced);
            }
            update.sample = update
                .sample
                .with_presentation_basis(presentation)
                .map_err(|_| UiCommandMotionAcceptanceDenial::SampleBasis)?;
        }
        let scroll = self
            .scroll
            .iter()
            .map(|group| group.validate(current, displayed))
            .collect::<Result<Vec<_>, _>>()?;
        for update in self.updates {
            update.slot.0.set(Some(UiDisplayedCommandMotion {
                sample: update.sample,
                change: update.change,
                layers: update.layers,
            }));
        }
        for group in scroll {
            group.commit();
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
            .flat_map(|instance| {
                self.command_identities_for_instance(*instance)
                    .filter_map(|identity| self.command_sample_change(identity))
                    .chain(self.appearance_surface_sample_change(*instance))
                    .chain(
                        self.scroll_motion_groups
                            .chrome_identities()
                            .filter(move |identity| identity.mounted_instance() == *instance)
                            .filter_map(|identity| self.accepted_motion_change(identity)),
                    )
            })
            .collect()
    }

    pub(super) fn command_sample_change(
        &self,
        identity: UiMountedPaintCommandIdentity,
    ) -> Option<worth_ui_host_contract::UiMountedPresentationSampleChange> {
        let accepted = self.accepted_motion_change(identity)?;
        let opacity = super::super::compose_opacity(
            self.appearance_opacity_for_command(identity),
            accepted.opacity().motion_units(),
        );
        Some(match accepted.clip() {
            Some(clip) => UiMountedPresentationSampleChange::from_runtime_scroll_sampling(
                identity,
                accepted.transform()?,
                opacity,
                clip,
            )
            .expect("accepted physical sample preserves its coordinate space"),
            None => UiMountedPresentationSampleChange::from_runtime_sampling(
                identity,
                accepted.transform(),
                opacity,
            ),
        })
    }

    pub(super) fn prepare_command_motion_update_with_change(
        &self,
        command: UiMountedPaintCommandIdentity,
        sample: UiPresentationMotionSampleReceipt,
        change: UiMountedPresentationSampleChange,
        layers: UiCommandMotionLayers,
    ) -> UiCommandMotionUpdate {
        UiCommandMotionUpdate {
            command,
            slot: self
                .motion_slot(command)
                .expect("prepared command is admitted")
                .clone(),
            sample,
            change,
            layers,
        }
    }

    fn motion_slot(
        &self,
        command: UiMountedPaintCommandIdentity,
    ) -> Option<&UiCommandMotionAcceptance> {
        if let Some(identity) = command.scroll_chrome_identity() {
            return self.scroll_motion_groups.motion_slot(identity);
        }
        if command.is_appearance_surface() {
            return self
                .appearance_surface_sample_target(command.mounted_instance())
                .map(super::state::UiMountedAppearanceSurfaceSampleTarget::motion);
        }
        self.commands_by_instance
            .get(&command.mounted_instance())?
            .motion_slot(command)
    }

    pub(super) fn accepted_motion_change(
        &self,
        command: UiMountedPaintCommandIdentity,
    ) -> Option<UiMountedPresentationSampleChange> {
        self.motion_slot(command)?
            .0
            .get()
            .map(|accepted| accepted.change)
    }

    /// The Motion layers the host shows `command` through now: none before
    /// any is accepted.
    pub(super) fn accepted_motion_layers(
        &self,
        command: UiMountedPaintCommandIdentity,
    ) -> UiCommandMotionLayers {
        self.motion_slot(command)
            .and_then(|slot| slot.0.get())
            .map_or_else(UiCommandMotionLayers::default, |accepted| accepted.layers)
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
        self.inherit_appearance_surface_targets(predecessor);
        self.scroll_motion_groups = predecessor.scroll_motion_groups.clone();
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
            .filter_map(|(identity, _)| self.command_sample_change(identity))
            .chain(
                self.bound_appearance_surface_instances()
                    .filter_map(|instance| self.appearance_surface_sample_change(instance)),
            )
            .chain(
                self.scroll_motion_groups
                    .chrome_identities()
                    .filter_map(|identity| self.accepted_motion_change(identity)),
            )
            .collect()
    }
}

/// Structural reservation includes a fixed live slot, its Rc control words,
/// and pending boxed updates. Preparation seals their exact length before effects.
/// The command count is an admitted upper bound, including clipped commands.
pub(in crate::mounting) fn motion_acceptance_reserved_bytes(commands: usize) -> Option<usize> {
    let per_command = std::mem::size_of::<UiCommandMotionAcceptance>()
        .checked_add(std::mem::size_of::<Option<UiDisplayedCommandMotion>>())?
        .checked_add(2 * std::mem::size_of::<usize>())?;
    commands.checked_mul(per_command.checked_add(std::mem::size_of::<UiCommandMotionUpdate>())?)
}

#[cfg(test)]
#[path = "motion_evidence_tests.rs"]
mod tests;
