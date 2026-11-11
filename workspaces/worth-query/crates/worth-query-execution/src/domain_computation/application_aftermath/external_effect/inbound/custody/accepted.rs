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
    pub(in crate::domain_computation) fn record_preparation_denial(
        &self,
        cause: Option<Arc<worth_runtime_world::facade::WorldNativePreparationDenial>>,
    ) {
        *self
            .preparation_denial
            .lock()
            .unwrap_or_else(|p| p.into_inner()) = cause;
    }
    pub(in crate::domain_computation) fn preparation_denial(
        &self,
    ) -> Option<Arc<worth_runtime_world::facade::WorldNativePreparationDenial>> {
        self.preparation_denial
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone()
    }

    pub(in crate::domain_computation) fn claims(&self) -> &WorthQueryInboundOccurrenceClaims {
        &self.claims
    }

    pub(in crate::domain_computation) fn operation(&self) -> &str {
        &self.operation
    }

    pub(in crate::domain_computation) fn owner(
        &self,
    ) -> &WorthQueryCommittedDispatchOutboxObservation {
        &self.owner
    }

    pub(in crate::domain_computation) fn signed_meaning_digest(&self) -> &[u8; 32] {
        &self.signed_meaning_digest
    }
}
