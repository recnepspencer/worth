//! Request-funded counters; a refused start never contacts the handler.
use super::super::super::execution::CoupledSourceReadStop;
use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial as Attempt;
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial as Denial, WorthQueryOutputDemandDenialKind as Kind,
};
use std::num::NonZeroUsize;
use worth_execution::{ExecutionOrderedMap, ExecutionOrderedMapDenial, ExecutionRequest};
use worth_relational::facade::identity::EntityId;
type Route = (EntityId, &'static str, [u8; 32]);
pub(in crate::domain_computation::primary_graph::application_contribution::producer) struct ProducerRoundBudget
{
    bounds: (NonZeroUsize, NonZeroUsize),
    routes: ExecutionOrderedMap<Route, usize>,
    total: usize,
}
impl ProducerRoundBudget {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn new(
        bounds: (NonZeroUsize, NonZeroUsize),
    ) -> Self {
        Self {
            bounds,
            routes: ExecutionOrderedMap::default(),
            total: 0,
        }
    }
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn start(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        source: EntityId,
        family: &'static str,
        partition: [u8; 32],
        producer: &str,
    ) -> Result<(), CoupledSourceReadStop> {
        super::super::checkpoint(
            execution,
            producer,
            producer
                .len()
                .saturating_add(family.len())
                .saturating_add(self.routes.len().max(1).ilog2() as usize + 1) as u64,
        )
        .map_err(|cause| cause.0)?;
        let key = (source, family, partition);
        let started = self.routes.get(&key).copied().unwrap_or(0);
        let refuse = |kind| Denial::new(kind, format!("{producer}: {family} at {source:?}"));
        if started >= self.bounds.0.get() {
            return Err(refuse(Kind::ProducerRouteRoundBudgetExceeded {
                source,
                started,
                limit: self.bounds.0.get(),
            })
            .into());
        }
        if self.total >= self.bounds.1.get() {
            return Err(refuse(Kind::ProducerAttemptRoundBudgetExceeded {
                source,
                started: self.total,
                limit: self.bounds.1.get(),
            })
            .into());
        }
        self.routes
            .insert_cloned_native_for_request(&key, &(started + 1), *execution)
            .map_err(|cause| match cause {
                ExecutionOrderedMapDenial::Allocation(cause) => {
                    CoupledSourceReadStop::capture(Attempt::allocation_denied(producer, cause))
                }
                ExecutionOrderedMapDenial::Request(cause) => CoupledSourceReadStop::capture(
                    Attempt::relational_read_stopped(cause.into(), producer),
                ),
                ExecutionOrderedMapDenial::CapacityOverflow => {
                    refuse(Kind::RetentionBudgetExceeded).into()
                }
            })?;
        self.total += 1;
        Ok(())
    }
}
