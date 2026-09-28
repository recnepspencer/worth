use crate::runtime::{
    WorthUiNodeLifecycleTransition, WorthUiQueryLiveRebindEntry, WorthUiQueryLiveRebindOutcome,
};

pub(super) fn transition(entry: &WorthUiQueryLiveRebindEntry) -> WorthUiNodeLifecycleTransition {
    match entry.outcome() {
        WorthUiQueryLiveRebindOutcome::Preserve(_) => WorthUiNodeLifecycleTransition::Preserve,
        WorthUiQueryLiveRebindOutcome::Rebind(rebind)
            if rebind.reason()
                == crate::runtime::WorthUiQueryBindingRebindReason::FreshCandidateBinding =>
        {
            WorthUiNodeLifecycleTransition::Create
        }
        WorthUiQueryLiveRebindOutcome::Rebind(_) => WorthUiNodeLifecycleTransition::Rebind,
        WorthUiQueryLiveRebindOutcome::Retire(_) | WorthUiQueryLiveRebindOutcome::Deny(_) => {
            WorthUiNodeLifecycleTransition::Drop
        }
    }
}

pub(super) fn preservation_receipt(
    entry: &WorthUiQueryLiveRebindEntry,
) -> Option<crate::runtime::WorthUiQueryBindingPreservationReceipt> {
    match entry.outcome() {
        WorthUiQueryLiveRebindOutcome::Preserve(value) => Some(value.preservation_receipt()),
        WorthUiQueryLiveRebindOutcome::Rebind(_)
        | WorthUiQueryLiveRebindOutcome::Retire(_)
        | WorthUiQueryLiveRebindOutcome::Deny(_) => None,
    }
}
