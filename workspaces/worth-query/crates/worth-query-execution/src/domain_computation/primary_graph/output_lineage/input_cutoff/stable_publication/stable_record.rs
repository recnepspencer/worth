//! Metered composition of an original handler prefix and a fresh source suffix.

use std::sync::Arc;

use worth_foundational::facade::AspectFieldLocator;
use worth_relational::facade::mvcc::CompanionPreflightStop;

use super::{PreparedStableLineageAddress, PreparedStableLineagePublication, StableFactSpans};
use crate::domain_computation::primary_graph::{
    application_attempt::retained_decision_facts::StorageControl,
    application_checkpoint::facts::FactPayload,
    application_query::BoundStableObservedSourceFacts,
    output_lineage::{
        invalidation::InvalidationEditAdmission, RecordedOutput, RecordedOutputMutable,
        RecordedSourceIdentity,
    },
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationObservedFact as Fact,
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind as Kind,
    WorthQueryProducerDemandResources,
};

impl<'lane, 'selected> PreparedStableLineageAddress<'lane, 'selected> {
    /// All allocation and retained custody is admitted before copying the
    /// fresh suffix facts or consuming the exact source binding. Shared
    /// handler facts are never flattened or deep-copied.
    pub(in crate::domain_computation::primary_graph) fn prepare_record<
        Schema,
        Operation,
        Input,
        Scope,
    >(
        mut self,
        fresh: BoundStableObservedSourceFacts,
        operation: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        resources: WorthQueryProducerDemandResources,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<PreparedStableLineagePublication<'lane, 'selected>, WorthQueryOutputDemandDenial>
    {
        if !fresh.matches_operation(operation) {
            return Err(denial(Kind::ForeignSource));
        }
        let control = StorageControl::new(
            execution.allocation_policy(),
            Some(operation.publication_request()),
        );
        control.check_live().map_err(super::storage_denial)?;
        let fresh_key = self
            .verified
            .fresh_key
            .take()
            .ok_or_else(|| denial(Kind::IncompleteDependencyCoverage))?;
        let candidate = &self.verified.candidate;
        let origin = candidate
            .originating_recorded()
            .ok_or_else(|| denial(Kind::IncompleteDependencyCoverage))?;
        let selected_row = candidate.recorded();
        let prefix_count = candidate
            .completed_handler_fact_count()
            .ok_or_else(|| denial(Kind::IncompleteDependencyCoverage))?;
        let prefix = self
            .verified
            .verified_facts
            .checked_slice(0..prefix_count)
            .ok_or_else(|| denial(Kind::IncompleteDependencyCoverage))?;
        let witness_facts = candidate
            .native_output_witness()
            .ok_or_else(|| denial(Kind::IncompleteDependencyCoverage))?
            .prepare_fact_projection(admission)
            .map_err(work_denial)?
            .ok_or_else(|| denial(Kind::IncompleteDependencyCoverage))?;
        let witness_count = witness_facts.count();
        if fresh.bound_source().partition_identity() != self.partition {
            return Err(denial(Kind::ForeignSource));
        }
        prefix_count
            .checked_add(witness_count)
            .and_then(|count| count.checked_add(fresh.source_facts().len()))
            .ok_or_else(|| denial(Kind::WorkBudgetExceeded))?;
        // Only genuinely new suffix facts are copied. The verified handler
        // prefix keeps its native/decoded backing and all existing grants.
        for fact in fresh.source_facts() {
            admission.charge_external_work(1).map_err(work_denial)?;
            charge_initialized_clone(fact, admission).map_err(work_denial)?;
        }
        let suffix = FactPayload::copy_funded(fresh.source_facts(), control)
            .map_err(super::storage_denial)?;
        let witness = witness_facts
            .retain_payload(control)
            .map_err(super::storage_denial)?;
        let facts = prefix
            .append_completed(witness, control)
            .and_then(|body| body.append_completed(suffix, control))
            .map_err(super::storage_denial)?;
        let spans = StableFactSpans::new(facts.clone(), prefix_count, witness_count)
            .ok_or_else(|| denial(Kind::IncompleteDependencyCoverage))?;
        // This is the installed aggregate retention policy's complete cached
        // descriptor, not an allocation quote for a new copy of shared facts.
        self.retained_capacity
            .as_mut()
            .expect("prepared stable address retains capacity")
            .reserve_additional(
                facts
                    .retained_resident_bytes()
                    .ok_or_else(|| denial(Kind::RetentionBudgetExceeded))?,
            )?;
        let bound = fresh
            .finish_stable(operation)
            .map_err(|_| denial(Kind::ForeignSource))?;
        let performed_origin = origin_cell(candidate);
        let recorded = RecordedOutput {
            required_publication: origin.required_publication.clone(),
            native_prior_checkpoint: None,
            computation_source: selected_row.computation_source,
            _retained_capacity: None,
            performed_origin: Some(performed_origin),
            consumed_outputs: self
                .verified
                .verified_consumed
                .take()
                .unwrap_or_else(|| Arc::clone(&selected_row.consumed_outputs)),
            completed_handler_facts: None,
            completed_decision_reuse: None,
            prepared_input_reuse_key: Some(fresh_key),
            native_output_witness: std::sync::OnceLock::new(),
            settlement_identity: Arc::clone(&self.identity),
            correspondence: Arc::clone(&origin.correspondence),
            source_identity: Some(RecordedSourceIdentity::Runtime(
                bound.recorded_source_identity(),
            )),
            source_partition_identity: Some(bound.partition_identity()),
            producer_dependency_identity: origin.producer_dependency_identity,
            idempotency_key_identity: origin.idempotency_key_identity,
            mutable: std::sync::Mutex::new(
                RecordedOutputMutable::new(
                    None,
                    Some(selected_row.computation_source.retain_facts(facts.clone())),
                    Some(resources),
                    selected_row
                        .mutable
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .computation
                        .shared(),
                )
                .with_stable_fact_spans(spans),
            ),
        };
        Ok(PreparedStableLineagePublication {
            address: self,
            recorded,
            facts,
        })
    }
}

fn origin_cell(
    candidate: &super::super::RetainedInputCutoffCandidate,
) -> Arc<std::sync::OnceLock<RecordedOutput>> {
    candidate
        .recorded()
        .performed_origin
        .as_ref()
        .map(Arc::clone)
        .unwrap_or_else(|| Arc::clone(&candidate.cell))
}

pub(super) fn charge_initialized_clone(
    fact: &Fact,
    admission: &mut InvalidationEditAdmission,
) -> Result<(), CompanionPreflightStop> {
    let work = match fact {
        Fact::SourcePublication { .. }
        | Fact::SourceEntity { .. }
        | Fact::Entity { .. }
        | Fact::WorkflowDefinitionPredecessor { .. }
        | Fact::WorkflowDefinitionCurrent { .. } => 0,
        Fact::RetiredOutputEntity { read_locator, .. } => initialized(read_locator.len())?,
        Fact::SourceAspectRevision { aspect, .. } | Fact::AspectPresence { aspect, .. } => {
            initialized(aspect.as_str().len())?
        }
        Fact::SourceFieldRevision { locator, .. }
        | Fact::AbsentField { locator, .. }
        | Fact::OptionalAspectAbsent { locator, .. } => {
            return charge_locator(locator, admission);
        }
        Fact::SourceAdjacencyRevision { endpoints, .. } => initialized(endpoints.len())?,
        Fact::Field { locator, value, .. } => {
            charge_locator(locator, admission)?;
            initialized(value.semantic_byte_width())?
        }
        Fact::Relation {
            matching_relations, ..
        } => initialized(matching_relations.len())?,
        Fact::Adjacency { relations, .. } => initialized(relations.len())?,
        Fact::IndexedEntitySelection {
            locator,
            value,
            candidates,
            ..
        } => {
            charge_locator(locator, admission)?;
            initialized(value.semantic_byte_width())?
                .checked_add(initialized(candidates.len())?)
                .and_then(|n| n.checked_add(1))
                .ok_or(CompanionPreflightStop::WorkCounterOverflow)?
        }
        Fact::WorkflowInstanceCapacity { instances, .. } => initialized(instances.len())?,
        Fact::WorkflowHistoryBasis { snapshot, .. } => initialized(snapshot.branch_id().0.len())?,
    };
    admission.charge_external_work(work)
}

fn charge_locator(
    locator: &AspectFieldLocator,
    admission: &mut InvalidationEditAdmission,
) -> Result<(), CompanionPreflightStop> {
    let fields = locator.field_path().fields();
    admission.charge_external_work(initialized(fields.len())?)?;
    let mut work = initialized(locator.aspect().aspect_key().as_str().len())?;
    for field in fields {
        work = work
            .checked_add(initialized(field.as_str().len())?)
            .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
    }
    admission.charge_external_work(work)
}

fn initialized(count: usize) -> Result<u64, CompanionPreflightStop> {
    u64::try_from(count).map_err(|_| CompanionPreflightStop::WorkCounterOverflow)
}

fn work_denial(_: CompanionPreflightStop) -> WorthQueryOutputDemandDenial {
    denial(Kind::WorkBudgetExceeded)
}

fn denial(kind: Kind) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(kind, "stable output fact preparation")
}
