//! Immutable accepted evidence retains the exact original source observation.
use super::*;
/// Immutable accepted evidence retained by Query even if every caller drops
/// its observer. Publication may borrow this but cannot change its meaning.
pub struct WorthQueryAcceptedInboundOccurrence {
    pub(super) operation: String,
    pub(super) claims: WorthQueryInboundOccurrenceClaims,
    pub(super) owner: WorthQueryCommittedDispatchOutboxObservation,
    pub(super) signed_meaning_digest: [u8; 32],
    pub(super) preparation_denial:
        Mutex<Option<Arc<worth_runtime_world::facade::WorldNativePreparationDenial>>>,
}

impl WorthQueryAcceptedInboundOccurrence {
    pub fn record_preparation_denial(
        &self,
        cause: Option<Arc<worth_runtime_world::facade::WorldNativePreparationDenial>>,
    ) {
        *self
            .preparation_denial
            .lock()
            .unwrap_or_else(|p| p.into_inner()) = cause;
    }
    pub fn preparation_denial(
        &self,
    ) -> Option<Arc<worth_runtime_world::facade::WorldNativePreparationDenial>> {
        self.preparation_denial
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone()
    }

    pub fn claims(&self) -> &WorthQueryInboundOccurrenceClaims {
        &self.claims
    }

    pub fn operation(&self) -> &str {
        &self.operation
    }

    pub fn owner(&self) -> &WorthQueryCommittedDispatchOutboxObservation {
        &self.owner
    }

    pub fn signed_meaning_digest(&self) -> &[u8; 32] {
        &self.signed_meaning_digest
    }
}
