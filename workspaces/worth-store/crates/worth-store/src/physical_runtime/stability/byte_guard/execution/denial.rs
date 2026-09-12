use super::StablePhysicalReadExecutionCounters;
use crate::physical_runtime::stability::{
    PhysicalByteGuardDenial, PhysicalByteGuardScope, StableReadSecurityScopeCarrierBasis,
};
use worth_store_physical_isolation::{
    CurrentPhysicalRoot, PhysicalReadPlanAdmissionDenial, PhysicalReadProtectedFootprintBasis,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalReadExecutionDenial {
    ReadPlanDenied(PhysicalReadPlanAdmissionDenial),
    ByteGuardDenied(PhysicalByteGuardDenial),
    GuardScopeNotInPlan {
        scope: PhysicalByteGuardScope,
        counters: StablePhysicalReadExecutionCounters,
    },
    ByteGuardScopeMismatch {
        admitted: PhysicalByteGuardScope,
        observed: PhysicalByteGuardScope,
    },
    LogicalDecodeScopeRootMismatch {
        admitted: CurrentPhysicalRoot,
        observed: CurrentPhysicalRoot,
    },
    LogicalDecodeScopeFootprintMismatch {
        admitted: PhysicalReadProtectedFootprintBasis,
        observed: PhysicalReadProtectedFootprintBasis,
    },
    LogicalDecodeScopeCarrierMismatch {
        admitted: StableReadSecurityScopeCarrierBasis,
        observed: PhysicalByteGuardScope,
    },
    HiddenStructuralLatchIoWithoutDeclaredCost {
        counters: StablePhysicalReadExecutionCounters,
    },
}

impl From<PhysicalReadPlanAdmissionDenial> for PhysicalReadExecutionDenial {
    fn from(denial: PhysicalReadPlanAdmissionDenial) -> Self {
        Self::ReadPlanDenied(denial)
    }
}

impl From<PhysicalByteGuardDenial> for PhysicalReadExecutionDenial {
    fn from(denial: PhysicalByteGuardDenial) -> Self {
        Self::ByteGuardDenied(denial)
    }
}
