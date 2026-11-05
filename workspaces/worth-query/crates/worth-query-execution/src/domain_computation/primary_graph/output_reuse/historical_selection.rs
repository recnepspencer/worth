//! One restored-output currentness comparison at any final selected endpoint.
use super::*;
use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence;
use crate::domain_computation::primary_graph::output_lineage::invalidation::{
    FullVerificationDecision, FullVerificationStop,
};
use crate::domain_computation::primary_graph::output_lineage::{
    ComparableSourceFacts, HistoricalSelectionCoordinate, SealedNativeOutputWitness,
};
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;

/// A borrowed view of one authentic retained row, used by selection and final
/// endpoint verification. It grants no acceptance or producer authority.
pub(in crate::domain_computation::primary_graph) struct HistoricalOutputRead<'a> {
    coordinate: &'a HistoricalSelectionCoordinate,
    publication: Option<&'a worth_relational::facade::mvcc::RelationalRequiredOutputPublication>,
    facts: Option<&'a ComparableSourceFacts>,
    witness: Option<&'a SealedNativeOutputWitness>,
    upstream: &'a [ConsumedOutputEvidence],
}
impl<'a> HistoricalOutputRead<'a> {
    pub(in crate::domain_computation::primary_graph) fn from_candidate(
        row: &'a crate::domain_computation::primary_graph::output_lineage::current_output::retained_candidates::WorthQueryRetainedOutputCandidate,
    ) -> Option<Self> {
        Some(Self {
            coordinate: row.historical_selection.as_deref()?,
            publication: row.required_publication.as_ref(),
            facts: row.observed_source_facts.as_ref(),
            witness: row
                .native_output_witness
                .as_ref()
                .and_then(|witness| witness.get()),
            upstream: &row.consumed_outputs,
        })
    }
    pub(in crate::domain_computation::primary_graph) fn from_current_read(
        row: &'a crate::domain_computation::primary_graph::output_lineage::RetainedOutputCurrentnessRead,
    ) -> Option<Self> {
        Some(Self {
            coordinate: row.historical_selection.as_deref()?,
            publication: row.required_publication.as_ref(),
            facts: row.facts.as_ref(),
            witness: row
                .native_output_witness
                .as_ref()
                .and_then(|witness| witness.get()),
            upstream: &row.consumed_outputs,
        })
    }
    pub(in crate::domain_computation::primary_graph) fn is_current<
        Schema: worth_query_installation::facade::ApplicationSchema,
    >(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        selected: &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
            '_,
            Schema,
        >,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        let snapshot = selected.application_basis().snapshot_handle();
        let basis = selected.product().relational_basis();
        let graph = &selected.application().runtime.primary_provider.graph;
        let (Some(publication), Some(publisher)) =
            (self.publication, graph.application_publication.as_deref())
        else {
            return Ok(false);
        };
        if !publication
            .matches_current_ownership_at(execution, &publisher.native, basis)
            .map_err(|cause| {
                use worth_relational::facade::transactions::{
                    CommitExecutionDenialKind, TransactionCommitError,
                };
                match cause {
                    TransactionCommitError::Execution { denial, .. } => {
                        let CommitExecutionDenialKind::Cause(cause) = denial.kind;
                        read_denial(cause)
                    }
                    TransactionCommitError::Interrupted { interruption, .. } => {
                        interruption_denial(interruption)
                    }
                    cause => WorthQueryOutputDemandDenial::new(
                        crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::CurrentPublication(Box::new(cause)),
                        "historically selected publication current facts",
                    ),
                }
            })?
        {
            return Ok(false);
        }
        let Some(witness) = self.witness else {
            return Ok(false);
        };
        let evidence = match runtime.read_truth().read_historical_selection(
            execution,
            &self.coordinate.selected,
            &self.coordinate.restored,
        ) {
            Ok(evidence) => evidence,
            Err(cause) => return evidence_answer(Err(cause)),
        };
        let current = runtime
            .read_truth()
            .project_snapshot(snapshot)
            .ok_or_else(evidence_unavailable)?;
        if !restored_facts_hold(
            execution, runtime, snapshot, &current, &evidence, self.facts,
        )? {
            return Ok(false);
        }
        if !evidence_answer(owner_read(
            witness.unchanged_since_restoration(runtime, snapshot, execution, &current, &evidence),
            "restored output witness could not be compared",
        )?)? {
            return Ok(false);
        }
        if self.upstream.is_empty() {
            return Ok(true);
        }
        let selected = runtime
            .read_truth()
            .positioned_snapshot(snapshot)
            .map_err(|_| evidence_unavailable())?;
        let result = graph
            .source_owner
            .invalidation_owner
            .capture_full_verification(runtime, snapshot, &selected)
            .and_then(|image| {
                image.verify_historical_upstream(execution, self.upstream, &evidence)
            });
        match result {
            Ok(FullVerificationDecision::Current) => Ok(true),
            Ok(FullVerificationDecision::Changed) => Ok(false),
            Err(FullVerificationStop::Admission(stop)) => {
                owner_read(Err(stop), "historical upstream comparison")
            }
            Err(FullVerificationStop::SourceRead(cause)) => {
                use crate::domain_computation::primary_graph::{
                    invariant_projection::consumed_output::map_verification_stop,
                    output_lineage::invalidation::SettlementVerificationStop,
                };
                Err(retained_fact_denial(
                    RetainedFactStop::OutputObservation(map_verification_stop(
                        SettlementVerificationStop::SourceRead(cause),
                    )),
                    "historical upstream comparison",
                ))
            }
            Err(_) => Ok(false),
        }
    }
}
