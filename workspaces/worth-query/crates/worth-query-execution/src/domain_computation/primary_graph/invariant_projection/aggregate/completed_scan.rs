//! Cold aggregate reconstruction and sealed completed-scan publication.

use std::sync::{Arc, Mutex};

use worth_foundational::facade::AspectValue;
use worth_query_installation::facade::ApplicationSignedAggregateValueBinding;

use super::cache_resolution::UncachedAggregatePlan;
use super::execution::AggregateWorkAccounting;
use super::validated_plan::ValidatedAggregatePlan;
use super::{
    denial, WorthQueryInvariantAggregate, WorthQueryInvariantAggregateDenial,
    WorthQueryInvariantAggregateDenialKind,
};
use crate::domain_computation::primary_graph::aggregate_projection::WorthQueryAggregateProjections;
use crate::domain_computation::primary_graph::invariant_projection::WorthQueryRealizedProjectionScope;

#[derive(Debug)]
pub(super) struct CompletedAggregateScan {
    plan: ValidatedAggregatePlan,
    sum: i64,
    source_count: u64,
}

pub(super) fn complete(
    runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    miss: UncachedAggregatePlan,
    accounting: &mut AggregateWorkAccounting<'_>,
) -> Result<CompletedAggregateScan, WorthQueryInvariantAggregateDenial> {
    let plan = miss.into_plan();
    let accumulator = AggregateScanReader {
        runtime,
        snapshot,
        plan: &plan,
        accounting,
    }
    .rebuild()?;
    Ok(CompletedAggregateScan {
        plan,
        sum: accumulator.sum,
        source_count: accumulator.source_count,
    })
}

struct AggregateScanReader<'scan, 'work> {
    runtime: &'scan mut worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &'scan worth_relational::facade::snapshots::SnapshotHandle,
    plan: &'scan ValidatedAggregatePlan,
    accounting: &'scan mut AggregateWorkAccounting<'work>,
}

impl AggregateScanReader<'_, '_> {
    fn rebuild(&mut self) -> Result<AggregateScanAccumulator, WorthQueryInvariantAggregateDenial> {
        let read = self
            .runtime
            .read_truth()
            .bounded_incoming_relations_of_kind_at_version(
                self.plan.target(),
                self.plan.key().relation_kind,
                self.plan.version(),
                self.accounting.remaining(),
            )
            .map_err(|limit| {
                self.accounting
                    .reject_initial_adjacency(limit, self.plan.relation_member())
            })?;
        let row_count = read.relation_records_examined();
        self.accounting.record_cold_lookup(row_count);
        self.accounting.complete_adjacency(
            read.work_units(),
            row_count,
            read.endpoint_records_reserved(),
        );
        let mut accumulator = AggregateScanAccumulator::new();
        for record in read.into_records() {
            let value = self.observe_source(&record)?;
            accumulator.push(value, self.plan)?;
        }
        Ok(accumulator)
    }

    fn observe_source(
        &mut self,
        record: &worth_relational::facade::runtime::RelationReadRecord,
    ) -> Result<Option<AspectValue>, WorthQueryInvariantAggregateDenial> {
        let relations = self
            .runtime
            .read_truth()
            .bounded_outgoing_relations_of_kind_at_version(
                record.source,
                self.plan.key().relation_kind,
                self.plan.version(),
                self.accounting.remaining(),
            )
            .map_err(|limit| {
                self.accounting
                    .reject_bounded_adjacency(limit, self.plan.relation_member())
            })?;
        self.accounting.complete_adjacency(
            relations.work_units(),
            relations.relation_records_examined(),
            relations.endpoint_records_reserved(),
        );
        if relations.into_records().as_slice() != [record.clone()] {
            return Err(denial(
                WorthQueryInvariantAggregateDenialKind::AmbiguousSourceRelation,
                self.plan.relation_member(),
            ));
        }
        self.accounting
            .admit_source_field(self.plan.field_member())?;
        let value =
            crate::domain_computation::primary_graph::application_attempt::observe_field_value(
                self.runtime,
                self.snapshot,
                record.source,
                self.plan.source_kind(),
                &self.plan.key().field,
            );
        Ok(value)
    }
}

impl CompletedAggregateScan {
    pub(super) fn publish<Binding>(
        self,
        cache: &Arc<Mutex<WorthQueryAggregateProjections>>,
        scope: &mut WorthQueryRealizedProjectionScope,
    ) -> Result<WorthQueryInvariantAggregate<Binding::Value>, WorthQueryInvariantAggregateDenial>
    where
        Binding: ApplicationSignedAggregateValueBinding,
    {
        let value = Binding::decode_aggregate(self.sum).map_err(|_| {
            denial(
                WorthQueryInvariantAggregateDenialKind::InvalidScalar,
                self.plan.field_member(),
            )
        })?;
        scope.record(self.plan.target());
        cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .retain_aggregate(
                self.plan.key().clone(),
                self.plan.target(),
                self.plan.version(),
                self.sum,
                self.source_count,
            );
        Ok(WorthQueryInvariantAggregate {
            value,
            source_count: self.source_count,
        })
    }
}

struct AggregateScanAccumulator {
    sum: i64,
    source_count: u64,
}

impl AggregateScanAccumulator {
    const fn new() -> Self {
        Self {
            sum: 0,
            source_count: 0,
        }
    }

    fn push(
        &mut self,
        value: Option<AspectValue>,
        plan: &ValidatedAggregatePlan,
    ) -> Result<(), WorthQueryInvariantAggregateDenial> {
        let Some(AspectValue::Int64(value)) = value else {
            return Err(denial(
                WorthQueryInvariantAggregateDenialKind::InvalidScalar,
                plan.field_member(),
            ));
        };
        self.sum = self.sum.checked_add(value).ok_or_else(|| {
            denial(
                WorthQueryInvariantAggregateDenialKind::ArithmeticOverflow,
                plan.field_member(),
            )
        })?;
        // On supported targets a scan cannot contain more than `usize::MAX`
        // records, so this is defensive for a future target wider than u64. We
        // intentionally do not fabricate an impossible production cardinality
        // merely to make this branch executable in a test.
        self.source_count = self.source_count.checked_add(1).ok_or_else(|| {
            denial(
                WorthQueryInvariantAggregateDenialKind::SourceCountOverflow,
                plan.relation_member(),
            )
        })?;
        Ok(())
    }
}
