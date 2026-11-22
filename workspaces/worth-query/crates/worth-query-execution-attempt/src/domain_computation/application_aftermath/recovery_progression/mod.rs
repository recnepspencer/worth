//! Recovery handle progression transitions (Gate 8.3 / R8.30 / Gate 8.7).

pub mod authority;
pub mod binding_axis;
pub mod compensate;
pub mod dispose;
pub mod inspect;
pub mod reconcile;
pub mod redispatch;
pub mod resolve;
pub mod safe_retry;

pub use authority::{
    require_fresh_effect_authority, WorthQueryRecoveryEffectAuthority,
    WorthQueryRecoveryInspectAuthority,
};
pub use compensate::{compensate_recovery_handle, WorthQueryRecoveryCompensateAdmission};
pub use dispose::{
    dispose_recovery_handle, expire_recovery_handle, WorthQueryRecoveryDisposalDenial,
    WorthQueryRecoveryDisposalReceipt,
};
pub use inspect::{inspect_recovery_handle, WorthQueryRecoveryInspectionView};
pub use reconcile::{reconcile_recovery_handle, WorthQueryRecoveryReconcileAdmission};
pub use redispatch::WorthQueryPerformedExternalRedispatch;
pub use resolve::{resolve_recovery_handle, WorthQueryAdmittedIdempotencyRead};
pub use safe_retry::{
    safe_retry_recovery_handle, WorthQueryRecoverySafeRetryAdmission,
    WorthQueryRecoverySafeRetryDenial,
};

pub mod expiry;
pub use expiry::{
    deny_if_expired, evaluate_expiry, WorthQueryRecoveryCurrentDecision,
    WorthQueryRecoveryExpiryDecision, WorthQueryRecoveryExpiryEvaluation,
};
