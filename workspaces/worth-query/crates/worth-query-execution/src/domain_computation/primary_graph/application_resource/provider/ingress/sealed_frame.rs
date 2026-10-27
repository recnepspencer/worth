use super::super::{attempt::AttemptAuthority, WorthQueryResourceProviderAttempt};
use super::backing::{lock, FrameState, IngressBacking, CHUNK_BYTES};
use super::WorthQueryResourceIngressDenial;
use std::sync::Arc;
use worth_execution::{
    ExecutionArray, ExecutionRequest, ExecutionRequestArrayBuilder, ExecutionRequestArrayDenial,
};

pub(in crate::domain_computation::primary_graph::application_resource) struct SealedIngressFrame {
    backing: Arc<IngressBacking>,
    attempt: Arc<AttemptAuthority>,
    index: usize,
    length: usize,
}

impl IngressBacking {
    pub(in crate::domain_computation::primary_graph::application_resource) fn take_frame(
        self: &Arc<Self>,
        attempt: &WorthQueryResourceProviderAttempt,
    ) -> Result<Option<SealedIngressFrame>, WorthQueryResourceIngressDenial> {
        let mut lane = lock(&self.lane);
        if !lane
            .attempt
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, &attempt.authority))
        {
            return Err(WorthQueryResourceIngressDenial::StaleAttempt);
        }
        if lane.sealed.is_some() {
            return Err(WorthQueryResourceIngressDenial::Full);
        }
        let Some(index) = lane.ready else {
            return Ok(None);
        };
        let mut frame = lock(&self.frames[index]);
        lane.ready = frame.next;
        if lane.ready.is_none() {
            lane.tail = None;
        }
        frame.next = None;
        frame.state = FrameState::Sealed;
        lane.sealed = Some(index);
        Ok(Some(SealedIngressFrame {
            backing: Arc::clone(self),
            attempt: Arc::clone(&attempt.authority),
            index,
            length: frame.length,
        }))
    }
}

impl SealedIngressFrame {
    pub(in crate::domain_computation::primary_graph::application_resource) fn length(
        &self,
    ) -> usize {
        self.length
    }
    /// A contiguous copy admitted by the fresh progression request. No ingress
    /// lock survives this operation or enters the arbitrary provider verifier.
    pub(in crate::domain_computation::primary_graph::application_resource) fn copy(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ExecutionArray<u8>, ExecutionRequestArrayDenial> {
        let mut output = ExecutionRequestArrayBuilder::allocate(*execution, self.length)?;
        for offset in 0..self.length.div_ceil(CHUNK_BYTES) {
            let bytes =
                *lock(&self.backing.chunks[self.index * self.backing.chunks_per_frame + offset]);
            let length = (self.length - offset * CHUNK_BYTES).min(CHUNK_BYTES);
            for byte in &bytes[..length] {
                output.push(*byte)?;
            }
        }
        output.seal()
    }
}

impl Drop for SealedIngressFrame {
    fn drop(&mut self) {
        self.backing.release_sealed(&self.attempt, self.index);
    }
}
