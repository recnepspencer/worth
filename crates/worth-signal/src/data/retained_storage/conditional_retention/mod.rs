//! Signal-owned allocation custody. Policy issuance remains on SignalOwner.
mod ledger;
mod measurement;
pub use ledger::SignalConditionalRetentionObservation;
pub(crate) use ledger::{
    SignalConditionalRetentionDenial, SignalConditionalRetentionLedger,
    SignalConditionalRetentionReservation,
};
#[cfg(test)]
#[path = "mod_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
