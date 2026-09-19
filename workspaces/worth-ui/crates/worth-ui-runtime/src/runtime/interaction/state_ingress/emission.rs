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
    generation: &WorthUiActiveApplicationGenerationIdentity,
) -> Vec<UiInteractionTransition> {
    let mut transitions = Vec::new();
    for outcome in outcomes {
        match outcome {
            UiPointerGestureOutcome::Pressed(press) => {
                transitions.push(UiInteractionTransition::PointerPressed(press));
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

pub(super) fn outside_press(
    core: UiHostObservationCanonicalCore,
    report: &worth_ui_host_contract::UiHostObservationReport,
) -> Option<UiInteractionTransition> {
    let worth_ui_host_contract::UiHostObservationPayload::PointerButton {
        transition: worth_ui_host_contract::UiHostPointerButtonTransition::Pressed,
        position,
        ..
    } = report.payload()
    else {
        return None;
    };
    Some(UiInteractionTransition::DismissRequested(
        UiDismissInteraction::outside_press(
            core.presentation(),
            report.sequence(),
            report.time_basis(),
            *position,
        ),
    ))
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
