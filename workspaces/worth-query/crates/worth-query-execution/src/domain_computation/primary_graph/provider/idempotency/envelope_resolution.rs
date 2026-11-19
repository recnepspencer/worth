//! One owner resolution of all genuinely admitted prefix meanings.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::authored_edit::AuthoredEnvelopeResolutionPurpose;
use crate::domain_computation::primary_graph::application_attempt::idempotency::envelope_candidates::EnvelopeCandidates;
impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn resolve_envelope_idempotency_at_product(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        candidates: &EnvelopeCandidates<'_>,
        product: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
        purpose: AuthoredEnvelopeResolutionPurpose,
    ) -> Result<
        WorthQueryProviderIdempotencyResolution,
        WorthQueryProviderIdempotencyResolutionDenial,
    > {
        let binding = candidates.first();
        self.application_attempt_work.observe_retained_resolution();
        if let Some(resolution) = self.resolve_recovered_envelope_idempotency(
            &WorthQueryProductIdempotencyAffinity::from_observation(product.observation()),
            candidates,
        ).inspect_err(|e| eprintln!("MGDEBUG idem recovered-envelope {e:?}"))? {
            return Ok(resolution);
        }

        let layout = self.graph.layout.provider_idempotency().clone();
        let product_affinity =
            WorthQueryProductIdempotencyAffinity::from_observation(product.observation());
        match self.resolve_unpublished_envelope_idempotency(&product_affinity, candidates).inspect_err(|e| eprintln!("MGDEBUG idem unpublished-1 {e:?}"))? {
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
            if let Some(pending) = self.inspect_pending_application_idempotency(
                product.observation().lifecycle_incarnation())
            {
                if let Some(recorded) = pending {
                    if recorded.key_identity() == binding.key_identity() {
                        if !candidates.matches(recorded)? {
                            return Ok(WorthQueryProviderIdempotencyResolution::Drift);
                        }
                    } else if recorded.correction_identity().is_some() {
                        eprintln!("MGDEBUG idem pending-correction Unpublished");
                        return Ok(WorthQueryProviderIdempotencyResolution::Unpublished);
                    }
                }
                if purpose == AuthoredEnvelopeResolutionPurpose::ResolveOnly {
                    return Err(WorthQueryProviderIdempotencyResolutionDenial::Unavailable);
                }
            }
            if purpose == AuthoredEnvelopeResolutionPurpose::Execute {
                self.resume_pending_application_publication(execution,
                    runtime,
                    product.observation().lifecycle_incarnation(),
                    &mut retired,
                ).map_err(pending_publication_denial).inspect_err(|e| eprintln!("MGDEBUG idem resume-pending {e:?}"))?;
            }
            // Concurrent requests can have admitted the same earlier head.
            // Under the branch lane, locate the key in the SAME live occurrence's
            // current owner selection, then read its immutable original source.
            let selected = product.observation_port().observe_product_branch_occurrence(
                product.observation().lifecycle_incarnation(),
            ).map_err(|_| WorthQueryProviderIdempotencyResolutionDenial::ForeignAffinity).inspect_err(|e| eprintln!("MGDEBUG idem observe-occurrence {e:?}"))?;
            let observation = &selected;
            let basis = observation.basis().relational_basis();
            if let Some(resolution) = self.resolve_unpublished_envelope_idempotency(
                &product_affinity,
                candidates,
            ).inspect_err(|e| eprintln!("MGDEBUG idem unpublished-2 {e:?}"))? {
                eprintln!("MGDEBUG idem unpublished-2 resolution {resolution:?}");
                return Ok(resolution);
            }
            self.graph
                .carry_primary_index_basis_for_basis(execution, runtime, basis)
                .map_err(|d| { eprintln!("MGDEBUG idem carry-index {d:?}"); idempotency_index_currency_denial(d) })?;
            // A carried known-empty selection is not yet a concrete lookup
            // generation. Admit only this key index through Native's ordinary
            // field preparation; it never reconstructs a missing population.
            runtime.index_authority().ensure_field_index_for_basis(
                layout.key_index_id,
                basis,
                crate::domain_computation::primary_graph::index_maintenance_budget::ordinary_index_maintenance_budget(),
                execution,
            ).map_err(|denial| { eprintln!("MGDEBUG idem ensure-key-index {:?}", denial.kind); denial }).map_err(|denial| match denial.kind {
                worth_relational::facade::indexes::DerivedIndexMaintenanceDenialKind::Execution(cause) => snapshot_resolution::idempotency_read_stop(cause),
                _ => WorthQueryProviderIdempotencyResolutionDenial::Unavailable,
            })?;
            let snapshot = crate::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(runtime, basis)
                .map_err(|denial| { eprintln!("MGDEBUG idem open-snapshot"); idempotency_snapshot_denial(denial.into()) })?;
            let resolution = resolve_at_snapshot(execution, self, runtime, &snapshot, &layout, product, candidates);
            crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
            let resolution = resolution.inspect_err(|e| eprintln!("MGDEBUG idem at-snapshot {e:?}
{}", std::backtrace::Backtrace::force_capture()))?;
            if matches!(resolution, WorthQueryProviderIdempotencyResolution::Unpublished) { eprintln!("MGDEBUG idem at-snapshot Unpublished"); }
            if purpose == AuthoredEnvelopeResolutionPurpose::Execute
                && let WorthQueryProviderIdempotencyResolution::Readmitted(receipt) = &resolution {
                runtime.repair_pending_publication_settlement(receipt.commit_reference().commit_id)
                    .map_err(|_| { eprintln!("MGDEBUG idem repair-settlement"); WorthQueryProviderIdempotencyResolutionDenial::Unavailable })?;
            }
            Ok(resolution)
        })
    }
}
