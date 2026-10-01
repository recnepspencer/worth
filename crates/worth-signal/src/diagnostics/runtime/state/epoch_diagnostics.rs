//! A private, selected-root diagnostics draft for one evaluation epoch.
//! Replay, branch, flow, and snapshot roots stay on the live state.
use std::sync::Arc;

use super::{
    lineage_publication::LineageRetentionCustody, DiagnosticsState, LineagePublicationDenial,
};
use crate::data::error::SignalError;
use crate::data::persistent_ord_map::{RetainedMapMutationDenial, RetainedMapMutationOutcome};
use crate::data::request_preparation::SignalPreparationBudget;
use crate::data::retained_storage::{
    arc_allocation_charge, RetainedStorageCharge as Charge, RetainedStorageForkPreparation,
    RetainedStorageMeasurement, RetainedStoragePreparation as Work,
    RetainedStoragePreparationDenial, SignalConditionalRetentionDenial,
    SignalConditionalRetentionLedger, SignalConditionalRetentionReservation as Reservation,
};
use crate::diagnostics::facts::{ExplanationFact, ProvenanceFact};
use crate::diagnostics::lineage::LineageRecord;

mod preparation_bounds;

pub(crate) struct PreparedEpochDiagnostics {
    staged: DiagnosticsState,
    ledger: Option<Arc<SignalConditionalRetentionLedger>>,
    fact_resources: Option<Reservation>,
    maximum_fact_nodes: usize,
    recorded_fact_nodes: usize,
}

impl DiagnosticsState {
    /// The fixed selected-root and index-path envelope before epoch callbacks.
    /// Writer-maintained charge facts make this O(1) in retained history size.
    pub(crate) fn epoch_preparation_capacity_bound(
        &self,
        maximum_fact_nodes: usize,
    ) -> Result<u64, SignalError> {
        self.lineage_records
            .prepared_retained_charge()
            .map_err(|_| SignalError::EvaluationStorageUnavailable)?;
        let artifact = self
            .lineage_records_by_artifact
            .prepared_fork_charge()
            .map_err(|_| SignalError::EvaluationStorageUnavailable)?;
        let node = self
            .lineage_records_by_node
            .prepared_fork_charge()
            .map_err(|_| SignalError::EvaluationStorageUnavailable)?;
        let explanation = self
            .explanation_facts
            .prepared_fork_charge()
            .map_err(|_| SignalError::EvaluationStorageUnavailable)?;
        let provenance = self
            .provenance_facts
            .prepared_fork_charge()
            .map_err(|_| SignalError::EvaluationStorageUnavailable)?;
        let request_growth = preparation_bounds::request_growth(
            self,
            maximum_fact_nodes,
            artifact,
            node,
            explanation,
            provenance,
        )?;
        (std::mem::size_of::<Self>() as u64)
            .checked_add(request_growth.bytes())
            .ok_or_else(|| SignalError::invalid_input("diagnostics draft memory overflow"))
    }

