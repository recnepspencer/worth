//! Byte construction uses the genuine capture request; legacy policy is test-only.
use crate::durability::data::RelationalNativeCheckpointCaptureDenial as Denial;
#[cfg(test)]
use worth_execution::{ExecutionAllocationPolicy, ExecutionByteBuffer};
use worth_execution::{
    ExecutionImmutableBytes, ExecutionRequest, ExecutionRequestArrayBuilder,
    ExecutionRequestArrayDenial,
};

#[derive(Clone, Copy)]
pub(crate) enum CheckpointEncodingControl<'request, 'authority> {
    Original(ExecutionRequest<'request, 'authority>),
    #[cfg(test)]
    Policy(ExecutionAllocationPolicy<'request, 'authority>),
}
impl CheckpointEncodingControl<'_, '_> {
    pub(crate) fn checkpoint(self, work: u64) -> Result<(), Denial> {
        match self {
            Self::Original(execution) => execution
                .checkpoint_stage::<crate::execution::RelationalExecutionDenialCause>(work)
                .map_err(Denial::Request),
            #[cfg(test)]
            Self::Policy(policy) => policy.check_live().map_err(Denial::Allocation),
        }
    }
    pub(crate) fn reserve_read(
        self,
        bytes: u64,
    ) -> Result<Option<worth_execution::ExecutionMemoryReservation>, Denial> {
        match self {
            Self::Original(execution) => execution
                .reserve_retained_memory(bytes)
                .map_err(|cause| Denial::Request(cause.into())),
            #[cfg(test)]
            Self::Policy(_) => Ok(None),
        }
    }
    pub(crate) fn locator(
        self,
        bytes: &[u8],
    ) -> Result<crate::durability::medium::RelationalDurableMemberLocator, Denial> {
        match self {
            Self::Original(execution) => {
                crate::durability::log::native_checkpoint_codec::member_frame::locator_for_request(
                    &execution, bytes,
                )
                .map_err(Denial::Request)
            }
            #[cfg(test)]
            Self::Policy(_) => {
                Ok(crate::durability::log::native_checkpoint_codec::member_frame::locator(bytes))
            }
        }
    }
}
pub(super) enum CheckpointEncodingBuffer<'request, 'authority> {
    Original(ExecutionRequestArrayBuilder<'request, 'authority, u8>),
    #[cfg(test)]
    Policy(ExecutionByteBuffer),
}
impl<'request, 'authority> CheckpointEncodingBuffer<'request, 'authority> {
    pub(super) fn allocate(
        control: CheckpointEncodingControl<'request, 'authority>,
        bytes: usize,
    ) -> Result<Self, Denial> {
        match control {
            CheckpointEncodingControl::Original(execution) => {
                ExecutionRequestArrayBuilder::allocate(execution, bytes)
                    .map(Self::Original)
                    .map_err(allocation)
            }
            #[cfg(test)]
            CheckpointEncodingControl::Policy(policy) => {
                ExecutionByteBuffer::allocate(bytes, policy)
                    .map(Self::Policy)
                    .map_err(Denial::Allocation)
            }
        }
    }
    pub(super) fn len(&self) -> usize {
        match self {
            Self::Original(bytes) => bytes.len(),
            #[cfg(test)]
            Self::Policy(bytes) => bytes.len(),
        }
    }
    pub(super) fn append(&mut self, bytes: &[u8]) -> Result<(), Denial> {
        match self {
            Self::Original(output) => output.extend_from_slice(bytes).map_err(allocation),
            #[cfg(test)]
            Self::Policy(output) => output.extend_from_slice(bytes).map_err(Denial::Allocation),
        }
    }
    pub(super) fn seal(self) -> Result<ExecutionImmutableBytes, Denial> {
        match self {
            Self::Original(output) => output
                .seal()
                .map(|bytes| bytes.into_immutable_bytes())
                .map_err(allocation),
            #[cfg(test)]
            Self::Policy(output) => output.seal().map_err(Denial::Allocation),
        }
    }
}
fn allocation(cause: ExecutionRequestArrayDenial) -> Denial {
    match cause {
        ExecutionRequestArrayDenial::Request(cause) => Denial::Request(cause.into()),
        ExecutionRequestArrayDenial::Allocation(cause) => Denial::Allocation(cause),
    }
}
