//! Typed owner-local stop admission reasons shared by the qualified stop paths.
use worth_execution::MapKernelStop;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalOwnerStopDenial {
    Busy,
    Poisoned,
    Unsupported,
    OwnerUnavailable,
    Stopped(MapKernelStop),
}
