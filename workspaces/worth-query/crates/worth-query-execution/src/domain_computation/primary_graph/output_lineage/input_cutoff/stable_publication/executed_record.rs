//! Fresh completed-handler support for a Native-judged no-effect adoption.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::retained_decision_facts::StorageControl,
    application_checkpoint::facts::FactPayload,
    output_lineage::{ComputationSourceEvidence, RecordedOutputMutable, RecordedSourceIdentity},
    provider::WorthQueryPrimaryGraphApplicationAttempt,
    PreparedPrerequisiteClaims,
};

impl<'lane, 'selected> PreparedStableLineageAddress<'lane, 'selected> {
    pub(in crate::domain_computation::primary_graph::output_lineage) fn prepare_executed_record(
        self,
        adoption: &worth_relational::facade::mvcc::RelationalCurrentOutputAdoption,
        association: &super::super::super::required_publication::head::ReadmittedRequiredPublication,
        attempt: &WorthQueryPrimaryGraphApplicationAttempt,
        facts: FactPayload,
        witness: Arc<OnceLock<super::super::super::SealedNativeOutputWitness>>,
        required: &mut PreparedPrerequisiteClaims,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<PreparedStableLineagePublication<'lane, 'selected>, WorthQueryOutputDemandDenial>
    {
        let control = StorageControl::new(
            execution.allocation_policy(),
            Some(attempt.affinity().publication_request()),
        );
        let prior = self
            .candidate
            .originating_recorded()
            .ok_or_else(|| denial(Kind::IncompleteDependencyCoverage))?;
        if self.input_cutoff.is_some()
            || !association.matches_cell(&self.candidate.cell)
            || !association
                .publication()
                .same_origin(adoption.publication())
            || attempt
                .idempotency()
                .producer_dependency_identity()
                .as_ref()
                != Some(adoption.dependency_identity())
            || attempt.idempotency().source_partition_identity() != Some(self.partition)
        {
            return Err(denial(Kind::ForeignSource));
        }
        let boundary = required
            .take_completed_handler_facts()
            .ok_or_else(|| denial(Kind::IncompleteDependencyCoverage))?;
        let count = boundary.handler_fact_count();
        let prefix = facts
            .checked_slice(0..count)
            .ok_or_else(|| denial(Kind::IncompleteDependencyCoverage))?;
        let suffix = facts
            .checked_slice(count..facts.len())
            .ok_or_else(|| denial(Kind::IncompleteDependencyCoverage))?;
        let projection = witness
            .get()
            .ok_or_else(|| denial(Kind::IncompleteDependencyCoverage))?
            .prepare_fact_projection()
            .map_err(storage_stop)?
            .ok_or_else(|| denial(Kind::IncompleteDependencyCoverage))?;
        let witness_count = projection.count();
        let projection = projection
            .retain_payload(control)
            .map_err(super::storage_denial)?;
        let facts = prefix
            .append_completed(projection, control)
            .and_then(|facts| facts.append_completed(suffix, control))
            .map_err(super::storage_denial)?;
        let spans = StableFactSpans::new(facts.clone(), count, witness_count)
            .ok_or_else(|| denial(Kind::IncompleteDependencyCoverage))?;
        let (sealed, _) = required.take_sealed_computation();
        let computation = match sealed {
            crate::domain_computation::primary_graph::SealedComputationRetention::Produced(
                sealed,
            ) => self
                .owner
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .retain_computation(sealed),
            crate::domain_computation::primary_graph::SealedComputationRetention::Absent(cause) => {
                super::super::super::retained_computation::RecordedComputation::Absent(cause)
            }
        };
        let computation_source = ComputationSourceEvidence::from_executed_adoption(adoption);
        let native_output_witness = OnceLock::from(witness);
        let recorded = RecordedOutput {
            original_ready_key: Some(required.original_ready_key()),
            coupled_completion: None,
            producing_implementation: prior.producing_implementation,
            required_publication: Some(adoption.publication().clone()),
            native_prior_checkpoint: None,
            computation_source,
            performed_origin: Some(self.candidate.recorded().performed_origin.as_ref()
                .map(Arc::clone).unwrap_or_else(|| Arc::clone(&self.candidate.cell))),
            historical_selection: None,
            consumed_outputs: attempt.retain_consumed_outputs(),
            completed_handler_facts: Some(boundary),
            completed_decision_reuse: required.take_completed_decision_reuse(),
            prepared_input_reuse_key: required.take_prepared_input_reuse_key(),
            native_output_witness,
            settlement_identity: Arc::clone(&self.identity),
            correspondence: Arc::clone(&prior.correspondence),
            source_identity: attempt.idempotency().source_identity().map(|identity| {
                RecordedSourceIdentity::Runtime(
                    crate::domain_computation::primary_graph::application_query::WorthQueryRuntimeSourceIdentity::new(identity))
            }),
            source_partition_identity: Some(self.partition),
            producer_dependency_identity: prior.producer_dependency_identity,
            idempotency_key_identity: prior.idempotency_key_identity,
            mutable: Mutex::new(RecordedOutputMutable::new(None,
                Some(computation_source.retain_facts(facts.clone())), computation)
                .with_stable_fact_spans(spans)),
        };
        Ok(PreparedStableLineagePublication {
            address: self,
            recorded,
            facts,
        })
    }
}

fn storage_stop(
    stop: worth_relational::facade::mvcc::CompanionPreflightStop,
) -> WorthQueryOutputDemandDenial {
    crate::domain_computation::primary_graph::output_reuse::retained_fact_denial(
        crate::domain_computation::primary_graph::output_reuse::preflight_stop(stop),
        "executed adoption witness support",
    )
}
