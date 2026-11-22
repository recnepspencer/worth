//! Select retained candidates under the caller's native request.

use super::*;

use worth_relational::facade::mvcc::CompanionPreflightStop;

/// The latest output one binding recorded for a source, with everything
/// selection compares before it reuses that output.
pub struct WorthQueryRetainedOutputCandidate {
    pub original_ready_key: Option<std::sync::Arc<crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandKey>>,
    pub coupled_completion: Option<super::super::CoupledOutputCompletion>,
    pub recovered_checkpoint: Option<crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity>,
    pub required_publication: Option<worth_relational::facade::mvcc::RelationalRequiredOutputPublication>,
    pub producing_implementation: Option<worth_relational::facade::mvcc::RequiredOutputProducingImplementation>,
    pub binding: TypeId,
    pub correspondence:
        Arc<crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence>,
    pub source_identity:
        Option<RecordedSourceIdentity>,
    pub observed_source_facts: Option<super::super::ComparableSourceFacts>,
    /// The performed output's sealed witness: its origin's for a stable
    /// alias. A restored row has none until a full verification builds it.
    pub native_output_witness:
        Option<Arc<OnceLock<crate::domain_computation::primary_graph::output_lineage::SealedNativeOutputWitness>>>,
    /// Why the row's settlement is verified in full, when it is: the row was
    /// restored, or the owner could not register it. A commit that could not
    /// rebase its facts states that here and retains none of them.
    pub verification_requirement:
        Option<crate::domain_computation::primary_graph::output_lineage::invalidation::FullVerificationReason>,
    /// The row's own settlement, as the owner knows it.
    pub settlement_identity:
        Arc<crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity>,

    pub idempotency_key_identity: [u8; 32],
    /// Only a row a graph selection transported; its own coordinate, never an alias's.
    pub consumed_outputs: Arc<[crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence]>,
    pub historical_selection:
        Option<Arc<super::super::HistoricalSelectionCoordinate>>,
}

impl WorthQueryRetainedOutputCandidate {
    /// The lifecycle this output was actually produced under. A standalone
    /// publication states it in its ready key. A publication coupled to its
    /// source commit has no key and states the lifecycle its writer was
    /// qualified under.
    pub fn performed_lifecycle(
        &self,
    ) -> Option<crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerLifecyclePosture>
{
        if let Some(key) = &self.original_ready_key {
            return Some(key.applicability().lifecycle());
        }
        Some(self.coupled_completion.as_ref()?.source().lifecycle())
    }
}

impl WorthQueryApplicationOutputLineage {
    /// Each binding's latest retained output, with every lookup reserved on
    /// the request's meter before it reads. A lookup that runs out keeps its
    /// reservation spent.
    pub fn retained_output_candidates_metered(
        &self,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        generation: u64,
        output_bindings: &[TypeId],
        source_partition_identity: [u8; 32],
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Vec<WorthQueryRetainedOutputCandidate>, CompanionPreflightStop> {
        let mut candidates = Vec::new();
        for output_binding in output_bindings {
            let source = SemanticSource {
                runtime_authority,
                schema: schema.clone(),
                scope,
                output_binding: self
                    .binding_identity(*output_binding)
                    .expect("output family binding is installed"),
            };
            if !self.by_source.contains_key(&source) {
                continue;
            }
            let mut coordinate = ProductCoordinate {
                occurrence,
                generation,
            };
            loop {
                execution
                    .consult()
                    .map_err(|stop| CompanionPreflightStop::Execution(stop.into()))?;
                let recorded = self
                    .partition_index
                    .latest_admitted(execution, &source, coordinate, source_partition_identity)?
                    .map(|(generation, slot)| {
                        self.recorded_at_partition_slot(
                            &source,
                            coordinate.occurrence,
                            generation,
                            Some(source_partition_identity),
                            slot,
                        )
                    });
                if let Some(recorded) = recorded {
                    let completed = recorded
                        .completed_computation_record()
                        .ok_or(CompanionPreflightStop::SelectedSourceMismatch)?;
                    candidates.push(WorthQueryRetainedOutputCandidate {
                        original_ready_key: completed.original_ready_key.clone(),
                        coupled_completion: recorded.coupled_completion.clone(),
                        producing_implementation: recorded.producing_implementation,
                        required_publication: recorded.required_publication.clone(),
                        recovered_checkpoint: recorded
                            .native_prior_checkpoint
                            .as_ref()
                            .and_then(|prior| prior.recovered().cloned()),
                        binding: *output_binding,
                        correspondence: Arc::clone(&recorded.correspondence),
                        source_identity: recorded.source_identity,
                        observed_source_facts: recorded
                            .observed_source_facts()
                            .and_then(|facts| facts.for_comparison()),
                        // Ordinary aliases keep the performed witness. An
                        // actual adopting execution owns its fresh live witness.
                        native_output_witness: completed
                            .native_output_witness_cell()
                            .map(Arc::clone),
                        verification_requirement: recorded.verification_requirement(),
                        settlement_identity: Arc::clone(&recorded.settlement_identity),

                        idempotency_key_identity: recorded.idempotency_key_identity,
                        historical_selection: recorded.historical_selection.clone(),
                        consumed_outputs: Arc::clone(&recorded.consumed_outputs),
                    });
                    break;
                }
                let Some(parent) = self.origins.get(&coordinate.occurrence).copied() else {
                    break;
                };
                coordinate = parent;
            }
        }
        Ok(candidates)
    }
}

mod court_doors_for_test;
