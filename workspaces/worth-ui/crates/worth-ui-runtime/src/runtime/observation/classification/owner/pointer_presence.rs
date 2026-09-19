use crate::fact_contract::{UiPointerPresenceTargetChangedFact, UiProducedFact};

pub(in crate::runtime::observation::classification) fn classify(
    transition: crate::runtime::interaction::UiPointerPresenceTargetTransition,
) -> UiProducedFact {
    UiProducedFact::PointerPresenceTarget(
        UiPointerPresenceTargetChangedFact::from_owner_transition(transition),
    )
}
