//! Preserve the existing partial/released owner after complete prefix matching.
use super::*;
pub(super) use crate::domain_computation::primary_graph::application_attempt::idempotency::envelope_candidates::EnvelopeCandidates;
impl super::super::WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn resolve_unpublished_envelope_idempotency(
        &self,
        product: &WorthQueryProductIdempotencyAffinity,
        candidates: &EnvelopeCandidates<'_>,
    ) -> Result<
        Option<WorthQueryProviderIdempotencyResolution>,
        super::super::WorthQueryProviderIdempotencyResolutionDenial,
    > {
        let store = self
            .unpublished_idempotency
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let key = WorthQueryUnpublishedIdempotencyKey::for_binding(product, &candidates.first());
        let Some(entry) = store.by_key.get(&key) else {
            return Ok(None);
        };
        let original = entry.lock().unwrap_or_else(|e| e.into_inner()).binding;
        if !candidates.matches(original)? {
            return Ok(Some(WorthQueryProviderIdempotencyResolution::Drift));
        }
        Ok(store.resolve(product, original))
    }
}
