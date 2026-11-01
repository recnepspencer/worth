use super::super::fixed_backing::OwnedFixedBacking;
use crate::{ExecutionMemoryReservation, ExecutionRequest, ExecutionRequestArrayDenial};
use std::{ops::Deref, sync::Arc};

/// Shared immutable byte custody; imported bytes carry no execution charge.
/// Clone shares the entire backing, including any linear payload reservation.
/// It confers no permission to execute or to recover graph state.
///
/// ```compile_fail
/// use worth_execution::ExecutionImmutableBytes;
/// fn rewrite(bytes: &mut ExecutionImmutableBytes) { bytes[0] = 1; }
/// ```
#[derive(Clone)]
pub struct ExecutionImmutableBytes {
    backing: Backing,
}

#[derive(Clone)]
enum Backing {
    External(Arc<Box<[u8]>>),
    Allocated(Arc<ImmutableByteBacking>),
}

// Neither this Arc nor a Weak is exposed. Payload storage drops before its
// independent fixed control admission; clones share both actual allocations.
struct ImmutableByteBacking {
    bytes: OwnedFixedBacking<u8>,
    _control: Option<ExecutionMemoryReservation>,
}

impl ExecutionImmutableBytes {
    /// Retain already-allocated, untrusted external bytes without a payload copy.
    /// This constructor does not retrospectively admit their allocation.
    pub fn from_external_bytes(bytes: Arc<Box<[u8]>>) -> Self {
        Self {
            backing: Backing::External(bytes),
        }
    }
    pub(in super::super) fn from_owned(backing: OwnedFixedBacking<u8>) -> Self {
        Self {
            backing: Backing::Allocated(Arc::new(ImmutableByteBacking {
                bytes: backing,
                _control: None,
            })),
        }
    }
    pub(in super::super) fn from_owned_for_request(
        backing: OwnedFixedBacking<u8>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, ExecutionRequestArrayDenial> {
        use ExecutionRequestArrayDenial as Denial;
        request.consult().map_err(Denial::Request)?;
        let bytes = control_bytes().map_err(Denial::Allocation)?;
        let control = request
            .reserve_retained_memory(bytes)
            .map_err(Denial::Request)?;
        Ok(Self {
            backing: Backing::Allocated(Arc::new(ImmutableByteBacking {
                bytes: backing,
                _control: control,
            })),
        })
    }
    pub fn bytes(&self) -> &[u8] {
        match &self.backing {
            Backing::External(bytes) => bytes,
            Backing::Allocated(backing) => backing.bytes.elements(),
        }
    }
    /// Actual retained payload charge. None distinguishes external/system bytes;
    /// an execution-admitted empty payload carries Some(0).
    pub fn charged_payload_bytes(&self) -> Option<u64> {
        match &self.backing {
            Backing::External(_) => None,
            Backing::Allocated(backing) => backing.bytes.charged_payload_bytes(),
        }
    }
}

fn control_bytes() -> Result<u64, super::super::fixed_backing::ExecutionAllocationDenial> {
    use super::super::fixed_backing::{
        ExecutionAllocationDenial as Denial, ExecutionAllocationDenialKind as Kind,
    };
    let layout = std::alloc::Layout::new::<(usize, usize)>()
        .extend(std::alloc::Layout::new::<ImmutableByteBacking>())
        .map_err(|_| Denial::new(Kind::Layout, None))?
        .0
        .pad_to_align();
    u64::try_from(layout.size()).map_err(|_| Denial::new(Kind::Layout, None))
}

#[cfg(test)]
mod request_tests;
impl Deref for ExecutionImmutableBytes {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        self.bytes()
    }
}
impl AsRef<[u8]> for ExecutionImmutableBytes {
    fn as_ref(&self) -> &[u8] {
        self.bytes()
    }
}
impl PartialEq for ExecutionImmutableBytes {
    fn eq(&self, other: &Self) -> bool {
        self.bytes() == other.bytes()
    }
}
impl Eq for ExecutionImmutableBytes {}
impl std::fmt::Debug for ExecutionImmutableBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExecutionImmutableBytes")
            .field("byte_len", &self.bytes().len())
            .finish_non_exhaustive()
    }
}
