use super::super::{ApplicationExternalEffectProtocol, WorthQueryExternalEffectCorrelationFamily};
use super::super::{ApplicationInboundOccurrenceLimits, ApplicationInboundOccurrenceProtocol};
use crate::portable_identity::WorthQueryPortableTypeIdentity;

pub(super) struct DeclaredInboundOccurrenceSlot {
    pub(super) protocol: ApplicationInboundOccurrenceProtocol,
    pub(super) source_identity: String,
    pub(super) limits: ApplicationInboundOccurrenceLimits,
}

pub(super) struct DeclaredExternalEffectSlot {
    pub(super) effect: String,
    pub(super) rust_payload_type: WorthQueryPortableTypeIdentity,
    pub(super) protocol: ApplicationExternalEffectProtocol,
    pub(super) maximum_payload_bytes: u64,
    pub(super) correlation_family: WorthQueryExternalEffectCorrelationFamily,
    pub(super) inbound: Option<DeclaredInboundOccurrenceSlot>,
}
