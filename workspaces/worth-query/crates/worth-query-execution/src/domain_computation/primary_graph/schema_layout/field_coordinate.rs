//! Resolve descriptive fields against the actual installed layout and Native binding.
use super::{WorthQueryPrimaryFieldLayout, WorthQueryPrimaryGraphLayout};
use worth_execution::MapKernelStop;
use worth_foundational::facade::{
    AspectBinding, AspectContractRevision, AspectFieldLocator, AspectKey, AspectShape, FieldKey,
    LocatorAuthority,
};
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationAspectContract,
    WorthQueryInstalledApplicationSchemaContractCatalog,
};
use worth_relational::facade::identity::KindId;

/// Borrowed installed meaning, never a read permission or a rewritten descriptor.
#[derive(Clone, Copy, Debug)]
pub(in crate::domain_computation::primary_graph) struct InstalledFieldCoordinate<'a> {
    field: &'a WorthQueryPrimaryFieldLayout,
    native: &'a WorthQueryInstalledApplicationAspectContract,
}
impl InstalledFieldCoordinate<'_> {
    pub(in crate::domain_computation::primary_graph) fn same_field(&self, other: &Self) -> bool {
        std::ptr::eq(self.field, other.field) && std::ptr::eq(self.native, other.native)
    }
    pub(in crate::domain_computation::primary_graph) fn entity_kind(&self) -> KindId {
        self.field.entity_kind
    }
    pub(in crate::domain_computation::primary_graph) fn contract_revision(
        &self,
    ) -> AspectContractRevision {
        self.native.contract().revision()
    }
}

impl WorthQueryPrimaryGraphLayout {
    /// O(installed fields + selected contract fields), diagnostic only; no allocation.
    pub(in crate::domain_computation::primary_graph) fn resolve_field_coordinate<'a>(
        &'a self,
        contracts: &'a WorthQueryInstalledApplicationSchemaContractCatalog,
        entity: &str,
        descriptor: &AspectFieldLocator,
        mut admit: impl FnMut(u64) -> Result<(), MapKernelStop>,
    ) -> Result<Option<InstalledFieldCoordinate<'a>>, MapKernelStop> {
        admit(1)?;
        if !matches!(
            descriptor.aspect().authority(),
            LocatorAuthority::Planned | LocatorAuthority::Authoritative
        ) {
            return Ok(None);
        }
        let [field] = descriptor.field_path().fields() else {
            return Ok(None);
        };
        let coordinate = self.resolve_declared_field_coordinate(
            contracts,
            entity,
            descriptor.aspect().aspect_key(),
            field,
            &mut admit,
        )?;
        Ok(coordinate.filter(|coordinate| {
            descriptor.aspect().authority() != LocatorAuthority::Planned
                || descriptor == &coordinate.field.locator
        }))
    }

    /// Resolve an already issued query footprint through the same installed inventory.
    pub(in crate::domain_computation::primary_graph) fn resolve_declared_field_coordinate<'a>(
        &'a self,
        contracts: &'a WorthQueryInstalledApplicationSchemaContractCatalog,
        entity: &str,
        aspect: &AspectKey,
        field: &FieldKey,
        mut admit: impl FnMut(u64) -> Result<(), MapKernelStop>,
    ) -> Result<Option<InstalledFieldCoordinate<'a>>, MapKernelStop> {
        let text = entity
            .len()
            .checked_add(aspect.as_str().len())
            .and_then(|bytes| bytes.checked_add(field.as_str().len()))
            .and_then(|bytes| bytes.checked_add(1))
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(MapKernelStop::WorkCounterOverflow)?;
        // Existing tuple-key inventory is borrowed directly, without allocating lookup keys.
        for ((owner, aspect_name, field_name), installed) in &self.fields {
            admit(text)?;
            if owner != entity || aspect_name != aspect.as_str() || field_name != field.as_str() {
                continue;
            }
            admit(text)?;
            let Some(native) = contracts.aspect(entity, aspect.as_str()) else {
                return Ok(None);
            };
            let Some(lowered) = self.aspect_contract(entity, aspect) else {
                return Ok(None);
            };
            let AspectShape::Struct(shape) = native.contract().shape() else {
                return Ok(None);
            };
            if !matches!(native.binding(), AspectBinding::EntityField { .. })
                || native.contract().identity() != lowered.identity()
                || native.contract().revision() != lowered.revision()
                || native.contract().key() != aspect
                || !native.contains_field(field)
                || shape.field(field).is_none()
                || self.entity_kind(entity) != Some(installed.entity_kind)
            {
                return Ok(None);
            }
            return Ok(Some(InstalledFieldCoordinate {
                field: installed,
                native,
            }));
        }
        Ok(None)
    }
}
