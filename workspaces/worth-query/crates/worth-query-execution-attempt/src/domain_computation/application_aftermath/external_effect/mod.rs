//! External-effect causality: re-exports pure postures, correlation, outbox, and
//! dispatch types from `worth-query-execution-lineage` and retains runtime dispatch
//! admission and inbound custody in `worth-query-execution`.

pub mod dispatch_admission;
pub use dispatch_admission::WorthQueryAdmittedExternalDispatchAttempt;
pub mod causal_event;
pub use causal_event::ExternalEffectPosture;
pub use worth_query_execution_lineage::facade::domain_computation::application_aftermath::external_effect::{
    correlation, identity, outbox, posture, transport,
};
pub mod classification;
pub mod proven_no_effect;
pub mod dispatch {
    pub use worth_query_execution_lineage::facade::domain_computation::application_aftermath::external_effect::dispatch::*;
    pub use super::dispatch_execution::dispatch_external_effect;
}
pub mod identity_derivation;
pub mod inbound;
pub mod observation;
pub mod recorded_completion;
pub use recorded_completion::RecordedCompletionCausality;

pub use classification::{ExternalEffectClassification, ExternalRailTransportFault};
pub use correlation::{
    derive_external_effect_correlation_identity, ExternalEffectCorrelationBasis,
    ExternalEffectCorrelationIdentity,
};
pub use dispatch::{
    WorthQueryExternalDispatchCausalRelation, WorthQueryExternalDispatchPosture,
    WorthQueryExternalDispatchPostureKind, WorthQueryExternalEffectCausalLadder,
    WorthQueryExternalEffectDispatch,
};
pub use identity::{ExternalEffectCausalLink, ExternalEffectPostureIdentity};
pub use inbound::{
    WorthQueryAcceptedInboundOccurrence, WorthQueryInboundCustody,
    WorthQueryInboundCustodyAdmission, WorthQueryInboundPublicationClaim,
    WorthQueryTransportPublicationPermit, WorthQueryTransportPublicationPermitDenial,
};
pub use inbound::{
    WorthQueryInboundCleanupReport, WorthQueryInboundOccurrenceClaims,
    WorthQueryInboundOccurrenceVerifier, WorthQueryInboundVerificationDenial,
};

pub use outbox::{
    bind_dispatch_outbox_create_intent, WorthQueryDispatchOutboxRestoredFields,
    WorthQueryPendingDispatchOutbox,
};
pub use outbox::{WorthQueryDispatchOutboxLayout, WorthQueryDispatchOutboxRecord};
pub use posture::ExternalEffectPostureKind;
pub use proven_no_effect::{ExternalEffectNoEffectCause, ExternalEffectProvenNoEffect};
pub use transport::{
    WorthQueryExternalDispatchRequest, WorthQueryExternalEffectTransport,
    WorthQueryExternalTransportOutcome,
};

#[path = "dispatch/execution.rs"]
pub mod dispatch_execution;
pub use dispatch_execution::dispatch_external_effect;

pub use inbound::{WorthQueryInboundRecoveryState, WorthQueryInboundTerminalOwnerResult};

