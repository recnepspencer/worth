use std::collections::BTreeMap;

use worth_ui_host_contract::{
    UiHostObservationCanonicalCore, UiHostObservationPayload, UiHostObservationPresentationBasis,
    UiHostObservationSequence, UiHostPointerIdentity, UiHostSurfacePosition,
    UiMountedInstanceIdentity, UiSemanticSurfaceIdentity,
};

use super::inspection::primary_pointer_admitted;
use super::presentation::UiPointerPresencePresentationTrigger;
use super::{
    UiPointerPresenceAppearanceOwnerSnapshot, UiPointerPresenceAppearancePosture,
    UiPointerPresenceClass, UiPointerPresenceTargetTransition, UiPrimaryPointerKind,
};

pub(crate) struct UiPointerPresenceOwner {
    pointers: BTreeMap<UiHostPointerIdentity, UiPointerPresenceRecord>,
    primary_by_surface: BTreeMap<UiSemanticSurfaceIdentity, UiHostPointerIdentity>,
    revision: u64,
}

struct UiPointerPresenceRecord {
    kind: UiPrimaryPointerKind,
    surface: Option<UiSemanticSurfaceIdentity>,
    binding: Option<worth_ui_host_contract::UiSurfaceBindingGeneration>,
    target: Option<UiMountedInstanceIdentity>,
    node_receipt: Option<worth_ui_host_contract::UiMountedNodeReceiptIdentity>,
    sequence: UiHostObservationSequence,
    #[allow(
        dead_code,
        reason = "Gate 0 retains admitted pointer geometry without host emission"
    )]
    position: UiHostSurfacePosition,
    #[allow(
        dead_code,
        reason = "Gate 0 retains the exact presentation basis without emission"
    )]
    presentation: UiHostObservationPresentationBasis,
}

impl UiPointerPresenceOwner {
    pub(crate) const fn new() -> Self {
        Self {
            pointers: BTreeMap::new(),
            primary_by_surface: BTreeMap::new(),
            revision: 0,
        }
    }

