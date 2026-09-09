use super::*;
use crate::runtime::interaction::targeting::UiPresentedPointerPosition;

impl UiPointerPresenceOwner {
    pub(crate) fn process_pointer_report(
        &mut self,
        core: UiHostObservationCanonicalCore,
        report: &worth_ui_host_contract::UiHostObservationReport,
        kind: UiPrimaryPointerKind,
        mounted: &crate::mounting::WorthUiMountedSessionState,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> Result<
        Option<UiPointerPresenceTargetTransition>,
        super::super::UiPointerPresenceAdmissionDenial,
    > {
        let (pointer, position) = match report.payload() {
            UiHostObservationPayload::PointerMotion {
                pointer, position, ..
            }
            | UiHostObservationPayload::PointerButton {
                pointer, position, ..
            } => (*pointer, *position),
            _ => return Ok(None),
        };
        self.admit_pointer(pointer, kind)?;
        let resolved = UiPresentedPointerPosition::resolve(mounted, core.presentation(), position)
            .map_err(
                |denial| super::super::UiPointerPresenceAdmissionDenial::Targeting {
                    pointer,
                    denial,
                },
            )?;
        self.record_observation(
            pointer,
            UiPointerPresenceRecord::from_position(kind, report.sequence(), &resolved),
            generation,
        )
    }

    fn record_observation(
        &mut self,
        pointer: UiHostPointerIdentity,
        successor: UiPointerPresenceRecord,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> Result<
        Option<UiPointerPresenceTargetTransition>,
        super::super::UiPointerPresenceAdmissionDenial,
    > {
        self.admit_pointer(pointer, successor.kind)?;
        let prior = self.pointers.get(&pointer);
        let prior_surface = prior.and_then(|record| record.surface);
        let previous = prior.and_then(|record| record.target());
        let previous_node_receipt = prior.and_then(|record| record.node_receipt());
        let record_changed = prior.is_none_or(|record| {
            record.surface != successor.surface
                || record.target() != successor.target()
                || record.binding != successor.binding
                || record.node_receipt() != successor.node_receipt()
                || record.kind != successor.kind
        });
        let prior_was_primary = prior_surface.is_some_and(|surface| {
            prior.is_some_and(|record| primary_pointer_admitted(record.kind))
                && self.primary_by_surface.get(&surface) == Some(&pointer)
        });
        let current_is_primary = primary_pointer_admitted(successor.kind);
        let primary_changed = prior_was_primary
            && (prior_surface != successor.surface || !current_is_primary)
            || current_is_primary
                && successor
                    .surface
                    .is_some_and(|surface| self.primary_by_surface.get(&surface) != Some(&pointer));
        let changed = record_changed || primary_changed;
        if changed {
            self.bump_revision();
        }
        self.reassign_primary(pointer, prior_surface, successor.surface, successor.kind);
        let transition = changed.then(|| UiPointerPresenceTargetTransition {
            generation: generation.clone(),
            pointer,
            previous_surface: prior_surface,
            current_surface: successor.surface,
            previous,
            current: successor.target(),
            previous_node_receipt,
            current_node_receipt: successor.node_receipt(),
            owner_revision: self.revision,
            position: successor.position,
            presentation: successor.presentation,
        });
        self.pointers.insert(pointer, successor);
        Ok(transition)
    }

    #[cfg(test)]
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
    ) -> Result<
        Option<UiPointerPresenceTargetTransition>,
        super::super::UiPointerPresenceAdmissionDenial,
    > {
        let (surface, binding, target, node_receipt) =
            resolved.expect("owner unit fixture supplies target affinity");
        self.record_observation(
            pointer,
            UiPointerPresenceRecord {
                kind,
                surface: Some(surface),
                binding: Some(binding),
                target: Some(
                    crate::runtime::interaction::targeting::interaction_target_view_for_test(
                        presentation,
                        crate::mounting::UiMountedInteractionAffinityInput {
                            surface,
                            binding,
                            mounted_instance: target,
                            node_receipt,
                        },
                    ),
                ),
                sequence,
                position,
                presentation,
            },
            generation,
        )
    }
}

impl UiPointerPresenceRecord {
    fn from_position(
        kind: UiPrimaryPointerKind,
        sequence: UiHostObservationSequence,
        resolved: &UiPresentedPointerPosition,
    ) -> Self {
        Self {
            kind,
            surface: Some(resolved.surface()),
            binding: Some(resolved.presentation().binding()),
            target: resolved.target().map(|target| target.view()),
            sequence,
            position: resolved.position(),
            presentation: resolved.presentation(),
        }
    }
}
