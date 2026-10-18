//! Which bounded resource refused consumed-output verification.

use worth_relational::facade::mvcc::CompanionPreflightStop;

/// Each cause keeps its owner's typed evidence. A capacity stop is neither
/// missing evidence nor exhausted work, and it never asks for a currentness
/// retry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum VerificationCapacityStop {
    /// The request meter's own stop: its preparation bytes, their counter, or
    /// retained companion capacity. No other meter stop is held here.
    Meter(CompanionPreflightStop),
    /// A worklist length or its byte size is not representable.
    ScratchSizeOverflow,
    /// The allocator refused worklist scratch the meter had already admitted.
    ScratchAllocationRefused { bytes: u64 },
}

impl VerificationCapacityStop {
    /// The meter's stop when it is a capacity stop.
    pub(in crate::domain_computation::primary_graph) fn of_meter(
        stop: CompanionPreflightStop,
    ) -> Option<Self> {
        match stop {
            CompanionPreflightStop::PreparationMemoryExhausted { .. }
            | CompanionPreflightStop::PreparationMemoryCounterOverflow
            | CompanionPreflightStop::RetainedCompanionCapacityExhausted { .. } => {
                Some(Self::Meter(stop))
            }
            CompanionPreflightStop::Execution(_)
            | CompanionPreflightStop::TopologyPending
            | CompanionPreflightStop::SelectedSourceMismatch
            | CompanionPreflightStop::SelectedPositionUnavailable { .. }
            | CompanionPreflightStop::ForeignCell
            | CompanionPreflightStop::RegistrationChanged
            | CompanionPreflightStop::WorkExhausted { .. }
            | CompanionPreflightStop::WorkExhaustedWithoutRequirement { .. }
            | CompanionPreflightStop::WorkCounterOverflow
            | CompanionPreflightStop::Interrupted(_) => None,
        }
    }
}
