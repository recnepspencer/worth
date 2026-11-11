//! External-effect causality: postures, correlation, outbox, classification.
//!
//! CDC is intentionally not the delivery substrate (R8.8). Dispatch work is
//! derived from Query-owned outbox records co-committed with the mutation.

pub(crate) mod causal_event;
pub(crate) mod classification;
pub(crate) mod correlation;
pub(crate) mod dispatch;
mod identity;
mod identity_derivation;
pub(crate) mod inbound;
mod observation;
pub(crate) mod outbox;
mod posture;
mod proven_no_effect;
mod transport;

pub use causal_event::ExternalEffectPosture;
pub(crate) mod dispatch_admission;
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
