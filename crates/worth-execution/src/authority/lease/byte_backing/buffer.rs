use super::super::ExecutionLeaseStatus;
use super::immutable::OwnedBacking;
use super::policy::check_status;
use super::{
    ExecutionByteAllocationDenial as Denial, ExecutionByteAllocationDenialKind as Kind,
    ExecutionByteAllocationPolicy, ExecutionImmutableBytes,
};
use allocator_api2::vec::Vec;
use std::alloc::Layout;

// A scheduling/check interval, not an input or allocation eligibility ceiling.
const COPY_CHUNK_BYTES: usize = 64 * 1024;

/// Fixed-layout payload builder. No operation can grow or extract mutable storage.
/// Allocation uses the pinned allocator-api2 fresh Global Vec implementation:
/// grow_exact requests Layout::array<u8>(total) and sets exactly that capacity.
/// Allocator/Arc/ledger metadata is outside the payload charge.
///
/// ```compile_fail
/// use worth_execution::ExecutionByteBuffer;
/// fn duplicate(buffer: &ExecutionByteBuffer) -> ExecutionByteBuffer { buffer.clone() }
/// ```
pub struct ExecutionByteBuffer {
    backing: OwnedBacking,
    total: usize,
    quote: u64,
    status: Option<ExecutionLeaseStatus>,
}

impl ExecutionByteBuffer {
    pub fn allocate(
        total: usize,
        policy: ExecutionByteAllocationPolicy<'_, '_>,
    ) -> Result<Self, Denial> {
        let layout = Layout::array::<u8>(total).map_err(|_| Denial::new(Kind::Layout, None))?;
        let quote = u64::try_from(layout.size()).map_err(|_| Denial::new(Kind::Layout, None))?;
        let status = match policy {
            ExecutionByteAllocationPolicy::SystemAllocation => None,
            ExecutionByteAllocationPolicy::Execution(lease) => Some(lease.status()),
        };
        if let Some(status) = &status {
            check_status(status, Some(quote))?;
        }
        let reservation = match policy {
            ExecutionByteAllocationPolicy::SystemAllocation => None,
            ExecutionByteAllocationPolicy::Execution(lease) => Some(
                lease
                    .reserve_memory(quote)
                    .map_err(|e| Denial::new(Kind::Lease(e), Some(quote)))?,
            ),
        };
        if let Some(status) = &status {
            check_status(status, Some(quote))?;
        }
        // The reservation exists before the first payload allocation. If growth
        // fails, bytes drops before the earlier local reservation.
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(total)
            .map_err(|_| Denial::new(Kind::Allocator, Some(quote)))?;
        if bytes.capacity() != total {
            return Err(Denial::new(Kind::CapacityMismatch, Some(quote)));
        }
        let buffer = Self {
            backing: OwnedBacking { bytes, reservation },
            total,
            quote,
            status,
        };
        buffer.check_live()?;
        Ok(buffer)
    }
    pub fn check_live(&self) -> Result<(), Denial> {
        self.status
            .as_ref()
            .map_or(Ok(()), |s| check_status(s, Some(self.quote)))
    }
    pub fn extend_from_slice(&mut self, payload: &[u8]) -> Result<(), Denial> {
        self.check_live()?;
        if payload.len() > self.total - self.len() {
            return Err(self.denial(Kind::WriteBeyondReserved));
        }
        for chunk in payload.chunks(COPY_CHUNK_BYTES) {
            self.check_live()?;
            self.backing.bytes.extend_from_slice(chunk);
        }
        self.check_live()
    }
    /// Overwrite only an already-written region, for example a checksum slot.
    pub fn overwrite(&mut self, offset: usize, payload: &[u8]) -> Result<(), Denial> {
        self.check_live()?;
        let end = offset
            .checked_add(payload.len())
            .filter(|end| *end <= self.len())
            .ok_or_else(|| self.denial(Kind::WriteBeyondReserved))?;
        for (target, chunk) in self.backing.bytes[offset..end]
            .chunks_mut(COPY_CHUNK_BYTES)
            .zip(payload.chunks(COPY_CHUNK_BYTES))
        {
            if let Some(status) = &self.status {
                check_status(status, Some(self.quote))?;
            }
            target.copy_from_slice(chunk);
        }
        self.check_live()
    }
    pub fn bytes(&self) -> &[u8] {
        &self.backing.bytes
    }
    pub fn len(&self) -> usize {
        self.backing.bytes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.backing.bytes.is_empty()
    }
    pub fn capacity(&self) -> usize {
        self.backing.bytes.capacity()
    }
    pub fn seal(self) -> Result<ExecutionImmutableBytes, Denial> {
        self.check_live()?;
        if self.len() != self.total {
            return Err(self.denial(Kind::IncompleteSeal));
        }
        Ok(ExecutionImmutableBytes::from_owned(self.backing))
    }
    fn denial(&self, kind: Kind) -> Denial {
        Denial::new(kind, Some(self.quote))
    }
}
impl std::fmt::Debug for ExecutionByteBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExecutionByteBuffer")
            .field("written_bytes", &self.len())
            .field("payload_bytes", &self.total)
            .finish_non_exhaustive()
    }
}
