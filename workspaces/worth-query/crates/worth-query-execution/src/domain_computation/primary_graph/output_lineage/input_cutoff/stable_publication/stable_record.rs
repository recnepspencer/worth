//! Metered composition of an original handler prefix and a fresh source suffix.

use std::{mem::size_of, sync::Arc};

use worth_foundational::facade::AspectFieldLocator;
use worth_relational::facade::mvcc::CompanionPreflightStop;

use super::{PreparedStableLineageAddress, PreparedStableLineagePublication};
use crate::domain_computation::primary_graph::{
    application_query::BoundStableObservedSourceFacts,
    output_lineage::{
        invalidation::{arc_slice_bytes, retained_fact_payload_bytes, InvalidationEditAdmission},
        RecordedOutput, RecordedOutputMutable, RecordedSourceIdentity,
    },
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationObservedFact as Fact,
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind as Kind,
    WorthQueryProducerDemandResources,
};

impl<'lane, 'selected> PreparedStableLineageAddress<'lane, 'selected> {
    /// All allocation and retained custody is admitted before copying the
    /// verified handler facts or consuming the fresh source fact carrier.
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
        admission: &mut InvalidationEditAdmission,
    ) -> Result<PreparedStableLineagePublication<'lane, 'selected>, WorthQueryOutputDemandDenial>
    {
        if !fresh.matches_operation(operation) {
            return Err(denial(Kind::ForeignSource));
        }
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
            .get(..prefix_count)
            .ok_or_else(|| denial(Kind::IncompleteDependencyCoverage))?;
        let witness_facts = candidate
            .native_output_witness()
            .ok_or_else(|| denial(Kind::IncompleteDependencyCoverage))?
            .prepare_fact_projection(admission)
            .map_err(work_denial)?
            .ok_or_else(|| denial(Kind::IncompleteDependencyCoverage))?;
        let suffix = fresh.source_facts();
        let total = prefix_count
            .checked_add(witness_facts.count())
            .and_then(|count| count.checked_add(suffix.len()))
            .ok_or_else(|| denial(Kind::WorkBudgetExceeded))?;
        if fresh.bound_source().partition_identity() != self.partition {
            return Err(denial(Kind::ForeignSource));
        }

        let mut retained_payload = witness_facts.retained_payload_bytes();
        let mut cloned_payload = retained_payload;
        for fact in prefix {
            admission.charge_external_work(1).map_err(work_denial)?;
            let payload = retained_fact_payload_bytes(fact, admission).map_err(footprint_denial)?;
            retained_payload = retained_payload
                .checked_add(payload)
                .ok_or_else(|| denial(Kind::RetentionBudgetExceeded))?;
            cloned_payload = cloned_payload
                .checked_add(payload)
                .ok_or_else(|| denial(Kind::RetentionBudgetExceeded))?;
            charge_initialized_clone(fact, admission).map_err(work_denial)?;
        }
        for fact in suffix {
            admission.charge_external_work(1).map_err(work_denial)?;
            retained_payload = retained_payload
                .checked_add(
                    retained_fact_payload_bytes(fact, admission).map_err(footprint_denial)?,
                )
                .ok_or_else(|| denial(Kind::RetentionBudgetExceeded))?;
        }
        let count = u64::try_from(total).map_err(|_| denial(Kind::WorkBudgetExceeded))?;
        // Building the flat Vec initializes every row (including the cloned
        // prefix); Arc conversion copies every initialized row again.
        admission
            .charge_external_work(
                count
                    .checked_mul(2)
                    .and_then(|units| units.checked_add(5))
                    .ok_or_else(|| denial(Kind::WorkBudgetExceeded))?,
            )
            .map_err(work_denial)?;
        let vec_bytes = total
            .checked_mul(size_of::<Fact>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(|| denial(Kind::RetentionBudgetExceeded))?;
        let arc_bytes =
            arc_slice_bytes::<Fact>(total).ok_or_else(|| denial(Kind::RetentionBudgetExceeded))?;
        let peak = vec_bytes
            .checked_add(arc_bytes)
            .and_then(|bytes| bytes.checked_add(cloned_payload))
            .ok_or_else(|| denial(Kind::RetentionBudgetExceeded))?;
        admission.admit_read_scratch(peak).map_err(memory_denial)?;
        self.retained_capacity
            .as_mut()
            .expect("prepared stable address retains capacity")
            .reserve_additional(
                arc_bytes
                    .checked_add(retained_payload)
                    .ok_or_else(|| denial(Kind::RetentionBudgetExceeded))?,
            )?;

        let mut combined = Vec::with_capacity(total);
        combined.extend(prefix.iter().cloned());
        witness_facts.append_into(&mut combined);
        let bound = fresh
            .append_into_stable_facts(operation, &mut combined)
            .map_err(|_| denial(Kind::ForeignSource))?;
        let facts: Arc<[Fact]> = Arc::from(combined.into_boxed_slice());
        let performed_origin = origin_cell(candidate);
        let recorded = RecordedOutput {
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
            mutable: std::sync::Mutex::new(RecordedOutputMutable::new(
                None,
                (Some(Arc::clone(&facts)))
                    .map(|facts| selected_row.computation_source.retain_facts(facts)),
                Some(resources),
                selected_row
                    .mutable
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .computation
                    .shared(),
            )),
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
        Fact::SourceEntity { .. }
        | Fact::Entity { .. }
        | Fact::WorkflowDefinitionPredecessor { .. }
        | Fact::WorkflowDefinitionCurrent { .. } => 0,
        Fact::RetiredOutputEntity { read_locator, .. } => initialized(read_locator.len())?,
        Fact::SourceAspectRevision { aspect, .. } => initialized(aspect.as_str().len())?,
        Fact::SourceFieldRevision { locator, .. } | Fact::AbsentField { locator, .. } => {
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

fn footprint_denial(stop: CompanionPreflightStop) -> WorthQueryOutputDemandDenial {
    match stop {
        CompanionPreflightStop::WorkExhausted { .. }
        | CompanionPreflightStop::WorkCounterOverflow => denial(Kind::WorkBudgetExceeded),
        _ => denial(Kind::RetentionBudgetExceeded),
    }
}

fn memory_denial(_: CompanionPreflightStop) -> WorthQueryOutputDemandDenial {
    denial(Kind::RetentionBudgetExceeded)
}

fn denial(kind: Kind) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(kind, "stable output fact preparation")
}
