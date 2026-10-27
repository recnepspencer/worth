use super::super::attempt::AttemptAuthority;
use std::sync::{Arc, Mutex};
use worth_execution::{
    ExecutionArray, ExecutionMemoryReservation, ExecutionRequest, ExecutionRequestArrayBuilder,
    ExecutionRequestArrayDenial, WorkCeilingDenial,
};

pub(super) const CHUNK_BYTES: usize = 4096;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum FrameState {
    Free,
    Buffered,
    Sealed,
}

pub(super) struct FrameSlot {
    pub(super) state: FrameState,
    pub(super) next: Option<usize>,
    pub(super) length: usize,
}

pub(super) struct Lane {
    pub(super) attempt: Option<Arc<AttemptAuthority>>,
    pub(super) closed: bool,
    pub(super) free: Option<usize>,
    pub(super) ready: Option<usize>,
    pub(super) tail: Option<usize>,
    pub(super) sealed: Option<usize>,
}

pub(in crate::domain_computation::primary_graph::application_resource) struct IngressBacking {
    pub(super) frames: ExecutionArray<Mutex<FrameSlot>>,
    pub(super) chunks: ExecutionArray<Mutex<[u8; CHUNK_BYTES]>>,
    pub(super) chunks_per_frame: usize,
    pub(super) maximum_frame_bytes: usize,
    pub(super) lane: Mutex<Lane>,
    _funding: Option<ExecutionMemoryReservation>,
}

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph::application_resource) enum IngressAllocationDenial
{
    AccountingOverflow,
    Request(WorkCeilingDenial),
    Array(ExecutionRequestArrayDenial),
}

impl IngressBacking {
    pub(in crate::domain_computation::primary_graph::application_resource) fn allocate(
        execution: &ExecutionRequest<'_, '_>,
        slots: usize,
        maximum_frame_bytes: usize,
    ) -> Result<Arc<Self>, IngressAllocationDenial> {
        use IngressAllocationDenial as Denial;
        if slots == 0 || maximum_frame_bytes == 0 {
            return Err(Denial::AccountingOverflow);
        }
        let chunks_per_frame = maximum_frame_bytes
            .checked_add(CHUNK_BYTES - 1)
            .ok_or(Denial::AccountingOverflow)?
            / CHUNK_BYTES;
        let chunk_count = chunks_per_frame
            .checked_mul(slots)
            .ok_or(Denial::AccountingOverflow)?;
        let bytes = std::mem::size_of::<Self>()
            .checked_add(2 * std::mem::size_of::<usize>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(Denial::AccountingOverflow)?;
        let funding = (*execution)
            .reserve_retained_memory(bytes)
            .map_err(Denial::Request)?;
        let mut frames =
            ExecutionRequestArrayBuilder::allocate(*execution, slots).map_err(Denial::Array)?;
        for index in 0..slots {
            frames
                .push(Mutex::new(FrameSlot {
                    state: FrameState::Free,
                    length: 0,
                    next: (index + 1 < slots).then_some(index + 1),
                }))
                .map_err(Denial::Array)?;
        }
        let mut chunks = ExecutionRequestArrayBuilder::allocate(*execution, chunk_count)
            .map_err(Denial::Array)?;
        for _ in 0..chunk_count {
            chunks
                .push(Mutex::new([0; CHUNK_BYTES]))
                .map_err(Denial::Array)?;
        }
        Ok(Arc::new(Self {
            frames: frames.seal().map_err(Denial::Array)?,
            chunks: chunks.seal().map_err(Denial::Array)?,
            chunks_per_frame,
            maximum_frame_bytes,
            lane: Mutex::new(Lane {
                attempt: None,
                closed: true,
                free: Some(0),
                ready: None,
                tail: None,
                sealed: None,
            }),
            _funding: funding,
        }))
    }

    pub(super) fn release_sealed(&self, attempt: &Arc<AttemptAuthority>, index: usize) {
        let mut lane = lock(&self.lane);
        if !lane
            .attempt
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, attempt))
            || lane.sealed != Some(index)
        {
            return;
        }
        let mut frame = lock(&self.frames[index]);
        frame.state = FrameState::Free;
        frame.length = 0;
        frame.next = lane.free;
        lane.free = Some(index);
        lane.sealed = None;
    }
}

pub(super) fn lock<T>(value: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    value
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
