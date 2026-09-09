use super::matching::matching_slots;
use super::state::{UiIntentConfirmationSlotState, UiIntentConfirmationState};
use super::validation::{
    validate_challenge, UiIntentConfirmationReadContext, UiIntentConfirmationSourceBasis,
};
use super::{UiIntentConfirmationLookupCost, UiIntentConfirmationStopReason};

/// Eligibility at an explicit host observation time, never confirmation authority.
/// Observing cannot consume a challenge, retain its payload, or reserve occupancy.
#[derive(Debug)]
pub struct UiIntentConfirmationObservation {
    stop: Option<UiIntentConfirmationStopReason>,
    cost: UiIntentConfirmationLookupCost,
    expiry_wake_millis: Option<u64>,
}

pub(crate) fn observe_confirmation(
    state: &UiIntentConfirmationState,
    target: crate::runtime::interaction::UiPresentedInteractionTargetView,
    time_basis: worth_ui_host_contract::UiHostObservationTimeBasis,
    context: UiIntentConfirmationReadContext<'_>,
) -> UiIntentConfirmationObservation {
    if context
        .mounted
        .current_semantic_surface_for_presentation(target.presentation())
        .is_err()
    {
        return UiIntentConfirmationObservation::new(
            Some(UiIntentConfirmationStopReason::ConfirmationPresentationStale),
            0,
        );
    }
    let affinity =
        match crate::runtime::interaction::targeting::admit_current_target(context.mounted, target)
        {
            Ok(affinity) => affinity,
            Err(denial) => {
                return UiIntentConfirmationObservation::new(
                    Some(UiIntentConfirmationStopReason::ConfirmationTargetChanged(
                        denial,
                    )),
                    0,
                );
            }
        };
    let Some((
        crate::declaration::UiIntentCatalogResolvedRoute::Confirmation { declaration, .. },
        _,
    )) = context.catalog.lookup(
        affinity.graph_node(),
        crate::capability::UiSemanticInteractionFamily::Activate,
    )
    else {
        return UiIntentConfirmationObservation::new(
            Some(UiIntentConfirmationStopReason::ConfirmationRouteChanged),
            0,
        );
    };
    let definition = context
        .definitions
        .definition_at(declaration.definition())
        .id();
    let identity = declaration.identity().as_str();
    let (pending, terminal, inspected) = matching_slots(state, identity, definition);
    let stop = match pending.as_slice() {
        [slot] => {
            let UiIntentConfirmationSlotState::Pending(challenge) = &state.slots[*slot].state
            else {
                unreachable!("selected confirmation slot remains immutably borrowed")
            };
            validate_challenge(
                challenge,
                definition,
                &declaration,
                UiIntentConfirmationSourceBasis {
                    generation: context.generation,
                    target,
                    time_basis,
                },
                &context,
            )
        }
        [] => Some(terminal_reason(state, identity, &terminal)),
        multiple => Some(UiIntentConfirmationStopReason::AmbiguousPendingChallenges {
            declaration: identity.into(),
            observed: multiple.len(),
        }),
    };
    let expiry_wake_millis = if stop.is_none() {
        let UiIntentConfirmationSlotState::Pending(challenge) = &state.slots[pending[0]].state
        else {
            unreachable!("eligible observation retains a pending challenge")
        };
        challenge.expires_at_millis.checked_add(1)
    } else {
        None
    };
    let mut observation = UiIntentConfirmationObservation::new(stop, inspected);
    observation.expiry_wake_millis = expiry_wake_millis;
    observation
}

fn terminal_reason(
    state: &UiIntentConfirmationState,
    declaration: &str,
    terminal: &[usize],
) -> UiIntentConfirmationStopReason {
    match terminal {
        [slot] => {
            let UiIntentConfirmationSlotState::Terminal(marker) = &state.slots[*slot].state else {
                unreachable!("selected confirmation terminal remains immutably borrowed")
            };
            marker.stop_reason()
        }
        [] => UiIntentConfirmationStopReason::NoPendingChallenge {
            declaration: declaration.into(),
        },
        multiple => UiIntentConfirmationStopReason::AmbiguousPendingChallenges {
            declaration: declaration.into(),
            observed: multiple.len(),
        },
    }
}

impl UiIntentConfirmationObservation {
    fn new(stop: Option<UiIntentConfirmationStopReason>, slots_inspected: usize) -> Self {
        Self {
            stop,
            cost: UiIntentConfirmationLookupCost::new(slots_inspected),
            expiry_wake_millis: None,
        }
    }

    pub const fn expiry_wake_millis(&self) -> Option<u64> {
        self.expiry_wake_millis
    }

    pub const fn is_eligible(&self) -> bool {
        self.stop.is_none()
    }

    pub const fn stop_reason(&self) -> Option<&UiIntentConfirmationStopReason> {
        self.stop.as_ref()
    }

    pub const fn cost(&self) -> UiIntentConfirmationLookupCost {
        self.cost
    }
}
