//! The lineage row of a republished generated output.

use std::any::TypeId;
use std::sync::{Arc, OnceLock};

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

use super::super::{
    input_cutoff::performed_fact_sequence, invalidation::InvalidationEditAdmission,
    latest_output_matching, retained_capacity::RetainedLineageCapacity,
    CompletedDecisionReuseProof, PreparedInputReuseKey, RecordedOutput, RecordedSettlementIdentity,
    RecordedSourceIdentity, SealedNativeOutputWitness, SemanticSource,
    WorthQueryApplicationOutputCorrespondence, WorthQueryApplicationOutputLineage,
};
use crate::domain_computation::primary_graph::{
    application_attempt::{
        CompletedHandlerFactBoundary, WorthQueryApplicationObservedFact as Fact,
    },
    application_contribution::WorthQueryProducerDemandResources,
    invariant_projection::ConsumedOutputEvidence,
};

/// What a restoration carries over from the retained performed record it
/// republishes. Restoring a generated output re-creates that record's exact
/// output on the runtime that performed it, so the completed read, the
/// prepared input and the consumed outputs are still the ones that produced
/// it. Only the output witness is new: the re-created entities carry new
/// revisions, sealed at the restoration commit.
pub(in crate::domain_computation::primary_graph) struct RepublishedOutput {
    predecessor: Arc<RecordedSettlementIdentity>,
    facts: Arc<[Fact]>,
    retained_capacity: Option<RetainedLineageCapacity>,
    consumed_outputs: Arc<[ConsumedOutputEvidence]>,
    completed_handler_facts: CompletedHandlerFactBoundary,
    completed_decision_reuse: CompletedDecisionReuseProof,
    prepared_input_reuse_key: PreparedInputReuseKey,
    witness: Arc<OnceLock<SealedNativeOutputWitness>>,
}

/// A recorded republication and what its mark row registers.
pub(in crate::domain_computation::primary_graph) struct RepublishedRecord {
    pub(in crate::domain_computation::primary_graph) identity: Arc<RecordedSettlementIdentity>,
    /// The suspended record this row continues.
    pub(in crate::domain_computation::primary_graph) predecessor: Arc<RecordedSettlementIdentity>,
    pub(in crate::domain_computation::primary_graph) facts: Arc<[Fact]>,
    pub(in crate::domain_computation::primary_graph) consumed_outputs:
        Arc<[ConsumedOutputEvidence]>,
    pub(in crate::domain_computation::primary_graph) witness:
        Arc<OnceLock<SealedNativeOutputWitness>>,
}

impl RepublishedOutput {
    pub(super) fn facts(&self) -> Arc<[Fact]> {
        Arc::clone(&self.facts)
    }

    /// Replace a new restored row's checkpoint posture with the performed
    /// record this republication continues. The row is a performed origin of
    /// its own: its witness covers the re-created entities.
    pub(super) fn continue_in(self, row: &mut RecordedOutput) {
        row._retained_capacity = self.retained_capacity;
        row.consumed_outputs = self.consumed_outputs;
        row.completed_handler_facts = Some(self.completed_handler_facts);
        row.completed_decision_reuse = Some(self.completed_decision_reuse);
        row.prepared_input_reuse_key = Some(self.prepared_input_reuse_key);
        row.native_output_witness = OnceLock::from(self.witness);
        let mutable = row
            .mutable
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        mutable.verification_requirement = None;
        mutable.observed_source_facts = Some(self.facts);
        mutable.computation = None;
    }
}

