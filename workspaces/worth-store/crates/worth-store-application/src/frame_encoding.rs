//! Admission and original-request billing of an owned publication frame.
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, WorkCeilingDenial};

pub(crate) struct EncodedHomeFrame {
    pub(crate) bytes: Vec<u8>,
    pub(crate) memory: Option<ExecutionMemoryReservation>,
}

pub(crate) fn encode(
    size: usize,
    execution: &ExecutionRequest<'_, '_>,
    emit: impl FnOnce(&mut Vec<u8>),
) -> Result<EncodedHomeFrame, WorkCeilingDenial> {
    let memory = execution.reserve_retained_memory(size as u64)?;
    let (bytes, _) = execution.run_native_with_checkpoint(|context| {
        if context.has_accounting() {
            context.checkpoint(size as u64)?;
        } else {
            context.poll()?;
        }
        let mut bytes = Vec::with_capacity(size);
        emit(&mut bytes);
        Ok::<_, worth_execution::MapKernelStop>(bytes)
    })?;
    let bytes = bytes.map_err(WorkCeilingDenial::Stopped)?;
    Ok(EncodedHomeFrame { bytes, memory })
}
