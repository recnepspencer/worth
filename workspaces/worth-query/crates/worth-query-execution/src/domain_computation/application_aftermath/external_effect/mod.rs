//! External-effect causality: re-exports pure postures, correlation, outbox, and
//! dispatch types from `worth-query-execution-lineage` and retains runtime dispatch
//! admission and inbound custody in `worth-query-execution`.

pub(crate) use worth_query_execution_lineage::facade::domain_computation::application_aftermath::external_effect::{
    correlation, identity, outbox, posture, transport,
};
pub(crate) mod causal_event;
pub(crate) mod classification;
pub(crate) mod proven_no_effect;
pub(crate) mod dispatch {
    pub use worth_query_execution_lineage::facade::domain_computation::application_aftermath::external_effect::dispatch::*;
    pub use super::dispatch_execution::dispatch_external_effect;
}
mod identity_derivation;
pub(crate) mod inbound;
mod observation;
mod recorded_completion;
pub(in crate::domain_computation) use recorded_completion::RecordedCompletionCausality;

pub use causal_event::ExternalEffectPosture;
pub(crate) mod dispatch_admission;
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
pub(in crate::domain_computation) use dispatch_admission::WorthQueryAdmittedExternalDispatchAttempt;
pub use identity::{ExternalEffectCausalLink, ExternalEffectPostureIdentity};
pub(in crate::domain_computation) use inbound::{
    WorthQueryAcceptedInboundOccurrence, WorthQueryInboundCustody,
    WorthQueryInboundCustodyAdmission, WorthQueryInboundPublicationClaim,
    WorthQueryInboundRecoveryState, WorthQueryInboundTerminalOwnerResult,
    WorthQueryTransportPublicationPermit, WorthQueryTransportPublicationPermitDenial,
};
pub use inbound::{
    WorthQueryInboundCleanupReport, WorthQueryInboundOccurrenceClaims,
    WorthQueryInboundOccurrenceVerifier, WorthQueryInboundVerificationDenial,
};

pub(crate) use outbox::{
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
mod dispatch_execution;
pub(in crate::domain_computation) use dispatch_execution::dispatch_external_effect;
