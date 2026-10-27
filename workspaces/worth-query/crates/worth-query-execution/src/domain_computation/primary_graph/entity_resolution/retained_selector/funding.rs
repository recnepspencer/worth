//! Exact retained scope backing stays funded after its original request returns.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::execution_denial::{
    publication_execution_kind, scope_stop_kind,
};
use crate::domain_computation::{
    WorthQueryProviderSessionControlStopKind, WorthQueryProviderSessionDenialKind,
};
use worth_execution::{
    ChargedBytes, ExecutionMemoryReservation, ExecutionRequest, ExecutionWorkCeiling, LeaseDenial,
    MapKernelFailure, WorkCeilingDenial,
};

type Stop = Result<WorthQueryProviderSessionDenialKind, WorthQueryProviderSessionControlStopKind>;

struct Captured(Arc<WorthQueryObservedScopeSelector>);
struct ReservationStop(WorkCeilingDenial);
impl ChargedBytes for ReservationStop {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
impl ChargedBytes for Captured {
    fn additional_charged_bytes(&self) -> u64 {
        bytes(&self.0.locator, &self.0.value).unwrap_or(u64::MAX)
    }
}

fn bytes(locator: &AspectFieldLocator, value: &AspectValue) -> Option<u64> {
    std::mem::size_of::<WorthQueryObservedScopeSelector>()
        .checked_add(2 * std::mem::size_of::<usize>())?
        .checked_add(locator.owned_allocation_capacity_bytes())?
        .checked_add(value.owned_allocation_capacity_bytes())?
        .try_into()
        .ok()
}

impl<Schema, Entity> WorthQueryApplicationEntityIdentity<Schema, Entity> {
    pub(in crate::domain_computation) fn retain_scope_selector_funded(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<WorthQueryObservedScopeSelector>, Stop> {
        let mut settled = None;
        let handoff = execution.run_native_with_checkpoint(|context| {
            settled = Some(if context.has_accounting() {
                self.retain_accounted_scope_selector(execution)
            } else {
                context
                    .poll()
                    .map(|()| self.retain_scope_selector())
                    .map_err(|cause| publication_execution_kind(WorkCeilingDenial::Stopped(cause)))
            });
        });
        match (handoff, settled) {
            (Ok(_), Some(result)) => result,
            (
                Err(WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::NestedStopped)),
                Some(Err(cause)),
            ) => Err(cause),
            (Err(cause), _) => Err(publication_execution_kind(cause)),
            (Ok(_), None) => unreachable!("successful source scope capture invokes its stage"),
        }
    }

    fn retain_accounted_scope_selector(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<WorthQueryObservedScopeSelector>, Stop> {
        let overflow = || {
            publication_execution_kind(WorkCeilingDenial::Admission(
                LeaseDenial::ChargedBytesOverflow,
            ))
        };
        let required =
            bytes(self.identity_locator(), self.identity_value()).ok_or_else(overflow)?;
        let mut memory = execution
            .in_scope(|lease| ExecutionMemoryReservation::reserve_in_scope(lease, 0))
            .map_err(publication_execution_kind)?
            .map_err(|cause| publication_execution_kind(WorkCeilingDenial::Admission(cause)))?;
        let measured = execution.run_measured_result(
            ExecutionWorkCeiling::new(None),
            0,
            &mut memory,
            |context, retained| {
                let work = self
                    .identity_locator()
                    .owned_allocation_capacity_bytes()
                    .checked_add(self.identity_value().semantic_byte_width())
                    .and_then(|value| value.checked_add(1))
                    .and_then(|value| u64::try_from(value).ok())
                    .ok_or(MapKernelFailure::Stop(
                        worth_execution::MapKernelStop::WorkCounterOverflow,
                    ))?;
                context.checkpoint(work)?;
                retained.resize(required).map_err(|cause| {
                    MapKernelFailure::Domain(ReservationStop(WorkCeilingDenial::Admission(
                        LeaseDenial::MemoryExhausted(cause),
                    )))
                })?;
                Ok(Captured(self.retain_scope_selector()))
            },
        );
        let mut captured = measured
            .result
            .map_err(|stop| match scope_stop_kind(stop) {
                Ok(cause) => publication_execution_kind(cause.0),
                Err(stop) => stop,
            })?;
        Arc::get_mut(&mut captured.0)
            .expect("fresh scope capture has not escaped")
            ._funding = Some(memory);
        Ok(captured.0)
    }
}
