//! External-effect causality: postures, correlation, outbox, classification.
//!
//! CDC is intentionally not the delivery substrate (R8.8). Dispatch work is
//! derived from Query-owned outbox records co-committed with the mutation.

mod causal_event;
pub(crate) mod classification;
mod correlation;
mod dispatch;
mod identity;
mod identity_derivation;
mod inbound;
mod observation;
mod outbox;
mod posture;
mod transport;

#[cfg(test)]
pub(crate) mod tests;

pub use causal_event::ExternalEffectPosture;
pub(in crate::domain_computation) use causal_event::WorthQueryAdmittedExternalDispatchAttempt;
pub use classification::{ExternalEffectClassification, ExternalRailTransportFault};
pub use correlation::{
    derive_external_effect_correlation_identity, ExternalEffectCorrelationBasis,
    ExternalEffectCorrelationIdentity,
};
pub(in crate::domain_computation) use dispatch::dispatch_external_effect;
pub use dispatch::{
    WorthQueryExternalDispatchCausalRelation, WorthQueryExternalDispatchPosture,
    WorthQueryExternalDispatchPostureKind, WorthQueryExternalEffectCausalLadder,
    WorthQueryExternalEffectDispatch,
};
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
#[cfg(test)]
pub(crate) use outbox::dispatch_outbox_create_intent;
pub(crate) use outbox::{
    bind_dispatch_outbox_create_intent, WorthQueryDispatchOutboxRestoredFields,
    WorthQueryPendingDispatchOutbox,
};
pub use outbox::{WorthQueryDispatchOutboxLayout, WorthQueryDispatchOutboxRecord};
pub use posture::ExternalEffectPostureKind;
pub use transport::{
    WorthQueryExternalDispatchRequest, WorthQueryExternalEffectTransport,
    WorthQueryExternalTransportOutcome,
};
