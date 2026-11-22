//! External effects, recovery custody, and canonical recorded correction causality.

pub use correction::OriginalCorrectionDisclosure;
pub use worth_query_execution_lineage::facade::domain_computation::application_aftermath::*;
pub use worth_query_execution_lineage::facade::domain_computation::application_aftermath::{
    correction, correction_coordinates, retained_preimage, retained_relations,
};
pub use worth_query_execution_obligations::facade::aftermath::correction_material;
pub mod commit_causality;
pub use commit_causality::ApplicationCommitCausality;
pub mod causal_fact;
pub use correction_coordinates::WorthQueryCorrectionSourceCoordinates;
pub use worth_query_execution_obligations::facade::aftermath::derivation_failure;
pub mod dispatch_journal;
pub mod external_effect;
pub mod recovery_handle;
use worth_query_execution_obligations::facade::aftermath::recovery_posture;
pub mod recovery_progression;
pub use retained_relations::{
    WorthQueryRelationPreImageDenial, WorthQueryRelationPreImageFailure,
    WorthQueryRetainedPreImageRelation,
};

pub use derivation_failure::WorthQueryAftermathDerivationFailure;

pub use causal_fact::CommittedAftermathCausalityExt;
pub use causal_fact::WorthQueryPendingAftermathCausality;
pub use causal_fact::{
    WorthQueryAftermathCausalRole, WorthQueryCommittedAftermathCausality,
    WorthQueryCorrectionLineageBinding, WorthQueryRecordedCorrectionSource,
};
pub use external_effect::dispatch_external_effect;

pub use external_effect::WorthQueryInboundCleanupReport;
pub use external_effect::{
    bind_dispatch_outbox_create_intent, WorthQueryDispatchOutboxRestoredFields,
    WorthQueryPendingDispatchOutbox,
};
pub use external_effect::{
    derive_external_effect_correlation_identity, ExternalEffectCausalLink,
    ExternalEffectClassification, ExternalEffectCorrelationBasis,
    ExternalEffectCorrelationIdentity, ExternalEffectNoEffectCause, ExternalEffectPosture,
    ExternalEffectPostureIdentity, ExternalEffectPostureKind, ExternalEffectProvenNoEffect,
    ExternalRailTransportFault, WorthQueryDispatchOutboxLayout, WorthQueryDispatchOutboxRecord,
    WorthQueryExternalDispatchCausalRelation, WorthQueryExternalDispatchPosture,
    WorthQueryExternalDispatchPostureKind, WorthQueryExternalDispatchRequest,
    WorthQueryExternalEffectCausalLadder, WorthQueryExternalEffectDispatch,
    WorthQueryExternalEffectTransport, WorthQueryExternalTransportOutcome,
    WorthQueryInboundOccurrenceClaims, WorthQueryInboundOccurrenceVerifier,
    WorthQueryInboundVerificationDenial,
};
pub use external_effect::{
    WorthQueryAcceptedInboundOccurrence, WorthQueryInboundCustody,
    WorthQueryInboundCustodyAdmission, WorthQueryInboundPublicationClaim,
    WorthQueryInboundRecoveryState, WorthQueryInboundTerminalOwnerResult,
    WorthQueryTransportPublicationPermit, WorthQueryTransportPublicationPermitDenial,
};
pub use recovery_posture::{
    WorthQueryDispatchOutboxDurabilityPosture, WorthQueryRecoveryDurabilityPosture,
};
pub use recovery_progression::require_fresh_effect_authority;
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
pub mod inline_type_budget;

pub use recovery_handle::binding::WorthQueryRecoveryHandleBinding;