impl WorthQueryApplicationOutputLineage {
    /// Find the record a suspension qualified and prepare its continuation.
    /// The suspended record is the latest one at or before the suspension
    /// that holds the suspended correspondence and source facts themselves.
    /// `None` means that record retained no performed proof to continue, or
    /// this preparation was not admitted: the restoration is then recorded as
    /// a restored row, Fresh until verified.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation::primary_graph) fn prepare_republication(
        &self,
        output_binding: TypeId,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        suspended_occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        suspended_generation: u64,
        source_partition_identity: [u8; 32],
        correspondence: &Arc<WorthQueryApplicationOutputCorrespondence>,
        suspended_facts: &Arc<[Fact]>,
        witness: Arc<OnceLock<SealedNativeOutputWitness>>,
        admission: &mut InvalidationEditAdmission,
    ) -> Option<RepublishedOutput> {
        let source = SemanticSource {
            runtime_authority,
            schema: schema.clone(),
            scope,
            output_binding,
        };
        let history = self.by_source.get(&source)?.get(&suspended_occurrence)?;
        let suspended = latest_output_matching(history, suspended_generation, |recorded| {
            recorded.source_partition_identity == Some(source_partition_identity)
                && Arc::ptr_eq(&recorded.correspondence, correspondence)
        })?;
        if !suspended
            .observed_source_facts()
            .is_some_and(|facts| Arc::ptr_eq(&facts, suspended_facts))
        {
            return None;
        }
        let origin = match &suspended.performed_origin {
            Some(origin) => origin.get()?,
            None => suspended,
        };
        if origin.performed_origin.is_some()
            || !Arc::ptr_eq(&origin.consumed_outputs, &suspended.consumed_outputs)
        {
            return None;
        }
        let boundary = origin.completed_handler_facts.as_ref()?;
        let decision = origin.completed_decision_reuse.as_ref()?;
        let key = suspended.prepared_input_reuse_key.as_ref()?;
        let (facts, retained_capacity) = if suspended.performed_origin.is_none() {
            (Arc::clone(suspended_facts), None)
        } else {
            // A stable alias projects its origin's output into its facts. The
            // republished row seals its own witness instead.
            let (facts, capacity) = performed_fact_sequence(
                suspended_facts,
                boundary.handler_fact_count(),
                origin.native_output_witness()?,
                &self.retention,
                admission,
            )?;
            (facts, Some(capacity))
        };
        if boundary.handler_fact_count() > facts.len() {
            return None;
        }
        Some(RepublishedOutput {
            predecessor: Arc::clone(&suspended.settlement_identity),
            facts,
            retained_capacity,
            consumed_outputs: Arc::clone(&origin.consumed_outputs),
            completed_handler_facts: boundary.continued_by_republication(),
            completed_decision_reuse: decision.continued_by_republication(),
            prepared_input_reuse_key: key.continued_by_republication(),
            witness,
        })
    }

    /// Record the restoration as the continuation of its performed record.
    /// `None` means no new row took the republication: the address was
    /// already recorded or is owned by a selected World publication, exactly
    /// as for any restored row.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation::primary_graph) fn record_republished_restoration(
        &mut self,
        output_binding: TypeId,
        runtime_authority: u64,
        schema: ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
        correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
        source_identity: RecordedSourceIdentity,
        source_partition_identity: [u8; 32],
        producer_dependency_identity: Option<[u8; 32]>,
        idempotency_key_identity: [u8; 32],
        suspended_facts: Arc<[Fact]>,
        resources: Option<WorthQueryProducerDemandResources>,
        republished: RepublishedOutput,
    ) -> Option<RepublishedRecord> {
        let predecessor = Arc::clone(&republished.predecessor);
        let consumed_outputs = Arc::clone(&republished.consumed_outputs);
        let witness = Arc::clone(&republished.witness);
        let mut republished = Some(republished);
        let restored = self.record_restored_row(
            output_binding,
            runtime_authority,
            schema,
            scope,
            observation,
            correspondence,
            source_identity,
            source_partition_identity,
            producer_dependency_identity,
            idempotency_key_identity,
            suspended_facts,
            resources,
            None,
            &mut republished,
        )?;
        republished.is_none().then_some(RepublishedRecord {
            identity: restored.identity,
            predecessor,
            facts: restored.facts,
            consumed_outputs,
            witness,
        })
    }
}
