use super::super::LeaseDenial;
use crate::{MapKernelStop, WorkCeilingDenial};

/// The request or physical boundary that refused a fixed payload backing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionAllocationDenialKind {
    Layout,
    Lease(LeaseDenial),
    Cancelled,
    DeadlineElapsed,
    Allocator,
    CapacityMismatch,
    WriteBeyondReserved,
    IncompleteSeal,
}

/// A machine or layout refusal, independent of request admission and control.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionPhysicalAllocationDenialKind {
    /// The requested payload has no representable layout.
    Layout,
    /// The allocator refused the checked payload layout.
    Allocator,
    /// The backing capacity differs from the reserved capacity.
    CapacityMismatch,
    /// A write exceeded the reserved payload.
    WriteBeyondReserved,
    /// Sealing encountered an incomplete payload.
    IncompleteSeal,
}

/// The owner's separation of request causes from physical allocation refusals.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionAllocationDenialClass {
    /// The request's lease or control refused allocation.
    Request(WorkCeilingDenial),
    /// The machine or layout refused allocation.
    Physical(ExecutionPhysicalAllocationDenialKind),
}

impl ExecutionAllocationDenialKind {
    /// Classify once at the owner; new raw kinds must declare their cause.
    pub fn classify(self) -> ExecutionAllocationDenialClass {
        use ExecutionAllocationDenialClass::{Physical, Request};
        use ExecutionPhysicalAllocationDenialKind as PhysicalKind;
        match self {
            Self::Layout => Physical(PhysicalKind::Layout),
            Self::Lease(denial) => Request(WorkCeilingDenial::Admission(denial)),
            Self::Cancelled => Request(WorkCeilingDenial::Stopped(MapKernelStop::Cancelled)),
            Self::DeadlineElapsed => {
                Request(WorkCeilingDenial::Stopped(MapKernelStop::DeadlineElapsed))
            }
            Self::Allocator => Physical(PhysicalKind::Allocator),
            Self::CapacityMismatch => Physical(PhysicalKind::CapacityMismatch),
            Self::WriteBeyondReserved => Physical(PhysicalKind::WriteBeyondReserved),
            Self::IncompleteSeal => Physical(PhysicalKind::IncompleteSeal),
        }
    }
}

/// Owner-produced refusal; a quote exists only after checked payload layout.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionAllocationDenial {
    kind: ExecutionAllocationDenialKind,
    requested_payload_bytes: Option<u64>,
}

impl ExecutionAllocationDenial {
    pub(in super::super) fn new(kind: ExecutionAllocationDenialKind, quote: Option<u64>) -> Self {
        Self {
            kind,
            requested_payload_bytes: quote,
        }
    }
    pub fn kind(&self) -> ExecutionAllocationDenialKind {
        self.kind
    }
    /// Separate the request cause from a physical refusal through the owner.
    pub fn classify(&self) -> ExecutionAllocationDenialClass {
        self.kind.classify()
    }
    pub fn requested_payload_bytes(&self) -> Option<u64> {
        self.requested_payload_bytes
    }
}

impl crate::ChargedBytes for ExecutionAllocationDenial {
    fn additional_charged_bytes(&self) -> u64 {
        let Self {
            kind,
            requested_payload_bytes,
        } = self;
        fn inline<T: Copy>(_: &T) {}
        inline(kind);
        inline(requested_payload_bytes);
        0
    }
}

impl std::fmt::Display for ExecutionAllocationDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "payload backing refused: {:?}; checked payload quote {:?}",
            self.kind, self.requested_payload_bytes
        )
    }
}
impl std::error::Error for ExecutionAllocationDenial {}

#[cfg(test)]
mod tests;
