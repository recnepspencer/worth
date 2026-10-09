use super::super::fixed_backing::{
    BuildingFixedBacking, ExecutionAllocationDenial as Denial,
    ExecutionAllocationDenialKind as Kind, ExecutionAllocationPolicy,
};
use super::ExecutionImmutableBytes;

// A scheduling/check interval, not an input or allocation eligibility ceiling.
const COPY_CHUNK_BYTES: usize = 64 * 1024;

/// Fixed-layout byte author. No operation can grow or extract mutable storage.
/// Physical layout, admission and lifetime belong to the private fixed owner.
/// Allocator/Arc/ledger metadata is outside the payload charge.
///
/// ```compile_fail
/// use worth_execution::ExecutionByteBuffer;
/// fn duplicate(buffer: &ExecutionByteBuffer) -> ExecutionByteBuffer { buffer.clone() }
/// ```
pub struct ExecutionByteBuffer {
    building: BuildingFixedBacking<u8>,
}

impl ExecutionByteBuffer {
    pub fn allocate(
        total: usize,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Self, Denial> {
        Ok(Self {
            building: BuildingFixedBacking::allocate(total, policy)?,
        })
    }
    pub fn check_live(&self) -> Result<(), Denial> {
        self.building.check_live()
    }
    pub fn extend_from_slice(&mut self, payload: &[u8]) -> Result<(), Denial> {
        self.check_live()?;
        if payload.len() > self.building.element_count() - self.len() {
            return Err(self.building.denial(Kind::WriteBeyondReserved));
        }
        for chunk in payload.chunks(COPY_CHUNK_BYTES) {
            self.building.append_bytes(chunk)?;
        }
        self.check_live()
    }
    /// Overwrite only an already-written region, for example a checksum slot.
    pub fn overwrite(&mut self, offset: usize, payload: &[u8]) -> Result<(), Denial> {
        self.check_live()?;
        offset
            .checked_add(payload.len())
            .filter(|end| *end <= self.len())
            .ok_or_else(|| self.building.denial(Kind::WriteBeyondReserved))?;
        let mut start = offset;
        for chunk in payload.chunks(COPY_CHUNK_BYTES) {
            self.check_live()?;
            let end = start + chunk.len();
            self.building.written_bytes_mut()[start..end].copy_from_slice(chunk);
            start = end;
        }
        self.check_live()
    }
    pub fn bytes(&self) -> &[u8] {
        self.building.elements()
    }
    pub fn len(&self) -> usize {
        self.building.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn capacity(&self) -> usize {
        self.building.physical_capacity()
    }
    pub fn seal(self) -> Result<ExecutionImmutableBytes, Denial> {
        Ok(ExecutionImmutableBytes::from_owned(self.building.seal()?))
    }
}
impl std::fmt::Debug for ExecutionByteBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExecutionByteBuffer")
            .field("written_bytes", &self.len())
            .field("payload_bytes", &self.building.element_count())
            .finish_non_exhaustive()
    }
}
