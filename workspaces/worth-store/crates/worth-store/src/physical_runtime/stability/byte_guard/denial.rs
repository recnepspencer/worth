use super::PhysicalByteGuardScope;
use crate::physical_runtime::PhysicalRecordChunkBasis;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalByteGuardDenial {
    ReachabilityLeaseIsNotByteGuard,
    GuardScopeMismatch {
        expected: PhysicalByteGuardScope,
        observed: PhysicalByteGuardScope,
    },
    StoreChunkBasisMismatch {
        expected: PhysicalRecordChunkBasis,
        observed: PhysicalRecordChunkBasis,
    },
    UnsupportedGuardScope(PhysicalByteGuardScope),
}