    pub(crate) fn process_mouse_report(
        &mut self,
        core: UiHostObservationCanonicalCore,
        report: &worth_ui_host_contract::UiHostObservationReport,
        mounted: &crate::mounting::WorthUiMountedSessionState,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> Option<UiPointerPresenceTargetTransition> {
        self.process_pointer_report(
            core,
            report,
            UiPrimaryPointerKind::Mouse,
            mounted,
            generation,
        )
    }

    pub(crate) fn process_pointer_report(
        &mut self,
        core: UiHostObservationCanonicalCore,
        report: &worth_ui_host_contract::UiHostObservationReport,
        kind: UiPrimaryPointerKind,
        mounted: &crate::mounting::WorthUiMountedSessionState,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> Option<UiPointerPresenceTargetTransition> {
        let UiHostObservationPayload::PointerMotion {
            pointer, position, ..
        } = report.payload()
        else {
            return None;
        };
        let resolved = crate::runtime::interaction::targeting::resolve_presented_target(
            mounted,
            core.presentation(),
            *position,
        )
        .ok()
        .map(|target| {
            (
                target.surface(),
                target.binding(),
                target.mounted_instance(),
                target.node_receipt(),
            )
        });
        self.record_pointer_target(
            *pointer,
            kind,
            report.sequence(),
            *position,
            core.presentation(),
            resolved,
            generation,
        )
    }

    fn record_mouse_target(
        &mut self,
        pointer: UiHostPointerIdentity,
        sequence: UiHostObservationSequence,
        position: UiHostSurfacePosition,
        presentation: UiHostObservationPresentationBasis,
        resolved: Option<(
            UiSemanticSurfaceIdentity,
            worth_ui_host_contract::UiSurfaceBindingGeneration,
            UiMountedInstanceIdentity,
            worth_ui_host_contract::UiMountedNodeReceiptIdentity,
        )>,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> Option<UiPointerPresenceTargetTransition> {
        self.record_pointer_target(
            pointer,
            UiPrimaryPointerKind::Mouse,
            sequence,
            position,
            presentation,
            resolved,
            generation,
        )
    }

    pub(crate) fn record_pointer_target(
        &mut self,
        pointer: UiHostPointerIdentity,
        kind: UiPrimaryPointerKind,
        sequence: UiHostObservationSequence,
        position: UiHostSurfacePosition,
        presentation: UiHostObservationPresentationBasis,
        resolved: Option<(
            UiSemanticSurfaceIdentity,
            worth_ui_host_contract::UiSurfaceBindingGeneration,
            UiMountedInstanceIdentity,
            worth_ui_host_contract::UiMountedNodeReceiptIdentity,
        )>,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> Option<UiPointerPresenceTargetTransition> {
        let prior = self.pointers.get(&pointer);
        let prior_surface = prior.and_then(|record| record.surface);
        let surface = resolved.as_ref().map(|target| target.0).or(prior_surface);
        let target = resolved.as_ref().map(|target| target.2);
        let node_receipt = resolved.as_ref().map(|target| target.3);
        let binding = resolved
            .as_ref()
            .map(|target| target.1)
            .or_else(|| prior.and_then(|record| record.binding));
        let previous = prior.and_then(|record| record.target);
        let previous_node_receipt = prior.and_then(|record| record.node_receipt);
        let record_changed = prior.is_none_or(|record| {
            record.surface != surface
                || record.target != target
                || record.binding != binding
                || record.node_receipt != node_receipt
                || record.kind != kind
        });
        let prior_was_primary = prior_surface.is_some_and(|prior_surface_identity| {
            prior.is_some_and(|record| primary_pointer_admitted(record.kind))
                && self.primary_by_surface.get(&prior_surface_identity) == Some(&pointer)
        });
        let current_is_primary = primary_pointer_admitted(kind);
        let primary_changed = prior_was_primary
            && (Some(prior_surface.expect("a primary pointer has a surface")) != surface
                || !current_is_primary)
            || current_is_primary
                && surface.is_some_and(|current_surface| {
                    self.primary_by_surface.get(&current_surface) != Some(&pointer)
                });
        self.reassign_primary(pointer, prior_surface, surface, kind);
        self.pointers.insert(
            pointer,
            UiPointerPresenceRecord {
                kind,
                surface,
                binding,
                target,
                node_receipt,
                sequence,
                position,
                presentation,
            },
        );
        let changed = record_changed || primary_changed;
        if changed {
            self.bump_revision();
        }
        changed.then_some(UiPointerPresenceTargetTransition {
            generation: generation.clone(),
            pointer,
            previous_surface: prior_surface,
            current_surface: surface,
            previous,
            current: target,
            previous_node_receipt,
            current_node_receipt: node_receipt,
            owner_revision: self.revision,
            position,
            presentation,
        })
    }

    pub(crate) fn retest_committed_presentation(
        &mut self,
        trigger: &UiPointerPresencePresentationTrigger,
        mounted: &crate::mounting::WorthUiMountedSessionState,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> usize {
        if mounted
            .validate_current_frame(trigger.presentation().frame())
            .is_err()
            || mounted
                .validate_binding(trigger.presentation().binding())
                .is_err()
        {
            return 0;
        }
        let changed_instances = trigger.changed_instances();
        let pointers = self
            .pointers
            .iter()
            .filter_map(|(pointer, record)| {
                (record.target.is_none()
                    || record
                        .target
                        .is_some_and(|target| changed_instances.binary_search(&target).is_ok()))
                .then_some(*pointer)
            })
            .collect::<Vec<_>>();
        let mut changed = 0;
        for pointer in pointers {
            let Some(record) = self.pointers.get(&pointer) else {
                continue;
            };
            let resolved = match crate::runtime::interaction::targeting::resolve_presented_target(
                mounted,
                trigger.presentation(),
                record.position,
            ) {
                Ok(target) => Some((
                    target.surface(),
                    target.binding(),
                    target.mounted_instance(),
                    target.node_receipt(),
                )),
                Err(crate::runtime::interaction::targeting::UiInteractionTargetingDenial::NoTarget { .. }) => None,
                Err(_) => continue,
            };
            let kind = record.kind;
            let sequence = record.sequence;
            let position = record.position;
            if self
                .record_pointer_target(
                    pointer,
                    kind,
                    sequence,
                    position,
                    trigger.presentation(),
                    resolved,
                    generation,
                )
                .is_some()
            {
                changed += 1;
            }
        }
        changed
    }

    fn reassign_primary(
        &mut self,
        pointer: UiHostPointerIdentity,
        prior: Option<UiSemanticSurfaceIdentity>,
        current: Option<UiSemanticSurfaceIdentity>,
        kind: UiPrimaryPointerKind,
    ) {
        if prior != current || !primary_pointer_admitted(kind) {
            if let Some(prior) = prior {
                if self.primary_by_surface.get(&prior) == Some(&pointer) {
                    self.primary_by_surface.remove(&prior);
                }
            }
        }
        if primary_pointer_admitted(kind) {
            let Some(current) = current else {
                return;
            };
            self.primary_by_surface.insert(current, pointer);
        }
    }

    pub(crate) fn cancel_binding(
        &mut self,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    ) {
        let selected = self
            .pointers
            .iter()
            .filter_map(|(pointer, record)| (record.binding == Some(binding)).then_some(*pointer))
            .collect::<Vec<_>>();
        let removed = selected
            .into_iter()
            .filter_map(|pointer| {
                self.pointers
                    .remove(&pointer)
                    .map(|record| (pointer, record.surface))
            })
            .collect::<Vec<_>>();
        for (pointer, surface) in &removed {
            if let Some(surface) = surface {
                if self.primary_by_surface.get(surface) == Some(pointer) {
                    self.primary_by_surface.remove(surface);
                }
            }
        }
        if !removed.is_empty() {
            self.bump_revision();
        }
    }

    pub(crate) fn cancel_instance(&mut self, instance: UiMountedInstanceIdentity) {
        let mut changed = false;
        for record in self.pointers.values_mut() {
            if record.target == Some(instance) {
                record.target = None;
                record.node_receipt = None;
                changed = true;
            }
        }
        if changed {
            self.bump_revision();
        }
    }

    pub(crate) fn cancel_all(&mut self) {
        if self.pointers.is_empty() && self.primary_by_surface.is_empty() {
            return;
        }
        self.pointers.clear();
        self.primary_by_surface.clear();
        self.bump_revision();
    }

    fn bump_revision(&mut self) {
        self.revision = self
            .revision
            .checked_add(1)
            .expect("bounded pointer-presence revision exhausted");
    }

    pub(crate) fn appearance_snapshot(&self) -> UiPointerPresenceAppearanceOwnerSnapshot {
        let postures = self
            .pointers
            .iter()
            .map(|(pointer, record)| UiPointerPresenceAppearancePosture {
                pointer: *pointer,
                kind: record.kind,
                presentation: record.presentation,
                target: record.target,
                node_receipt: record.node_receipt,
                class: if record.target.is_some() {
                    UiPointerPresenceClass::Hovered
                } else {
                    UiPointerPresenceClass::Outside
                },
                owner_revision: self.revision,
                observation_sequence: record.sequence,
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        UiPointerPresenceAppearanceOwnerSnapshot {
            owner_revision: self.revision,
            primary_by_surface: self
                .primary_by_surface
                .iter()
                .map(|(surface, pointer)| (*surface, *pointer))
                .collect(),
            postures,
        }
    }
}

#[cfg(test)]
#[path = "owner_tests.rs"]
mod tests;
