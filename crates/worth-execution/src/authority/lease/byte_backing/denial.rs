use super::super::LeaseDenial;

/// The physical or emission boundary that refused a fixed payload backing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionByteAllocationDenialKind {
    Layout,
    Lease(LeaseDenial),
    Cancelled,
    DeadlineElapsed,
    Allocator,
    CapacityMismatch,
    WriteBeyondReserved,
    IncompleteSeal,
}

/// Owner-produced refusal; a quote exists only after checked payload layout.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionByteAllocationDenial {
    kind: ExecutionByteAllocationDenialKind,
    requested_payload_bytes: Option<u64>,
}

impl ExecutionByteAllocationDenial {
    pub(super) fn new(kind: ExecutionByteAllocationDenialKind, quote: Option<u64>) -> Self {
        Self {
            kind,
            requested_payload_bytes: quote,
        }
    }
    pub fn kind(&self) -> ExecutionByteAllocationDenialKind {
        self.kind
    }
    pub fn requested_payload_bytes(&self) -> Option<u64> {
        self.requested_payload_bytes
    }
}

impl std::fmt::Display for ExecutionByteAllocationDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "byte backing refused: {:?}; checked payload quote {:?}",
            self.kind, self.requested_payload_bytes
        )
    }
}
impl std::error::Error for ExecutionByteAllocationDenial {}
