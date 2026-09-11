//! Cache lookup as a sealed hit-or-rebuild transition.

use std::sync::{Arc, Mutex};

use worth_query_installation::facade::ApplicationSignedAggregateValueBinding;

use super::execution::AggregateWorkAccounting;
use super::validated_plan::ValidatedAggregatePlan;
use super::{WorthQueryInvariantAggregate, WorthQueryInvariantAggregateDenial};
use crate::domain_computation::primary_graph::aggregate_projection::{
    WorthQueryAggregateProjections, WorthQueryIncomingAggregate,
};
use crate::domain_computation::primary_graph::invariant_projection::WorthQueryRealizedProjectionScope;

#[derive(Debug)]
pub(super) struct AggregateCacheResolution {
    outcome: CacheOutcome,
}

#[derive(Debug)]
enum CacheOutcome {
    Hit(CachedAggregate),
    Miss(UncachedAggregatePlan),
}

#[derive(Debug)]
pub(super) struct CachedAggregate {
    aggregate: WorthQueryIncomingAggregate,
    target: worth_relational::facade::identity::EntityId,
    field_member: String,
}

#[derive(Debug)]
pub(super) struct UncachedAggregatePlan {
    plan: ValidatedAggregatePlan,
}

pub(super) fn probe(
    cache: &Arc<Mutex<WorthQueryAggregateProjections>>,
    plan: ValidatedAggregatePlan,
    accounting: &mut AggregateWorkAccounting<'_>,
) -> Result<AggregateCacheResolution, WorthQueryInvariantAggregateDenial> {
    accounting.admit_cache_lookup(plan.relation_member())?;
    let cached = cache
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .cached_aggregate(plan.key(), plan.target(), plan.version());
    let outcome = match cached {
        Some(aggregate) => {
            accounting.complete_warm_lookup();
            CacheOutcome::Hit(CachedAggregate {
                aggregate,
                target: plan.target(),
                field_member: plan.field_member().to_owned(),
            })
        }
        None => CacheOutcome::Miss(UncachedAggregatePlan { plan }),
    };
    Ok(AggregateCacheResolution { outcome })
}

impl AggregateCacheResolution {
    pub(super) fn resolve(self) -> Result<CachedAggregate, UncachedAggregatePlan> {
        match self.outcome {
            CacheOutcome::Hit(hit) => Ok(hit),
            CacheOutcome::Miss(miss) => Err(miss),
        }
    }
}

impl CachedAggregate {
    pub(super) fn complete<Binding>(
        self,
        scope: &mut WorthQueryRealizedProjectionScope,
    ) -> Result<WorthQueryInvariantAggregate<Binding::Value>, WorthQueryInvariantAggregateDenial>
    where
        Binding: ApplicationSignedAggregateValueBinding,
    {
        scope.record(self.target);
        let value = Binding::decode_aggregate(self.aggregate.sum).map_err(|_| {
            super::denial(
                super::WorthQueryInvariantAggregateDenialKind::InvalidScalar,
                self.field_member,
            )
        })?;
        Ok(WorthQueryInvariantAggregate {
            value,
            source_count: self.aggregate.source_count,
        })
    }
}

impl UncachedAggregatePlan {
    pub(super) fn into_plan(self) -> ValidatedAggregatePlan {
        self.plan
    }
}
