//! Privately prepare an append-only catalog on the actual indexing owner.
use super::*;
use worth_execution::{ExecutionRequest, WorkCeilingDenial};

mod backing;
pub(super) use crate::indexes::IndexDefinitionBacking;

#[derive(Debug)]
pub enum RelationalIndexDefinitionPreparationDenial {
    Request(WorkCeilingDenial),
    CapacityOverflow,
    IdentityExhausted,
}

/// Unpublished definitions. Numeric identities are assigned by the original
/// owner; neither preparation nor abandonment changes its installed catalog.
pub struct RelationalIndexDefinitionPreparation {
    runtime: u64,
    base: CapturedIndexDefinitionCatalog,
    catalog: IndexDefinitionCatalog,
    next_index_id: u64,
}

/// Sealed candidate data, without a live index port or an installation grant.
#[derive(Debug)]
pub struct PreparedRelationalIndexDefinitions {
    pub(crate) runtime: u64,
    base: CapturedIndexDefinitionCatalog,
    pub(crate) captured: CapturedIndexDefinitionCatalog,
}

impl RelationalIndexDefinitionPreparation {
    pub(crate) fn capture(
        runtime: u64,
        binding: IndexDefinitionReadBinding,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, RelationalIndexDefinitionPreparationDenial> {
        let base = binding.capture_catalog();
        let catalog = backing::copy_catalog(&base.catalog, request)?;
        Ok(Self {
            runtime,
            next_index_id: base.next_index_id,
            base,
            catalog,
        })
    }

    /// Reuse exact installed meaning, or privately issue the next identity.
    /// Caller-supplied numeric identities are never used.
    pub fn install(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        definition: &DerivedIndexDefinition,
    ) -> Result<DerivedIndexId, RelationalIndexDefinitionPreparationDenial> {
        use RelationalIndexDefinitionPreparationDenial as Denial;
        let (matched, _) = request
            .run_native_with_checkpoint(|context| {
                for actual in self.catalog.definitions.values() {
                    if context.has_accounting() {
                        let units = actual
                            .owned_allocation_capacity_bytes()
                            .checked_add(definition.owned_allocation_capacity_bytes())
                            .and_then(|n| n.checked_add(1))
                            .ok_or(worth_execution::MapKernelStop::WorkCounterOverflow)?;
                        context.checkpoint(units)?;
                    } else {
                        context.poll()?;
                    }
                    if actual.name == definition.name
                        && actual.kind == definition.kind
                        && actual.branch_scoped == definition.branch_scoped
                    {
                        return Ok(Some(actual.index_id));
                    }
                }
                Ok::<_, worth_execution::MapKernelStop>(None)
            })
            .map_err(Denial::Request)?;
        if let Some(id) =
            matched.map_err(|cause| Denial::Request(WorkCeilingDenial::Stopped(cause)))?
        {
            return Ok(id);
        }
        let next = self
            .next_index_id
            .checked_add(1)
            .ok_or(Denial::IdentityExhausted)?;
        backing::prepare_insert(&mut self.catalog, definition, request)?;
        let mut installed = definition.clone();
        installed.index_id = DerivedIndexId(self.next_index_id);
        let id = installed.index_id;
        self.catalog.insert(installed);
        self.next_index_id = next;
        Ok(id)
    }

    pub fn finish(
        self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<PreparedRelationalIndexDefinitions, RelationalIndexDefinitionPreparationDenial>
    {
        // Retain both the catalog Arc and its backing charge in the catalog,
        // including after the preparation and its incumbent capture disappear.
        let mut catalog = self.catalog;
        backing::retain_arc(&mut catalog, request)?;
        let coverage = catalog.coverage();
        Ok(PreparedRelationalIndexDefinitions {
            runtime: self.runtime,
            base: self.base,
            captured: CapturedIndexDefinitionCatalog {
                catalog: Arc::new(catalog),
                coverage,
                next_index_id: self.next_index_id,
            },
        })
    }
}

impl PreparedRelationalIndexDefinitions {
    pub(crate) fn admits(&self, actual: &CapturedIndexDefinitionCatalog) -> bool {
        Arc::ptr_eq(&actual.catalog, &self.base.catalog)
            && actual.next_index_id == self.base.next_index_id
    }

    pub(crate) fn introduced_ids(&self) -> impl Iterator<Item = DerivedIndexId> + '_ {
        self.captured
            .introduced(self.base.next_index_id)
            .map(|definition| definition.index_id)
    }

    pub(crate) fn introduces_definitions(&self) -> bool {
        self.base.next_index_id != self.captured.next_index_id
    }
}
