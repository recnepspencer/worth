mod admission;
mod denial;
mod grant;
mod scope;

pub use admission::PhysicalScopedAllocationAdmission;
pub use denial::PhysicalScopedAllocationFailure;
pub use grant::{
    BlobPhysicalAllocation, LayoutPhysicalAllocation, MaintenancePhysicalAllocation,
    RecoveryPhysicalAllocation, ScrubPhysicalAllocation, VerificationPhysicalAllocation,
};
