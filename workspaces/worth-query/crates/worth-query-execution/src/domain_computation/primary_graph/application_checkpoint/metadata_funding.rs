//! Pre-allocation ownership of reconstructed checkpoint metadata. External
//! immutable artifact backing and reconstructed heap payload have distinct payers.
use super::{CheckpointCursor, CheckpointDecodeDenial};
use worth_execution::{
    ExecutionMemoryReservation, ExecutionRequest, LeaseDenial, WorkCeilingDenial,
};

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) struct CheckpointMetadataFunding {
    held: ExecutionMemoryReservation,
}
impl CheckpointMetadataFunding {
    pub(in crate::domain_computation::primary_graph) fn new(
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, CheckpointDecodeDenial> {
        execution
            .consult()
            .map_err(CheckpointDecodeDenial::execution)?;
        let held = execution
            .in_scope(|lease| ExecutionMemoryReservation::reserve_in_scope(lease, 0))
            .map_err(CheckpointDecodeDenial::execution)?
            .map_err(|cause| {
                CheckpointDecodeDenial::execution(WorkCeilingDenial::Admission(cause))
            })?;
        Ok(Self { held })
    }
    pub(in crate::domain_computation::primary_graph) fn check_live(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), CheckpointDecodeDenial> {
        execution
            .consult()
            .map_err(CheckpointDecodeDenial::execution)
    }
    pub(in crate::domain_computation::primary_graph) fn slots<T>(
        &mut self,
        count: usize,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), CheckpointDecodeDenial> {
        let bytes = count
            .checked_mul(std::mem::size_of::<T>())
            .ok_or_else(CheckpointDecodeDenial::capacity)?;
        self.bytes(bytes, execution)
    }
    pub(in crate::domain_computation::primary_graph) fn bytes(
        &mut self,
        bytes: usize,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), CheckpointDecodeDenial> {
        self.check_live(execution)?;
        let total = self
            .held
            .bytes()
            .checked_add(u64::try_from(bytes).map_err(|_| CheckpointDecodeDenial::capacity())?)
            .ok_or_else(CheckpointDecodeDenial::capacity)?;
        self.held.resize(total).map_err(|cause| {
            CheckpointDecodeDenial::execution(WorkCeilingDenial::Admission(
                LeaseDenial::MemoryExhausted(cause),
            ))
        })
    }
    pub(in crate::domain_computation::primary_graph) fn release_slots<T>(
        &mut self,
        count: usize,
    ) -> Result<(), CheckpointDecodeDenial> {
        let bytes = count
            .checked_mul(std::mem::size_of::<T>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(CheckpointDecodeDenial::capacity)?;
        let total = self
            .held
            .bytes()
            .checked_sub(bytes)
            .ok_or_else(CheckpointDecodeDenial::capacity)?;
        self.held.resize(total).map_err(|cause| {
            CheckpointDecodeDenial::execution(WorkCeilingDenial::Admission(
                LeaseDenial::MemoryExhausted(cause),
            ))
        })
    }
    pub(in crate::domain_computation::primary_graph) fn vec<T>(
        &mut self,
        count: usize,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Vec<T>, CheckpointDecodeDenial> {
        self.slots::<T>(count, execution)?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(count)
            .map_err(|_| CheckpointDecodeDenial::allocator())?;
        if std::mem::size_of::<T>() != 0 && values.capacity() != count {
            return Err(CheckpointDecodeDenial::allocator());
        }
        Ok(values)
    }
    pub(in crate::domain_computation::primary_graph) fn copy_text(
        &mut self,
        value: &str,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<String, CheckpointDecodeDenial> {
        self.bytes(value.len(), execution)?;
        let mut text = String::new();
        text.try_reserve_exact(value.len())
            .map_err(|_| CheckpointDecodeDenial::allocator())?;
        if text.capacity() != value.len() {
            return Err(CheckpointDecodeDenial::allocator());
        }
        text.push_str(value);
        Ok(text)
    }
    pub(super) fn text(
        &mut self,
        cursor: &mut CheckpointCursor<'_>,
        execution: &ExecutionRequest<'_, '_>,
        maximum: usize,
        subject: &str,
    ) -> Result<String, CheckpointDecodeDenial> {
        let length = usize::try_from(cursor.next_u64()?)
            .map_err(|_| format!("{subject} length exceeds this host"))?;
        if length == 0 || length > maximum {
            return Err(format!("{subject} length is invalid").into());
        }
        let value = std::str::from_utf8(cursor.next_bytes(length)?)
            .map_err(|_| format!("{subject} is not UTF-8"))?;
        self.copy_text(value, execution)
    }
}
