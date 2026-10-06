use std::marker::PhantomData;

use super::{ApplicationInboundOccurrenceLimits, ApplicationInboundOccurrenceProtocol};

/// Declared completion meaning for the exact outbound effect selected by an
/// operation builder. This carries no live verification or completion authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationInboundOccurrenceBinding<Effect> {
    protocol: ApplicationInboundOccurrenceProtocol,
    source_identity: String,
    limits: ApplicationInboundOccurrenceLimits,
    _effect: PhantomData<fn() -> Effect>,
}

impl<Effect> ApplicationInboundOccurrenceBinding<Effect> {
    pub fn new(
        protocol: ApplicationInboundOccurrenceProtocol,
        source_identity: impl Into<String>,
        limits: ApplicationInboundOccurrenceLimits,
    ) -> Option<Self> {
        let source_identity = source_identity.into();
        if source_identity.trim().is_empty() || !limits.accommodates_payload() {
            return None;
        }
        Some(Self {
            protocol,
            source_identity,
            limits,
            _effect: PhantomData,
        })
    }

    pub const fn protocol(&self) -> &ApplicationInboundOccurrenceProtocol {
        &self.protocol
    }

    pub fn source_identity(&self) -> &str {
        &self.source_identity
    }

    pub const fn limits(&self) -> ApplicationInboundOccurrenceLimits {
        self.limits
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        ApplicationInboundOccurrenceProtocol,
        String,
        ApplicationInboundOccurrenceLimits,
    ) {
        (self.protocol, self.source_identity, self.limits)
    }
}
