//! The original in-memory journals freeze actual bytes, never regenerated artifacts.
use super::bytes::MemberBytes;
use sha2::{Digest, Sha256};
use worth_execution::{
    ExecutionImmutableBytes, ExecutionMemoryReservation, ExecutionOrderedMap, ExecutionRequest,
    ExecutionRequestArrayBuilder,
};
/// Storage-only denial shared by two Native compilation identities.
#[derive(Debug)]
pub enum OwnedSnapshotDenial {
    Request(worth_execution::WorkCeilingDenial),
    Buffer(worth_execution::ExecutionRequestArrayDenial),
    Missing,
    Integrity,
    Limit { length: u64, limit: u64 },
    Capacity,
}
use OwnedSnapshotDenial as Refusal;
impl From<worth_execution::WorkCeilingDenial> for OwnedSnapshotDenial {
    fn from(cause: worth_execution::WorkCeilingDenial) -> Self {
        Self::Request(cause)
    }
}
pub struct OwnedNativeReadSnapshot {
    bounds: [u64; 2],
    canonical: ExecutionOrderedMap<u64, ExecutionImmutableBytes>,
    transitions: ExecutionOrderedMap<u64, ExecutionImmutableBytes>,
    members: ExecutionOrderedMap<([u8; 32], u64), ExecutionImmutableBytes>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl OwnedNativeReadSnapshot {
    /// Copy/fund every selected original buffer while the caller holds its own
    /// journal locks. Snapshot cost is actual prefix bytes plus ordered cells.
    pub fn capture<'a>(
        request: &ExecutionRequest<'_, '_>,
        canonical: impl Iterator<Item = (u64, &'a [u8])>,
        transitions: impl Iterator<Item = (u64, &'a [u8])>,
        members: &MemberBytes,
    ) -> Result<Box<Self>, Refusal> {
        let memory = request.reserve_retained_memory(std::mem::size_of::<Self>() as u64)?;
        let mut snapshot = Self {
            bounds: [0, 0],
            canonical: Default::default(),
            transitions: Default::default(),
            members: Default::default(),
            _memory: memory,
        };
        for (position, bytes) in canonical {
            let owned = copy(request, bytes)?;
            snapshot
                .canonical
                .insert_cloned_native_for_request(&position, &owned, *request)
                .map_err(map_failure)?;
            snapshot.bounds[0] = snapshot.bounds[0].max(position);
        }
        for (sequence, bytes) in transitions {
            let owned = copy(request, bytes)?;
            snapshot
                .transitions
                .insert_cloned_native_for_request(&sequence, &owned, *request)
                .map_err(map_failure)?;
            snapshot.bounds[1] = snapshot.bounds[1].max(sequence);
        }
        for (key, bytes) in members.iter() {
            let owned = copy(request, bytes)?;
            snapshot
                .members
                .insert_cloned_native_for_request(key, &owned, *request)
                .map_err(map_failure)?;
        }
        Ok(Box::new(snapshot))
    }
    pub fn bounds(&self) -> [u64; 2] {
        self.bounds
    }
    pub fn canonical(
        &self,
        request: &ExecutionRequest<'_, '_>,
        position: u64,
        limit: u64,
    ) -> Result<ExecutionImmutableBytes, Refusal> {
        self.read(request, self.canonical.get(&position), limit)
    }
    pub fn transition(
        &self,
        request: &ExecutionRequest<'_, '_>,
        sequence: u64,
        limit: u64,
    ) -> Result<ExecutionImmutableBytes, Refusal> {
        self.read(request, self.transitions.get(&sequence), limit)
    }
    pub fn member(
        &self,
        request: &ExecutionRequest<'_, '_>,
        digest: [u8; 32],
        length: u64,
        limit: u64,
    ) -> Result<ExecutionImmutableBytes, Refusal> {
        let bytes = self.read(request, self.members.get(&(digest, length)), limit)?;
        if bytes.len() as u64 != length || <[u8; 32]>::from(Sha256::digest(bytes.bytes())) != digest
        {
            return Err(Refusal::Integrity);
        }
        Ok(bytes)
    }
    fn read(
        &self,
        request: &ExecutionRequest<'_, '_>,
        bytes: Option<&ExecutionImmutableBytes>,
        limit: u64,
    ) -> Result<ExecutionImmutableBytes, Refusal> {
        request.check_live().map_err(|cause| {
            Refusal::Request(worth_execution::WorkCeilingDenial::Stopped(cause))
        })?;
        let bytes = bytes.ok_or(Refusal::Missing)?;
        if bytes.len() as u64 > limit {
            return Err(Refusal::Limit {
                length: bytes.len() as u64,
                limit,
            });
        }
        Ok(bytes.clone())
    }
}
fn copy(
    request: &ExecutionRequest<'_, '_>,
    bytes: &[u8],
) -> Result<ExecutionImmutableBytes, Refusal> {
    let mut buffer = ExecutionRequestArrayBuilder::<u8>::allocate(*request, bytes.len())
        .map_err(Refusal::Buffer)?;
    buffer.extend_from_slice(bytes).map_err(Refusal::Buffer)?;
    Ok(buffer
        .seal()
        .map_err(Refusal::Buffer)?
        .into_immutable_bytes())
}
fn map_failure(cause: worth_execution::ExecutionOrderedMapDenial) -> Refusal {
    use worth_execution::ExecutionOrderedMapDenial as Denial;
    match cause {
        Denial::Request(cause) => Refusal::from(cause),
        Denial::Allocation(cause) => Refusal::Buffer(
            worth_execution::ExecutionRequestArrayDenial::Allocation(cause),
        ),
        Denial::CapacityOverflow => Refusal::Capacity,
    }
}
