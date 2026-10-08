use super::super::ExecutionMemoryReservation;
use allocator_api2::vec::Vec;
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
    Allocated(Arc<OwnedBacking>),
}

pub(super) struct OwnedBacking {
    // Rust drops fields in declaration order: deallocate before releasing charge.
    pub(super) bytes: Vec<u8>,
    pub(super) reservation: Option<ExecutionMemoryReservation>,
}

impl ExecutionImmutableBytes {
    /// Retain already-allocated, untrusted external bytes without a payload copy.
    /// This constructor does not retrospectively admit their allocation.
    pub fn from_external_bytes(bytes: Arc<Box<[u8]>>) -> Self {
        Self {
            backing: Backing::External(bytes),
        }
    }
    pub(super) fn from_owned(backing: OwnedBacking) -> Self {
        Self {
            backing: Backing::Allocated(Arc::new(backing)),
        }
    }
    pub fn bytes(&self) -> &[u8] {
        match &self.backing {
            Backing::External(bytes) => bytes,
            Backing::Allocated(backing) => &backing.bytes,
        }
    }
    /// Actual retained payload charge. None distinguishes external/system bytes;
    /// an execution-admitted empty payload carries Some(0).
    pub fn charged_payload_bytes(&self) -> Option<u64> {
        match &self.backing {
            Backing::External(_) => None,
            Backing::Allocated(backing) => backing
                .reservation
                .as_ref()
                .map(ExecutionMemoryReservation::charged_bytes),
        }
    }
}
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
