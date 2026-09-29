use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};

/// Stable wire meaning accepted as evidence of one declared outbound effect.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ApplicationInboundOccurrenceProtocol {
    identity: BoundaryProtocolIdentity,
    version: BoundaryProtocolVersion,
}

impl ApplicationInboundOccurrenceProtocol {
    pub const fn new(identity: BoundaryProtocolIdentity, version: BoundaryProtocolVersion) -> Self {
        Self { identity, version }
    }

    pub const fn identity(&self) -> &BoundaryProtocolIdentity {
        &self.identity
    }

    pub const fn version(&self) -> BoundaryProtocolVersion {
        self.version
    }
}
