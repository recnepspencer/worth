//! Recovery handle progression transitions (Gate 8.3 / R8.30 / Gate 8.7).

mod authority;
mod binding_axis;
mod compensate;
mod disclose;
mod dispose;
mod expiry;
mod inspect;
mod reconcile;
mod redispatch;
mod resolve;
mod safe_retry;

pub use authority::{
    require_fresh_effect_authority, WorthQueryRecoveryEffectAuthority,
    WorthQueryRecoveryInspectAuthority,
};
pub use compensate::{compensate_recovery_handle, WorthQueryRecoveryCompensateAdmission};
pub use disclose::WorthQueryRecoveryDisclosureAdmission;
pub use dispose::{
    dispose_recovery_handle, expire_recovery_handle, WorthQueryRecoveryDisposalReceipt,
};
pub use expiry::{
    WorthQueryRecoveryCurrentDecision, WorthQueryRecoveryExpiryDecision,
    WorthQueryRecoveryExpiryEvaluation,
};
pub use inspect::{inspect_recovery_handle, WorthQueryRecoveryInspectionView};
pub use reconcile::{reconcile_recovery_handle, WorthQueryRecoveryReconcileAdmission};
pub use redispatch::WorthQueryPerformedExternalRedispatch;
pub use resolve::{resolve_recovery_handle, WorthQueryAdmittedIdempotencyRead};
pub use safe_retry::{
    safe_retry_recovery_handle, WorthQueryRecoverySafeRetryAdmission,
    WorthQueryRecoverySafeRetryDenial,
};

#[cfg(test)]
#[path = "resolve_tests.rs"]
mod resolve_tests;

#[cfg(test)]
mod authority_tests;

#[cfg(test)]
mod mechanism_tests;
