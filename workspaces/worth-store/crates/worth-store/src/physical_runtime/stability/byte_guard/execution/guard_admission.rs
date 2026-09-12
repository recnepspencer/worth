use crate::physical_runtime::stability::PhysicalByteGuardScope;
use worth_store_physical_isolation::{
    PhysicalReadProtectedFootprintBasis, StablePhysicalReadHandle,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalByteGuardAdmission {
    scope: PhysicalByteGuardScope,
    footprint_basis: PhysicalReadProtectedFootprintBasis,
}

impl PhysicalByteGuardAdmission {
    pub(crate) fn from_execution_handle(
        handle: &StablePhysicalReadHandle,
        scope: PhysicalByteGuardScope,
    ) -> Result<Self, worth_store_physical_isolation::PhysicalReadPlanAdmissionDenial> {
        handle.read_protected_reference(scope.reference())?;
        Ok(Self {
            scope,
            footprint_basis: handle.plan().footprint().declared_footprint_basis(),
        })
    }

    pub const fn scope(self) -> PhysicalByteGuardScope {
        self.scope
    }

    pub const fn footprint_basis(self) -> PhysicalReadProtectedFootprintBasis {
        self.footprint_basis
    }
}
