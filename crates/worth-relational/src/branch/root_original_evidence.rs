//! One Original root owner carries its envelope, encoded payload and original funding.
use super::super::{reserve_arc_memory, RelationalBranchRootCaptureDenial as Denial};
use crate::history::data::CanonicalCommitEnvelope;
use sha2::{Digest, Sha256};
use std::{
    io::{self, Write},
    sync::Arc,
};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, MapKernelContext};

#[derive(Debug)]
pub(crate) struct RelationalOriginalRootCommitEvidence {
    envelope: Arc<CanonicalCommitEnvelope>,
    payload: Vec<u8>,
    payload_digest: [u8; 32],
    // Bytes drop before the ticket; every clone shares this exact complete owner.
    _payload_memory: Option<ExecutionMemoryReservation>,
    _memory: Option<ExecutionMemoryReservation>,
}

impl RelationalOriginalRootCommitEvidence {
    pub(crate) fn envelope(&self) -> &Arc<CanonicalCommitEnvelope> {
        &self.envelope
    }
    pub(crate) fn payload(&self) -> &[u8] {
        &self.payload
    }
    pub(crate) fn payload_digest(&self) -> [u8; 32] {
        self.payload_digest
    }
    pub(crate) fn payload_storage_bytes(&self) -> u64 {
        self.payload.capacity() as u64
    }

    pub(crate) fn prepare(
        request: &ExecutionRequest<'_, '_>,
        envelope: Arc<CanonicalCommitEnvelope>,
    ) -> Result<Arc<Self>, Denial> {
        let memory = reserve_arc_memory::<Self>(Some(request))?;
        let payload_memory = request
            .reserve_retained_memory(0)
            .map_err(|cause| Denial::Request(cause.into()))?;
        let mut answer = None;
        let scope = request.run_native_with_checkpoint(|context| {
            let mut writer = OriginalPayloadWriter {
                bytes: Vec::new(),
                hash: Sha256::new(),
                memory: payload_memory,
                context,
                refusal: None,
            };
            let encoded = envelope.write_authoritative_payload(&mut writer);
            answer = Some(match (encoded, writer.refusal.take()) {
                (_, Some(cause)) => Err(cause),
                (Err(_), None) => Err(Denial::CanonicalEnvelopeMismatch),
                (Ok(()), None) => Ok(Arc::new(Self {
                    envelope,
                    payload: writer.bytes,
                    payload_digest: writer.hash.finalize().into(),
                    _payload_memory: writer.memory,
                    _memory: memory,
                })),
            });
        });
        match (scope, answer) {
            (Ok(_), Some(answer)) => answer,
            (
                Err(worth_execution::WorkCeilingDenial::Stopped(
                    worth_execution::MapKernelStop::NestedStopped,
                )),
                Some(Err(cause)),
            ) => Err(cause),
            (Err(cause), _) => Err(Denial::Request(cause.into())),
            (Ok(_), None) => unreachable!("completed Original payload emission"),
        }
    }
}

struct OriginalPayloadWriter<'context, 'scope, 'authority> {
    bytes: Vec<u8>,
    hash: Sha256,
    memory: Option<ExecutionMemoryReservation>,
    context: &'context mut MapKernelContext<'scope, 'authority>,
    refusal: Option<Denial>,
}
impl OriginalPayloadWriter<'_, '_, '_> {
    fn append(&mut self, input: &[u8]) -> Result<(), Denial> {
        let result = if self.context.has_accounting() {
            self.context.checkpoint(input.len() as u64)
        } else {
            self.context.poll()
        };
        result.map_err(|cause| Denial::Request(cause.into()))?;
        let required = self
            .bytes
            .len()
            .checked_add(input.len())
            .ok_or(Denial::AllocationLayoutOverflow)?;
        if required > self.bytes.capacity() {
            let capacity = required
                .checked_next_power_of_two()
                .filter(|capacity| *capacity <= isize::MAX as usize)
                .ok_or(Denial::AllocationLayoutOverflow)?;
            if let Some(memory) = &mut self.memory {
                // Reallocation may hold old and new backing simultaneously.
                let peak = self
                    .bytes
                    .capacity()
                    .checked_add(capacity)
                    .ok_or(Denial::AllocationLayoutOverflow)?;
                memory
                    .grow_for_allocation(peak as u64)
                    .map_err(backing_failure)?;
            }
            self.bytes
                .try_reserve_exact(capacity - self.bytes.len())
                .map_err(|_| allocation_failed())?;
            if self.bytes.capacity() != capacity {
                return Err(allocation_failed());
            }
            if let Some(memory) = &mut self.memory {
                memory
                    .resize(capacity as u64)
                    .expect("releasing transient growth backing cannot exhaust memory");
            }
        }
        for chunk in input.chunks(64 * 1024) {
            self.context
                .poll()
                .map_err(|cause| Denial::Request(cause.into()))?;
            self.bytes.extend_from_slice(chunk);
            crate::execution::phase_observation::hash_bytes(
                crate::execution::phase_observation::HashPurpose::CommitPayload,
                chunk.len(),
            );
            self.hash.update(chunk);
        }
        Ok(())
    }
}
impl Write for OriginalPayloadWriter<'_, '_, '_> {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        match self.append(input) {
            Ok(()) => Ok(input.len()),
            Err(cause) => {
                self.refusal = Some(cause);
                Err(io::Error::other("Original root payload request refused"))
            }
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn allocation_failed() -> Denial {
    Denial::Request(crate::execution::RelationalExecutionDenialCause::AllocationFailed)
}
fn backing_failure(cause: worth_execution::ExecutionAllocationDenial) -> Denial {
    match cause.classify() {
        worth_execution::ExecutionAllocationDenialClass::Request(stop) => {
            Denial::Request(stop.into())
        }
        worth_execution::ExecutionAllocationDenialClass::Physical(_) => {
            Denial::CanonicalPayloadBacking(cause)
        }
    }
}