    pub(crate) fn prepare_epoch_diagnostics(
        &mut self,
        work: &mut Work,
        mut request: Option<&mut SignalPreparationBudget>,
        ledger: Option<&Arc<SignalConditionalRetentionLedger>>,
        maximum_fact_nodes: usize,
    ) -> Result<PreparedEpochDiagnostics, SignalError> {
        work.reserve_visits(std::mem::size_of::<Self>())
            .map_err(map_node_edit_accounting)?;
        if let Some(request) = request.as_deref_mut() {
            request.claim(std::mem::size_of::<Self>() as u64)?;
        }

        // Only the selected roots are prepared and forked. History clone shares
        // frames; map conversion is reserved before constructing the draft.
        let history = self
            .lineage_records
            .prepared_retained_charge()
            .map_err(|_| SignalError::EvaluationStorageUnavailable)?;
        if self
            .lineage_records_by_artifact
            .prepared_retained_charge()
            .is_err()
            || self
                .lineage_records_by_node
                .prepared_retained_charge()
                .is_err()
            || self.explanation_facts.prepared_retained_charge().is_err()
            || self.provenance_facts.prepared_retained_charge().is_err()
        {
            return Err(SignalError::EvaluationStorageUnavailable);
        }
        let artifact = self
            .lineage_records_by_artifact
            .prepare_fork_charge(work)
            .map_err(map_node_edit_accounting)?;
        let node = self
            .lineage_records_by_node
            .prepare_fork_charge(work)
            .map_err(map_node_edit_accounting)?;
        let explanation = self
            .explanation_facts
            .prepare_fork_charge(work)
            .map_err(map_node_edit_accounting)?;
        let provenance = self
            .provenance_facts
            .prepare_fork_charge(work)
            .map_err(map_node_edit_accounting)?;

        let (lineage_reserve, fact_reserve) = preparation_bounds::reserves(
            self,
            maximum_fact_nodes,
            history,
            artifact,
            node,
            explanation,
            provenance,
        )?;
        if let Some(request) = request {
            request.claim(
                preparation_bounds::request_growth(
                    self,
                    maximum_fact_nodes,
                    artifact,
                    node,
                    explanation,
                    provenance,
                )?
                .bytes(),
            )?;
        }

        let mut lineage_resources = ledger
            .map(|ledger| ledger.reserve(0, lineage_reserve))
            .transpose()
            .map_err(map_node_edit_retention)?;
        let mut fact_resources = ledger
            .map(|ledger| ledger.reserve(0, fact_reserve))
            .transpose()
            .map_err(map_node_edit_retention)?;

        let mut staged = Self::default();
        staged.installed_retention_budget = self.installed_retention_budget;
        staged.installed_tier = self.installed_tier;
        staged.installed_frontier_tracing_policy = self.installed_frontier_tracing_policy;
        staged.next_lineage_artifact_id = self.next_lineage_artifact_id;
        staged.next_lineage_sequence = self.next_lineage_sequence;
        staged.lineage_records = self.lineage_records.clone();
        staged.lineage_records_by_artifact = match lineage_resources.as_mut() {
            Some(resources) => self.lineage_records_by_artifact.fork_reserved(resources),
            None => self.lineage_records_by_artifact.fork_persistent(),
        };
        staged.lineage_records_by_node = match lineage_resources.as_mut() {
            Some(resources) => self.lineage_records_by_node.fork_reserved(resources),
            None => self.lineage_records_by_node.fork_persistent(),
        };
        staged.explanation_facts = match fact_resources.as_mut() {
            Some(resources) => self.explanation_facts.fork_reserved(resources),
            None => self.explanation_facts.fork_persistent(),
        };
        staged.provenance_facts = match fact_resources.as_mut() {
            Some(resources) => self.provenance_facts.fork_reserved(resources),
            None => self.provenance_facts.fork_persistent(),
        };
        staged.lineage_custody = lineage_resources
            .map(|resources| LineageRetentionCustody(Some(Arc::new(resources))))
            .unwrap_or_else(|| self.lineage_custody.clone());
        Ok(PreparedEpochDiagnostics {
            staged,
            ledger: ledger.cloned(),
            fact_resources,
            maximum_fact_nodes,
            recorded_fact_nodes: 0,
        })
    }
}

impl PreparedEpochDiagnostics {
    pub(crate) fn state_mut(&mut self) -> &mut DiagnosticsState {
        &mut self.staged
    }

    pub(crate) fn push_lineage(
        &mut self,
        record: LineageRecord,
        work: &mut Work,
        request: Option<&mut SignalPreparationBudget>,
    ) -> Result<(), SignalError> {
        let copied = record
            .retained_heap_charge(work)
            .and_then(|charge| charge.checked_add(arc_allocation_charge::<LineageRecord>()?))
            .map_err(map_node_edit_accounting)?;
        if let Some(request) = request {
            // One global and up to two indexed frames can coexist until the
            // epoch finishes. The retained ledger performs finer admission.
            request.claim(
                copied
                    .checked_mul(3)
                    .map_err(map_node_edit_accounting)?
                    .bytes(),
            )?;
        }
        match &self.ledger {
            Some(ledger) => self
                .staged
                .record_retained_lineage(record, ledger, work)
                .map_err(map_lineage_denial),
            None => {
                self.staged.record_lineage_record(record);
                Ok(())
            }
        }
    }

