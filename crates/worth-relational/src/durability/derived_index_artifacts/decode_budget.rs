//! One invoking request funds the actual index decode and its retained owners.
use crate::durability::data::RelationalRecoveryStop as Stop;
use worth_execution::{
    ExecutionArray, ExecutionMemoryReservation, ExecutionRequest, ExecutionRequestArrayBuilder,
};

#[derive(Default)]
pub(super) struct IndexDecodeBudget<'r, 'a> {
    request: Option<ExecutionRequest<'r, 'a>>,
    funds: Option<ExecutionRequestArrayBuilder<'r, 'a, Option<ExecutionMemoryReservation>>>,
}
impl<'r, 'a> IndexDecodeBudget<'r, 'a> {
    pub(super) fn new(request: ExecutionRequest<'r, 'a>, count: usize) -> Result<Self, Stop> {
        let slots = count
            .checked_mul(2)
            .and_then(|n| n.checked_add(3))
            .ok_or_else(overflow)?;
        Ok(Self {
            request: Some(request),
            funds: Some(
                ExecutionRequestArrayBuilder::allocate(request, slots).map_err(array_stop)?,
            ),
        })
    }
    pub(super) fn transient(
        &self,
        bytes: usize,
    ) -> Result<Option<ExecutionMemoryReservation>, Stop> {
        self.request
            .map(|request| {
                request
                    .reserve_retained_memory(bytes as u64)
                    .map_err(|cause| Stop::Request(cause.into()))
            })
            .transpose()
            .map(Option::flatten)
    }
    pub(super) fn consult(&self) -> Result<(), Stop> {
        if let Some(request) = self.request {
            request
                .consult()
                .map_err(|cause| Stop::Request(cause.into()))?;
        }
        Ok(())
    }
    pub(super) fn request(&self) -> Option<ExecutionRequest<'r, 'a>> {
        self.request
    }
    pub(super) fn reserve(&mut self, bytes: u64) -> Result<(), Stop> {
        if let Some(request) = self.request {
            request
                .consult()
                .map_err(|cause| Stop::Request(cause.into()))?;
            let memory = request
                .reserve_retained_memory(bytes)
                .map_err(|cause| Stop::Request(cause.into()))?;
            self.funds
                .as_mut()
                .expect("controlled decode retains its fixed funding roster")
                .push(memory)
                .map_err(array_stop)?;
        }
        Ok(())
    }
    pub(super) fn finish(self) -> Result<ExecutionArray<Option<ExecutionMemoryReservation>>, Stop> {
        {
            let mut funds = self
                .funds
                .expect("only controlled decode exports funded DATA");
            while funds.len() < funds.element_count() {
                funds.push(None).map_err(array_stop)?;
            }
            funds.seal().map_err(array_stop)
        }
    }
}
pub(super) fn array_stop(cause: worth_execution::ExecutionRequestArrayDenial) -> Stop {
    match cause {
        worth_execution::ExecutionRequestArrayDenial::Request(cause) => Stop::Request(cause.into()),
        worth_execution::ExecutionRequestArrayDenial::Allocation(cause) => Stop::Allocation(cause),
    }
}
pub(super) fn overflow() -> Stop {
    Stop::Request(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
}
pub(super) fn wire_stop(
    cause: crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal,
) -> Stop {
    use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal as Refusal;
    match cause {
        Refusal::Request(cause) => Stop::Request(cause),
        cause => {
            super::corrupt(&format!("index checkpoint decode quote refused: {cause:?}")).into()
        }
    }
}
