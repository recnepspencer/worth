//! One owner resolution of all genuinely admitted prefix meanings.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::idempotency::envelope_candidates::EnvelopeCandidates;
impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn resolve_envelope_idempotency_at_product(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        candidates: &EnvelopeCandidates<'_>,
        product: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
    ) -> Result<
        WorthQueryProviderIdempotencyResolution,
        WorthQueryProviderIdempotencyResolutionDenial,
    > {
        let binding = candidates.first();
        self.application_attempt_work.observe_retained_resolution();
        if let Some(resolution) = self.resolve_recovered_envelope_idempotency(
            &WorthQueryProductIdempotencyAffinity::from_observation(product.observation()),
            candidates,
        )? {
            return Ok(resolution);
        }

        let layout = self.graph.layout.provider_idempotency().clone();
        let product_affinity =
            WorthQueryProductIdempotencyAffinity::from_observation(product.observation());
        match self.resolve_unpublished_envelope_idempotency(&product_affinity, candidates)? {
            Some(WorthQueryProviderIdempotencyResolution::PreparationBusy) => {
                return Err(WorthQueryProviderIdempotencyResolutionDenial::PreparationBusy)
            }
            Some(
                resolution @ (WorthQueryProviderIdempotencyResolution::PreparationPending {
                    ..
                }
                | WorthQueryProviderIdempotencyResolution::Drift
                | WorthQueryProviderIdempotencyResolution::ReleasedOriginal(_)),
            ) => return Ok(resolution),
            None
            | Some(
                WorthQueryProviderIdempotencyResolution::Absent
                | WorthQueryProviderIdempotencyResolution::Readmitted(_)
                | WorthQueryProviderIdempotencyResolution::RecoveredUnpublished { .. }
                | WorthQueryProviderIdempotencyResolution::Unpublished,
            ) => {}
        }
        let mut retired = crate::domain_computation::primary_graph::application_query::derived_view::PredicatePreparationRetirement::default();
        self.graph.with_runtime(execution, |runtime| {
            if binding.correction_identity().is_some()
                && self.inspect_pending_correction_idempotency(binding.key_identity())
                    .is_some_and(|recorded| recorded != binding)
            {
                return Ok(WorthQueryProviderIdempotencyResolution::Drift);
            }
            if let Some(Some(recorded)) = self.inspect_pending_application_idempotency(
                product.observation().lifecycle_incarnation())
            {
                if recorded.key_identity() == binding.key_identity() {
                    if !candidates.matches(recorded)? {
                        return Ok(WorthQueryProviderIdempotencyResolution::Drift);
                    }
                } else if recorded.correction_identity().is_some() {
                    return Ok(WorthQueryProviderIdempotencyResolution::Unpublished);
                }
            }
            self.resume_pending_application_publication(execution,
                runtime,
                product.observation().lifecycle_incarnation(),
                &mut retired,
            )
                .map_err(pending_publication_denial)?;
            // Concurrent requests can have admitted the same earlier head.
            // Under the branch lane, locate the key in the SAME live occurrence's
            // current owner selection, then read its immutable original source.
            let selected = product.observation_port().observe_product_branch_occurrence(
                product.observation().lifecycle_incarnation(),
            ).map_err(|_| WorthQueryProviderIdempotencyResolutionDenial::ForeignAffinity)?;
            let observation = &selected;
            let basis = observation.basis().relational_basis();
            if let Some(resolution) = self.resolve_unpublished_envelope_idempotency(
                &product_affinity,
                candidates,
            )? {
                return Ok(resolution);
            }
            self.graph
                .carry_primary_index_basis_for_basis(execution, runtime, basis)
                .map_err(idempotency_index_currency_denial)?;
            let snapshot = crate::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(runtime, basis)
                .map_err(|denial| idempotency_snapshot_denial(denial.into()))?;
            let resolution = resolve_at_snapshot(execution, self, runtime, &snapshot, &layout, product, candidates);
            crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
            let resolution = resolution?;
            if let WorthQueryProviderIdempotencyResolution::Readmitted(receipt) = &resolution {
                runtime.repair_pending_publication_settlement(receipt.commit_reference().commit_id)
                    .map_err(|_| WorthQueryProviderIdempotencyResolutionDenial::Unavailable)?;
            }
            Ok(resolution)
        })
    }
}
