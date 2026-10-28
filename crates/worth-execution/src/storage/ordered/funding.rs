//! Allocation custody for the sole ordered-cell construction algorithm.
use super::ExecutionOrderedMapDenial as Denial;
use crate::{ExecutionAllocationPolicy, ExecutionMemoryReservation, ExecutionRequest};

#[derive(Clone, Copy)]
pub(super) enum Funding<'request, 'authority> {
    Policy(ExecutionAllocationPolicy<'request, 'authority>),
    Request {
        request: ExecutionRequest<'request, 'authority>,
        accounted: bool,
    },
}
impl<'request, 'authority> From<ExecutionAllocationPolicy<'request, 'authority>>
    for Funding<'request, 'authority>
{
    fn from(policy: ExecutionAllocationPolicy<'request, 'authority>) -> Self {
        Self::Policy(policy)
    }
}
impl Funding<'_, '_> {
    pub(super) fn uses_accounting(self) -> bool {
        match self {
            Self::Policy(policy) => policy.uses_accounting(),
            Self::Request { accounted, .. } => accounted,
        }
    }
    pub(super) fn reserve_memory(
        self,
        bytes: u64,
    ) -> Result<Option<ExecutionMemoryReservation>, Denial> {
        match self {
            Self::Policy(policy) => policy.reserve_memory(bytes).map_err(Denial::Allocation),
            Self::Request { request, .. } => request
                .reserve_retained_memory(bytes)
                .map_err(Denial::Request),
        }
    }
}
