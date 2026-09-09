use super::state::{UiIntentConfirmationSlotState, UiIntentConfirmationState};

pub(super) fn matching_slots(
    state: &UiIntentConfirmationState,
    declaration: &str,
    definition: crate::capability::UiIntentId,
) -> (Vec<usize>, Vec<usize>, usize) {
    let mut pending = Vec::new();
    let mut terminal = Vec::new();
    for (index, slot) in state.slots.iter().enumerate() {
        match &slot.state {
            UiIntentConfirmationSlotState::Pending(challenge)
                if challenge.candidate.declaration_identity() == declaration
                    && challenge.candidate.definition_id() == definition =>
            {
                pending.push(index);
            }
            UiIntentConfirmationSlotState::Terminal(marker)
                if marker.declaration.as_ref() == declaration
                    && marker.definition == definition =>
            {
                terminal.push(index);
            }
            UiIntentConfirmationSlotState::Vacant
            | UiIntentConfirmationSlotState::Pending(_)
            | UiIntentConfirmationSlotState::Terminal(_) => {}
        }
    }
    (pending, terminal, state.slots.len())
}
