use crate::runtime::rebind::*;

/// Theme settlement preserves the existing rebind receipts and continuation handles.
pub enum UiThemeSwitchOutcome<'session> {
    Duplicate(UiDuplicateObservationReceipt),
    ObservedNoChange(crate::runtime::observation::UiObservedNoChangeReceipt),
    RejectedBeforeEffects(UiRebindDenialReceipt<'session>),
    CancelledBeforeEffects(UiRebindCancellationReceipt),
    TimedOutBeforeEffects(UiRebindTimeoutReceipt),
    SupersededBeforeEffects(UiRebindSupersededReceipt),
    Published(UiRebindReceipt),
    InFlight(UiRebindCompletionHandle<'session>),
    Indeterminate(UiRebindRecoveryHandle<'session>),
    InternalDefect(UiRebindInternalDefectOutcome),
}

impl<'session> From<UiRebindOutcome<'session>> for UiThemeSwitchOutcome<'session> {
    fn from(outcome: UiRebindOutcome<'session>) -> Self {
        match outcome {
            UiRebindOutcome::Duplicate(receipt) => Self::Duplicate(receipt),
            UiRebindOutcome::ObservedNoChange(receipt) => Self::ObservedNoChange(receipt),
            UiRebindOutcome::RejectedBeforeEffects(receipt) => Self::RejectedBeforeEffects(receipt),
            UiRebindOutcome::CancelledBeforeEffects(receipt) => {
                Self::CancelledBeforeEffects(receipt)
            }
            UiRebindOutcome::TimedOutBeforeEffects(receipt) => Self::TimedOutBeforeEffects(receipt),
            UiRebindOutcome::SupersededBeforeEffects(receipt) => {
                Self::SupersededBeforeEffects(receipt)
            }
            UiRebindOutcome::Published(receipt) => Self::Published(receipt),
            UiRebindOutcome::InFlight(handle) => Self::InFlight(handle),
            UiRebindOutcome::Indeterminate(handle) => Self::Indeterminate(handle),
            UiRebindOutcome::InternalDefect(outcome) => Self::InternalDefect(outcome),
        }
    }
}
