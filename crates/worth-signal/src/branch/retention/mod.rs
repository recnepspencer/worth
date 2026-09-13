//! Signal branch retention: bounded obligation accounting over exact targets.
//!
//! Responsibilities are split so that no single file owns both the owner's
//! bookkeeping and the vocabulary callers see:
//!
//! - [`registry`] holds the narrow retention owner, the surviving cleanup
//!   ledger, and the exact-target obligation records.
//! - [`accounting`] holds the obligation counting and terminality recording
//!   that ledger performs.
//! - [`lease`] holds the owner-issued guards: the internal admission lease that
//!   pins an admitted basis, and the external, non-cloneable component lease.
//! - [`outcome`] holds the acquisition, release, and terminality vocabulary.

mod accounting;
mod lease;
mod outcome;
mod registry;

pub use lease::SignalBranchRetentionLease;
pub(crate) use lease::{SignalBranchAdmissionLease, SignalBranchAdmissionReservation};
pub use outcome::{
    SignalBranchRetentionAcquisitionDenial, SignalBranchRetentionOwnerPosture,
    SignalBranchRetentionReleaseDenial, SignalBranchRetentionReleaseOutcome,
    SignalBranchRetentionReleaseReceipt, SignalBranchRetentionTerminalCounts,
    SignalBranchRetentionTerminalOutcome,
};
#[cfg(test)]
pub(crate) use registry::SignalRetentionLedgerObservation;
pub(crate) use registry::{
    SignalBranchRetentionBinding, SignalBranchRetentionOwnerRelationship,
    SignalBranchRetentionRegistry, SignalBranchRetirementRetentionCounts,
    SignalExternalRetentionAcquisition,
};
