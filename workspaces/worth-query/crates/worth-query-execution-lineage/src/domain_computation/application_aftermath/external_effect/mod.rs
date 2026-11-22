//! External-effect causality: pure postures, correlation, outbox, and dispatch models.

pub mod causal_event;
pub mod classification;
pub mod correlation;
pub mod dispatch;
pub mod identity;
pub mod identity_derivation;
pub mod outbox;
pub mod posture;
pub mod proven_no_effect;
pub mod transport;

pub use causal_event::ExternalEffectPosture;
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
