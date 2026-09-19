//! External-effect causality postures, dispatch outbox, production dispatch,
//! recovery-handle lifecycle (Gates 8.2 / 8.3), fresh undo (8.4), and fresh
//! redo intent with Relational-owned branch lineage (8.5).

mod causal_commit;
mod causal_fact;
pub mod derivation_failure;
pub mod external_effect;
mod governed_input;
pub mod recovery_handle;
mod recovery_posture;
pub mod recovery_progression;
pub mod redo_admission;
pub mod redo_denial;
pub mod redo_intent;
pub mod redo_progression;
pub mod redo_recovery;
pub mod retained_preimage;
pub mod undo_admission;
pub mod undo_denial;
mod undo_evidence;
pub mod undo_intent;
pub mod undo_progression;

pub use derivation_failure::WorthQueryAftermathDerivationFailure;

pub(crate) use causal_fact::WorthQueryPendingAftermathCausality;
pub use causal_fact::{WorthQueryAftermathCausalRole, WorthQueryCommittedAftermathCausality};
pub(in crate::domain_computation) use external_effect::dispatch_external_effect;
#[cfg(test)]
pub(crate) use external_effect::dispatch_outbox_create_intent;
pub(crate) use external_effect::{
    bind_dispatch_outbox_create_intent, WorthQueryDispatchOutboxRestoredFields,
    WorthQueryPendingDispatchOutbox,
};
pub use external_effect::{
    derive_external_effect_correlation_identity, ExternalEffectCausalLink,
    ExternalEffectClassification, ExternalEffectCorrelationBasis,
    ExternalEffectCorrelationIdentity, ExternalEffectPosture, ExternalEffectPostureIdentity,
    ExternalEffectPostureKind, ExternalRailTransportFault, WorthQueryDispatchOutboxLayout,
    WorthQueryDispatchOutboxRecord, WorthQueryExternalDispatchCausalRelation,
    WorthQueryExternalDispatchPosture, WorthQueryExternalDispatchPostureKind,
    WorthQueryExternalDispatchRequest, WorthQueryExternalEffectCausalLadder,
    WorthQueryExternalEffectDispatch, WorthQueryExternalEffectTransport,
    WorthQueryExternalTransportOutcome,
};
pub use recovery_handle::{
    WorthQueryOpaqueRecoveryWireIdentity, WorthQueryRecoveryClaimStatus, WorthQueryRecoveryHandle,
    WorthQueryRecoveryHandleBinding, WorthQueryRecoveryHandleDenial,
    WorthQueryRecoveryHandleDenialKind,
};
pub use recovery_posture::{
    WorthQueryDispatchOutboxDurabilityPosture, WorthQueryRecoveryDurabilityPosture,
};
pub(crate) use recovery_progression::require_fresh_effect_authority;
pub use recovery_progression::{
    compensate_recovery_handle, dispose_recovery_handle, expire_recovery_handle,
    inspect_recovery_handle, reconcile_recovery_handle, resolve_recovery_handle,
    safe_retry_recovery_handle, WorthQueryAdmittedIdempotencyRead,
    WorthQueryPerformedExternalRedispatch, WorthQueryRecoveryCompensateAdmission,
    WorthQueryRecoveryCurrentDecision, WorthQueryRecoveryDisclosureAdmission,
    WorthQueryRecoveryDisposalReceipt, WorthQueryRecoveryEffectAuthority,
    WorthQueryRecoveryExpiryDecision, WorthQueryRecoveryExpiryEvaluation,
    WorthQueryRecoveryInspectAuthority, WorthQueryRecoveryInspectionView,
    WorthQueryRecoveryReconcileAdmission, WorthQueryRecoverySafeRetryAdmission,
    WorthQueryRecoverySafeRetryDenial,
};
pub use redo_admission::WorthQueryRedoAdmission;
pub use redo_denial::{WorthQueryRedoDenial, WorthQueryRedoDenialKind};
pub use redo_intent::{WorthQueryProvedUndo, WorthQueryRedoIntent, WorthQueryRedoIntentIdentity};
pub use redo_progression::{
    consume_redo_progression, map_ordinary_commit_conflict_to_redo, progress_admitted_redo,
    WorthQueryRedoProgressionHandoff,
};
pub use redo_recovery::WorthQueryRedoRecovery;
pub use retained_preimage::{
    demanded_field_slot, WorthQueryPreImageRetentionDenial, WorthQueryRetainedPreImage,
};
pub use undo_admission::{
    admit_undo, deny_irreversible_undo_attempt, WorthQueryUndoAdmission,
    WorthQueryUndoDerivedRequest,
};
pub use undo_denial::{WorthQueryUndoDenial, WorthQueryUndoDenialKind};
pub use undo_intent::WorthQueryUndoIntentIdentity;
// `WorthQueryPreImageRetentionDenial` is re-exported as of slice 10: the
// provider session now maps each retention denial onto a refused commit
// (Q8.26-C2), so it is consumed by name. `WorthQueryRetainedPreImageField`
// remains reachable only through `retained_preimage` (a `pub mod`) — nothing
// consumes it by name, and this phase does not export for consumers that do
// not exist.
pub use undo_progression::{
    consume_unresolved_undo_progression, map_ordinary_commit_conflict,
    progress_admitted_reconciliation, progress_admitted_undo, WorthQueryUndoProgressionHandoff,
};

#[cfg(test)]
pub(crate) mod aftermath_schema_fixture;
#[cfg(test)]
mod redo_admission_tests;
#[cfg(test)]
mod redo_intent_tests;
#[cfg(test)]
mod retained_preimage_tests;
#[cfg(test)]
mod undo_admission_tests;
