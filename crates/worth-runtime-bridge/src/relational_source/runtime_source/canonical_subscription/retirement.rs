//! Forward authentic Native registration retirement without dropping its consumer.
use super::{PublicationCompanionRegistrationPort, RelationalBridgeCanonicalSubscription};
use worth_execution::ExecutionRequest;
use worth_relational::facade::mvcc::{
    PublicationCompanionRetirementDenial, RetiredPublicationCompanionRegistration,
};

/// Original Native callback custody and Bridge port after routing is detached.
/// Final destruction is unbudgeted; no source or runtime authority is exposed.
#[derive(Debug)]
pub struct RetiredRelationalBridgeCanonicalSubscription {
    _native: RetiredPublicationCompanionRegistration,
    _port: PublicationCompanionRegistrationPort,
}

impl RetiredRelationalBridgeCanonicalSubscription {
    /// Reclaim only outside a request-controlled operation.
    pub fn reclaim(self) {
        drop(self);
    }
}

impl RelationalBridgeCanonicalSubscription {
    /// Consume the same subscription, retaining its consumer after detachment.
    /// A refusal returns the original subscription for retry or ordinary disposal.
    pub fn try_retire_for_request(
        self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        RetiredRelationalBridgeCanonicalSubscription,
        (Self, PublicationCompanionRetirementDenial),
    > {
        let Self { registration, port } = self;
        match port.try_retire_required_for_request(registration, request) {
            Ok(native) => Ok(RetiredRelationalBridgeCanonicalSubscription {
                _native: native,
                _port: port,
            }),
            Err((registration, denial)) => Err((Self { registration, port }, denial)),
        }
    }
}
