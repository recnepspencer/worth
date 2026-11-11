//! Readmit only the exact historical row the selector verified, at the current head.
use super::restoration::ReadmittedOutput;
use super::*;
use crate::domain_computation::primary_graph::{
    application_output_demand::{RestoredOutputOrigin, WorthQueryRestoredAcceptedOutput},
    application_query::WorthQueryApplicationBasisSelectionIdentity,
};
use crate::equality_type_id::EqualityTypeId;

impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(super) fn readmit_historical_output<Family: WorthQueryProducerOutputFamily<Schema>>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        producer: &str,
        binding: EqualityTypeId,
        key: [u8; 32],
        observed_source: &WorthQueryObservedSource<FamilySourceQuery<Schema, Family>>,
        owner_retained_program_basis: bool,
    ) -> Result<Option<ReadmittedOutput>, WorthQueryOutputDemandDenial> {
        let WorthQueryApplicationBasisSelectionIdentity::Product(source_read) =
            &observed_source.selection
        else {
            return Ok(None);
        };
        let selected = self
            .on_branch(source_read.product_branch())
            .select_with_request(execution)
            .map_err(|cause| {
                WorthQueryOutputDemandDenial::product_selection(cause, "historical output source")
            })?;
        let observation = selected.product().observation();
        if !crate::domain_computation::primary_graph::application_contribution::producer::demand::selection::source_basis_is_admitted(
            source_read,
            observation,
            owner_retained_program_basis,
        ) {
            return Ok(None);
        }
        let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(observed_source.source_root());
        let graph = &self.runtime.primary_provider.graph;
        let (candidate, origin) = {
            let lineage = graph
                .output_lineage
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let candidates = lineage
                .retained_output_candidates_metered(
                    self.runtime.authority_identity().as_u64(),
                    &self.installed_schema.binding_identity(),
                    scope,
                    observation.lifecycle_incarnation(),
                    observation.reference_generation().get(),
                    &[binding],
                    observed_source.partition_identity(),
                    execution,
                )
                .map_err(|stop| {
                    crate::domain_computation::primary_graph::output_reuse::retained_fact_denial(
                        crate::domain_computation::primary_graph::output_reuse::preflight_stop(
                            stop,
                        ),
                        "historical output admission",
                    )
                })?;
            let Some(candidate) = candidates.into_iter().find(|candidate| {
                candidate.idempotency_key_identity == key
                    && candidate.historical_selection.is_some()
            }) else {
                return Ok(None);
            };
            let Some(origin) = lineage.historical_accepted_origin(&candidate) else {
                return Ok(None);
            };
            (candidate, origin)
        };
        if origin.publication().producer() != producer {
            return Ok(None);
        }
        let current = graph.with_runtime(execution, |native| {
            self.restored_output_is_current::<Family>(execution, native, &selected, &candidate)
        })?;
        if !current {
            return Ok(None);
        }
        let Some(epoch) = observed_source.output_source_epoch() else {
            return Ok(None);
        };
        let Some(facts) = candidate.observed_source_facts.as_ref() else {
            return Ok(None);
        };
        // Admission is a readonly acceptance of the already transported row.
        // It creates neither another lineage row nor a producer completion.
        Ok(Some(ReadmittedOutput {
            computation_source: origin.computation_source(),
            settlement: std::sync::Arc::clone(origin.settlement()),
            verified_at: None,
            restored: WorthQueryRestoredAcceptedOutput {
                origin: RestoredOutputOrigin::HistoricalSelection(origin),
                correspondence: candidate.correspondence,
                observation: observation.clone(),
                source_scope: scope,
                source_identity: epoch.checkpoint_identity(),
                observed_source_facts: facts.facts().clone(),
                native_output_witness: candidate.native_output_witness,
            },
        }))
    }
}
