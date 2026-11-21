//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphLayout {
    #[doc(hidden)]
    pub fn resolve_field_coordinate_for_test<'a>(
        &'a self,
        contracts: &'a WorthQueryInstalledApplicationSchemaContractCatalog,
        entity: &str,
        descriptor: &AspectFieldLocator,
        admit: impl FnMut(u64) -> Result<(), MapKernelStop>,
    ) -> Result<Option<InstalledFieldCoordinate<'a>>, MapKernelStop> {
        Self::resolve_field_coordinate(self, contracts, entity, descriptor, admit)
    }
}

impl InstalledFieldCoordinate<'_> {
    #[doc(hidden)]
    pub fn same_field_for_test(&self, other: &Self) -> bool {
        Self::same_field(self, other)
    }
}

impl InstalledFieldCoordinate<'_> {
    #[doc(hidden)]
    pub fn entity_kind_for_test(&self) -> KindId {
        Self::entity_kind(self)
    }
}

impl WorthQueryPrimaryGraphLayout {
    #[doc(hidden)]
    pub fn resolve_declared_field_coordinate_for_test<'a>(
        &'a self,
        contracts: &'a WorthQueryInstalledApplicationSchemaContractCatalog,
        entity: &str,
        aspect: &AspectKey,
        field: &FieldKey,
        admit: impl FnMut(u64) -> Result<(), MapKernelStop>,
    ) -> Result<Option<InstalledFieldCoordinate<'a>>, MapKernelStop> {
        Self::resolve_declared_field_coordinate(self, contracts, entity, aspect, field, admit)
    }
}

impl InstalledFieldCoordinate<'_> {
    #[doc(hidden)]
    pub fn field_for_test(&self) -> &FieldKey {
        Self::field(self)
    }
}

impl InstalledFieldCoordinate<'_> {
    #[doc(hidden)]
    pub fn native_for_test(&self) -> &WorthQueryInstalledApplicationAspectContract {
        Self::native(self)
    }
}
