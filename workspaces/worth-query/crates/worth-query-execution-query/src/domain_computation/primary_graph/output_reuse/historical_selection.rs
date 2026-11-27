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
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;

/// A borrowed view of one authentic retained row, used by selection and final
/// endpoint verification. It grants no acceptance or producer authority.
pub struct HistoricalOutputRead<'a> {
    /// The row's own settlement: its coordinate is where its chain is
    /// transported to.
    consumer: &'a crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity,
    coordinate: &'a HistoricalSelectionCoordinate,
    publication: Option<&'a worth_relational::facade::mvcc::RelationalRequiredOutputPublication>,
    facts: Option<&'a ComparableSourceFacts>,
    witness: Option<&'a SealedNativeOutputWitness>,
    upstream: &'a std::sync::Arc<[ConsumedOutputEvidence]>,
    /// A current read's re-rooted sets: the evidence its readmission proved
    /// and marked answers in place of the transported provenance (public #358).
    transported:
        Option<&'a crate::domain_computation::primary_graph::output_lineage::TransportedView>,
}
impl<'a> HistoricalOutputRead<'a> {
    pub fn from_candidate(
        row: &'a crate::domain_computation::primary_graph::output_lineage::current_output::retained_candidates::WorthQueryRetainedOutputCandidate,
    ) -> Option<Self> {
        Some(Self {
            consumer: &row.settlement_identity,
            coordinate: row.historical_selection.as_deref()?,
            publication: row.required_publication.as_ref(),
            facts: row.observed_source_facts.as_ref(),
            witness: row
                .native_output_witness
                .as_ref()
                .and_then(|witness| witness.get()),
            upstream: &row.consumed_outputs,
            transported: None,
        })
    }
    pub fn from_current_read(
        row: &'a crate::domain_computation::primary_graph::output_lineage::RetainedOutputCurrentnessRead,
    ) -> Option<Self> {
        Some(Self {
            consumer: &row.identity,
            coordinate: row.historical_selection.as_deref()?,
            publication: row.required_publication.as_ref(),
            facts: row.facts.as_ref(),
            witness: row
                .native_output_witness
                .as_ref()
                .and_then(|witness| witness.get()),
            upstream: &row.consumed_outputs,
            transported: row.transported.as_ref(),
        })
    }
    pub fn is_current<Schema: worth_query_installation::facade::ApplicationSchema>(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        selected: &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
            '_,
            Schema,
        >,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        let snapshot = selected.application_basis().snapshot_handle();
        let basis = crate::domain_computation::primary_graph::output_lineage::CertificationBasis::of(
            selected.product().lifecycle_incarnation(),
            selected.product().reference_generation(),
            snapshot,
        );
        let graph = &selected.application().runtime.primary_provider.graph_ref();
        let Some(publication) = self.publication else {
            return Ok(false);
        };
        if !publication_owned_at(execution, publication, selected)? {
            return Ok(false);
        }
        let Some(witness) = self.witness else {
            return Ok(false);
        };
        let evidence = match self.coordinate.read_evidence(execution, runtime) {
            Ok(evidence) => evidence,
            Err(cause) => return evidence_answer(Err(cause)),
        };
        let current = runtime
            .read_truth()
            .project_snapshot(snapshot)
            .ok_or_else(evidence_unavailable)?;
        // The upstream chain is verified first, so its rows can stand in for
        // their own family heads when this row's facts are compared (plan
        // §3.5, S1 STOP 3). A row's `FamilySelection` fact on a producer of
        // its own chain re-resolves at this basis, where the producer's head
        // is the transported row this readmission is about to post.
        let readmitted = self.verify_upstream(execution, runtime, basis, snapshot, &evidence, *graph)?;
        let Some(readmitted) = readmitted else {
            return Ok(false);
        };
        if !restored_facts_hold(
            execution,
            runtime,
            basis.with_readmitted(&readmitted),
            &current,
            &evidence,
            self.facts,
        )? {
            return Ok(false);
        }
        evidence_answer(owner_read(
            witness.unchanged_since_restoration(runtime, snapshot, execution, &current, &evidence),
            "restored output witness could not be compared",
        )?)
    }

    /// The upstream chain verified at `basis` and re-identified at the row's
    /// own coordinate, or `None` when it is not current there. A chain that
    /// verifies but has no transported copy stands in for nothing.
    fn verify_upstream(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        basis: crate::domain_computation::primary_graph::output_lineage::CertificationBasis<'_>,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        evidence: &worth_relational::facade::runtime::RelationalHistoricalSelectionRead,
        graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle,
    ) -> Result<
        Option<Vec<std::sync::Arc<crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity>>>,
        WorthQueryOutputDemandDenial,
    > {
        if self.upstream.is_empty() {
            return Ok(Some(Vec::new()));
        }
        let selected = runtime
            .read_truth()
            .positioned_snapshot(snapshot)
            .map_err(|_| evidence_unavailable())?;
        let upstream = crate::domain_computation::primary_graph::output_lineage::TransportedView::consumed_outputs_at(
            self.transported,
            self.upstream,
            &selected,
        );
        let result = graph
            .source_owner_ref()
            .invalidation_owner
            .capture_full_verification(runtime, basis, &selected)
            .and_then(|image| image.verify_historical_upstream(execution, &upstream, evidence));
        match result {
            Ok(FullVerificationDecision::Current) => {}
            Ok(FullVerificationDecision::Changed) => return Ok(None),
            Err(FullVerificationStop::Admission(stop)) => {
                return owner_read::<()>(Err(stop), "historical upstream comparison").map(|()| None);
            }
            Err(FullVerificationStop::SourceRead(cause)) => {
                use crate::domain_computation::primary_graph::{
                    invariant_projection::consumed_output::map_verification_stop,
                    output_lineage::invalidation::SettlementVerificationStop,
                };
                return Err(retained_fact_denial(
                    RetainedFactStop::OutputObservation(map_verification_stop(
                        SettlementVerificationStop::SourceRead(cause),
                    )),
                    "historical upstream comparison",
                ));
            }
            Err(_) => return Ok(None),
        }
        let readmitted = graph
            .output_lineage_ref()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .reidentified_chain(self.consumer, self.coordinate, &upstream, &selected)
            .unwrap_or_default();
        Ok(Some(readmitted))
    }
}

/// Whether `publication` still belongs to the current owner at `selected`'s
/// basis. A transported row and each upstream readmission re-identifies
/// (public #358) must pass it before anything is marked.
pub(crate) fn publication_owned_at<Schema: worth_query_installation::facade::ApplicationSchema>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    publication: &worth_relational::facade::mvcc::RelationalRequiredOutputPublication,
    selected: &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
        '_,
        Schema,
    >,
) -> Result<bool, WorthQueryOutputDemandDenial> {
    let graph = &selected.application().runtime.primary_provider.graph_ref();
    let Some(publisher) = graph.application_publication_ref() else {
        return Ok(false);
    };
    publication
        .matches_current_ownership_at(execution, &publisher.native, selected.product().relational_basis())
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
        })
}