    pub(crate) fn record_facts(
        &mut self,
        explanation: Option<ExplanationFact>,
        provenance: Option<ProvenanceFact>,
        work: &mut Work,
        mut request: Option<&mut SignalPreparationBudget>,
    ) -> Result<(), SignalError> {
        if explanation.is_none() && provenance.is_none() {
            return Ok(());
        }
        if self.recorded_fact_nodes >= self.maximum_fact_nodes {
            return Err(SignalError::invalid_input(
                "epoch diagnostics exceeded declared producers",
            ));
        }
        self.recorded_fact_nodes += 1;
        if let Some(fact) = explanation {
            if self.staged.installed_retention_budget.explanation_retention
                == crate::diagnostics::policy::ArtifactRetentionPolicy::Retain
            {
                work.reserve_visits(self.staged.explanation_facts.lookup_steps())
                    .map_err(map_node_edit_accounting)?;
                let replaced = self
                    .staged
                    .explanation_facts
                    .get(&fact.node)
                    .map(|old| old.retained_heap_charge(work))
                    .transpose()
                    .map_err(map_node_edit_accounting)?
                    .unwrap_or(Charge::ZERO);
                let charge = fact
                    .retained_heap_charge(work)
                    .and_then(|charge| charge.checked_add(replaced))
                    .and_then(|charge| {
                        charge.checked_add(arc_allocation_charge::<ExplanationFact>()?)
                    })
                    .and_then(|charge| {
                        charge.checked_add(arc_allocation_charge::<crate::data::handle::NodeId>()?)
                    })
                    .map_err(map_node_edit_accounting)?;
                self.admit_fact_payload(charge, request.as_deref_mut())?;
                work.reserve_visits(
                    self.staged
                        .explanation_facts
                        .lookup_steps()
                        .saturating_mul(32),
                )
                .map_err(map_node_edit_accounting)?;
                accounted(
                    self.staged
                        .explanation_facts
                        .insert_with_retained_charge(fact.node, fact, work),
                )?;
            }
        }
        if let Some(fact) = provenance {
            if self.staged.installed_retention_budget.provenance_retention
                == crate::diagnostics::policy::ArtifactRetentionPolicy::Retain
            {
                work.reserve_visits(self.staged.provenance_facts.lookup_steps())
                    .map_err(map_node_edit_accounting)?;
                let replaced = self
                    .staged
                    .provenance_facts
                    .get(&fact.node)
                    .map(|old| old.retained_heap_charge(work))
                    .transpose()
                    .map_err(map_node_edit_accounting)?
                    .unwrap_or(Charge::ZERO);
                let charge = fact
                    .retained_heap_charge(work)
                    .and_then(|charge| charge.checked_add(replaced))
                    .and_then(|charge| {
                        charge.checked_add(arc_allocation_charge::<ProvenanceFact>()?)
                    })
                    .and_then(|charge| {
                        charge.checked_add(arc_allocation_charge::<crate::data::handle::NodeId>()?)
                    })
                    .map_err(map_node_edit_accounting)?;
                self.admit_fact_payload(charge, request)?;
                work.reserve_visits(
                    self.staged
                        .provenance_facts
                        .lookup_steps()
                        .saturating_mul(32),
                )
                .map_err(map_node_edit_accounting)?;
                accounted(
                    self.staged
                        .provenance_facts
                        .insert_with_retained_charge(fact.node, fact, work),
                )?;
            }
        }
        Ok(())
    }

    fn admit_fact_payload(
        &mut self,
        charge: Charge,
        request: Option<&mut SignalPreparationBudget>,
    ) -> Result<(), SignalError> {
        // The index structure for every producer was admitted at packet
        // creation. Each selected payload is admitted before its map edit.
        let growth = charge.checked_mul(2).map_err(map_node_edit_accounting)?;
        if let Some(request) = request {
            request.claim(growth.bytes())?;
        }
        if let Some(resources) = self.fact_resources.as_mut() {
            resources.grow(growth).map_err(map_node_edit_retention)?;
        }
        Ok(())
    }

    pub(crate) fn publish(mut self, live: &mut DiagnosticsState) {
        self.staged.fact_custody = self
            .fact_resources
            .take()
            .map(|resources| LineageRetentionCustody(Some(Arc::new(resources))))
            .unwrap_or_else(|| live.fact_custody.clone());
        live.lineage_records = self.staged.lineage_records;
        live.lineage_records_by_artifact = self.staged.lineage_records_by_artifact;
        live.lineage_records_by_node = self.staged.lineage_records_by_node;
        live.explanation_facts = self.staged.explanation_facts;
        live.provenance_facts = self.staged.provenance_facts;
        live.next_lineage_artifact_id = self.staged.next_lineage_artifact_id;
        live.next_lineage_sequence = self.staged.next_lineage_sequence;
        live.lineage_custody = self.staged.lineage_custody;
        live.fact_custody = self.staged.fact_custody;
    }
}

fn map_lineage_denial(denial: LineagePublicationDenial) -> SignalError {
    match denial {
        LineagePublicationDenial::Accounting(denial) => map_node_edit_accounting(denial),
        LineagePublicationDenial::Retention(denial) => map_node_edit_retention(denial),
        LineagePublicationDenial::Unprepared => SignalError::EvaluationStorageUnavailable,
    }
}

fn map_node_edit_accounting(denial: RetainedStoragePreparationDenial) -> SignalError {
    match denial {
        RetainedStoragePreparationDenial::WorkExhausted { maximum_visits } => {
            SignalError::ConditionalEvaluationWorkExhausted { maximum_visits }
        }
        _ => SignalError::EvaluationStorageCapacityExhausted,
    }
}

fn map_node_edit_retention(denial: SignalConditionalRetentionDenial) -> SignalError {
    match denial {
        SignalConditionalRetentionDenial::CapacityExhausted => {
            SignalError::EvaluationStorageCapacityExhausted
        }
        _ => SignalError::EvaluationStorageUnavailable,
    }
}

fn accounted<R>(
    outcome: Result<RetainedMapMutationOutcome<R>, RetainedMapMutationDenial>,
) -> Result<R, SignalError> {
    match outcome {
        Ok(RetainedMapMutationOutcome::Accounted { output, .. }) => Ok(output),
        Ok(RetainedMapMutationOutcome::Unaccounted { denial, .. }) => {
            Err(map_node_edit_accounting(denial))
        }
        Err(RetainedMapMutationDenial::Accounting(denial)) => Err(map_node_edit_accounting(denial)),
        Err(_) => Err(SignalError::EvaluationStorageUnavailable),
    }
}
