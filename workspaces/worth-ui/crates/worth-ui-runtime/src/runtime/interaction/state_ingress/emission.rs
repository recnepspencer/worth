use worth_ui_host_contract::UiHostObservationCanonicalCore;

use crate::runtime::interaction::gesture::UiPointerGestureOutcome;
use crate::runtime::interaction::{
    UiActivateInteraction, UiDismissInteraction, UiInteractionStop, UiInteractionTransition,
    UiSemanticInteraction,
};
use crate::runtime::WorthUiActiveApplicationGenerationIdentity;

use super::super::UiInteractionRuntimeState;

pub(super) fn emit_pointer(
    state: &mut UiInteractionRuntimeState,
    outcomes: Vec<UiPointerGestureOutcome>,
    core: UiHostObservationCanonicalCore,
    generation: &WorthUiActiveApplicationGenerationIdentity,
) -> Vec<UiInteractionTransition> {
    let mut transitions = Vec::new();
    for outcome in outcomes {
        match outcome {
            UiPointerGestureOutcome::Pressed(press) => {
                let dismissal = UiDismissInteraction::outside_press(
                    core.presentation(),
                    press.sequence(),
                    press.time_basis(),
                    press.position(),
                );
                transitions.push(UiInteractionTransition::PointerPressed(press));
                transitions.push(UiInteractionTransition::DismissRequested(dismissal));
            }
            UiPointerGestureOutcome::Completed(gesture) => {
                if gesture.pointer_device_kind()
                    == worth_ui_host_contract::UiHostPointerDeviceKind::Touch
                {
                    continue;
                }
                state.record_semantic();
                transitions.push(UiInteractionTransition::Semantic(
                    UiSemanticInteraction::Activate(UiActivateInteraction::from_pointer(
                        gesture,
                        generation.clone(),
                    )),
                ));
            }
            UiPointerGestureOutcome::Stopped(stop) => {
                transitions.push(UiInteractionTransition::Stopped(
                    UiInteractionStop::PointerGesture(stop),
                ));
            }
        }
    }
    transitions
}

pub(super) fn emit_draft(
    state: &mut UiInteractionRuntimeState,
    outcomes: Vec<crate::runtime::interaction::draft::UiDraftProcessingOutcome>,
) -> Vec<UiInteractionTransition> {
    outcomes
        .into_iter()
        .map(|outcome| match outcome {
            crate::runtime::interaction::draft::UiDraftProcessingOutcome::Mutation(receipt) => {
                UiInteractionTransition::DraftMutation(receipt)
            }
            crate::runtime::interaction::draft::UiDraftProcessingOutcome::DismissRequested(
                interaction,
            ) => UiInteractionTransition::DismissRequested(interaction),
            crate::runtime::interaction::draft::UiDraftProcessingOutcome::Semantic(interaction) => {
                state.record_semantic();
                UiInteractionTransition::Semantic(interaction)
            }
            crate::runtime::interaction::draft::UiDraftProcessingOutcome::Stopped(stop) => {
                UiInteractionTransition::Stopped(UiInteractionStop::LocalInput(stop))
            }
        })
        .collect()
}
