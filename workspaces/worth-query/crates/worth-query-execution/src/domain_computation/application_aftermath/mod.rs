//! External effects, recovery custody, and canonical recorded correction causality.

mod correction;
pub(crate) mod correction_material;
pub(in crate::domain_computation) use correction::OriginalCorrectionDisclosure;
pub(crate) mod commit_causality;
pub(crate) use commit_causality::ApplicationCommitCausality;
pub(crate) mod causal_fact;
mod correction_coordinates;
pub use correction_coordinates::WorthQueryCorrectionSourceCoordinates;
pub mod derivation_failure;
pub mod external_effect;
pub mod recovery_handle;
mod recovery_posture;
pub mod recovery_progression;
pub mod retained_preimage;
mod retained_relations;
pub use retained_relations::{
    WorthQueryRelationPreImageDenial, WorthQueryRelationPreImageFailure,
    WorthQueryRetainedPreImageRelation,
};

pub use derivation_failure::WorthQueryAftermathDerivationFailure;

pub(crate) use causal_fact::WorthQueryPendingAftermathCausality;
pub use causal_fact::{
    WorthQueryAftermathCausalRole, WorthQueryCommittedAftermathCausality,
    WorthQueryCorrectionLineageBinding, WorthQueryRecordedCorrectionSource,
};
pub(in crate::domain_computation) use external_effect::dispatch_external_effect;

pub use external_effect::WorthQueryInboundCleanupReport;
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
    WorthQueryExternalTransportOutcome, WorthQueryInboundOccurrenceClaims,
    WorthQueryInboundOccurrenceVerifier, WorthQueryInboundVerificationDenial,
};
pub(in crate::domain_computation) use external_effect::{
    WorthQueryAcceptedInboundOccurrence, WorthQueryInboundCustody,
    WorthQueryInboundCustodyAdmission, WorthQueryInboundPublicationClaim,
    WorthQueryInboundRecoveryState, WorthQueryInboundTerminalOwnerResult,
    WorthQueryTransportPublicationPermit, WorthQueryTransportPublicationPermitDenial,
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
    WorthQueryRecoveryDisposalDenial, WorthQueryRecoveryDisposalReceipt,
    WorthQueryRecoveryEffectAuthority, WorthQueryRecoveryExpiryDecision,
    WorthQueryRecoveryExpiryEvaluation, WorthQueryRecoveryInspectAuthority,
    WorthQueryRecoveryInspectionView, WorthQueryRecoveryReconcileAdmission,
    WorthQueryRecoverySafeRetryAdmission, WorthQueryRecoverySafeRetryDenial,
};
pub use retained_preimage::{
    demanded_field_slot, WorthQueryPreImageRetentionDenial, WorthQueryRetainedPreImage,
};
// `WorthQueryPreImageRetentionDenial` is re-exported as of slice 10: the
// provider session now maps each retention denial onto a refused commit
// (Q8.26-C2), so it is consumed by name. `WorthQueryRetainedPreImageField`
// remains reachable only through `retained_preimage` (a `pub mod`) — nothing
// consumes it by name, and this phase does not export for consumers that do
// not exist.

#[cfg(test)]
#[path = "mod_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
#[cfg(test)]
mod retained_preimage_tests;
