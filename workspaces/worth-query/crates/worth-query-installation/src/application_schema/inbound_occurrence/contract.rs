use worth_query_declaration::facade::application_schema::{
    ApplicationInboundOccurrenceLimits, ApplicationInboundOccurrenceProtocol,
};

/// Installed immutable support for one operation's declared outbound effect.
/// Live verifier keys, time, admission and custody are owned by the host and
/// execution runtime, never by this portable contract.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct InstalledInboundOccurrenceContract {
    effect: String,
    protocol: ApplicationInboundOccurrenceProtocol,
    source_identity: String,
    limits: ApplicationInboundOccurrenceLimits,
}

impl InstalledInboundOccurrenceContract {
    pub(super) fn new(
        effect: String,
        protocol: ApplicationInboundOccurrenceProtocol,
        source_identity: String,
        limits: ApplicationInboundOccurrenceLimits,
    ) -> Self {
        Self {
            effect,
            protocol,
            source_identity,
            limits,
        }
    }

    pub fn effect(&self) -> &str {
        &self.effect
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
}
