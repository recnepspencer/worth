//! Aggregate phase execution and exclusive work-accounting construction.

mod work_accounting;

pub(super) use work_accounting::AggregateWorkAccounting;

use super::validated_plan::ValidatedAggregatePlan;
use super::{
    cache_resolution, completed_scan, WorthQueryInvariantAggregate,
    WorthQueryInvariantAggregateDenial,
};
use crate::domain_computation::primary_graph::invariant_projection::WorthQueryApplicationInvariantProjectionReader;
use worth_query_installation::facade::{ApplicationSchema, ApplicationSignedAggregateValueBinding};

pub(super) fn execute<Schema, Binding>(
    reader: &mut WorthQueryApplicationInvariantProjectionReader<'_, Schema>,
    plan: ValidatedAggregatePlan,
) -> Result<WorthQueryInvariantAggregate<Binding::Value>, WorthQueryInvariantAggregateDenial>
where
    Schema: ApplicationSchema,
    Binding: ApplicationSignedAggregateValueBinding,
{
    let cache = std::sync::Arc::clone(&reader.aggregate_projections);
    let mut accounting = AggregateWorkAccounting::new(&mut reader.work, &mut reader.work_budget);
    match cache_resolution::probe(&cache, plan, &mut accounting)?.resolve() {
        Ok(hit) => hit.complete::<Binding>(&mut reader.realized_scope),
        Err(miss) => {
            completed_scan::complete(reader.runtime, reader.snapshot, miss, &mut accounting)
                .and_then(|scan| scan.publish::<Binding>(&cache, &mut reader.realized_scope))
        }
    }
}
