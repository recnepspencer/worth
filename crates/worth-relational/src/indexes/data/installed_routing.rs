//! Coverage compiled at installation; commits consult only changed record kinds.
use super::{DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind};
use crate::identity::data::KindId;
use crate::schema::data::AspectContractPlanCatalog;
use std::collections::{BTreeMap, BTreeSet};
mod allocation;
mod compilation;
mod funding;
use worth_foundational::facade::{AspectFieldLocator, AspectKey, FieldKey};

#[derive(Debug, Default)]
pub(crate) struct InstalledIndexRouting {
    maintenance: BTreeSet<crate::storage::substrate::maintenance::MaintenanceRoute>,
    entities: BTreeMap<KindId, Vec<DerivedIndexId>>,
    relations: BTreeMap<KindId, Vec<DerivedIndexId>>,
    entity_fields: BTreeMap<KindId, Vec<AspectFieldLocator>>,
    entity_aspect_fields: BTreeMap<KindId, BTreeMap<AspectKey, Vec<AspectFieldLocator>>>,
    entity_topology: BTreeMap<KindId, Vec<DerivedIndexId>>,
    relation_topology: BTreeMap<KindId, Vec<DerivedIndexId>>,
    fields: BTreeMap<(bool, KindId, AspectKey, FieldKey), Vec<DerivedIndexId>>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl InstalledIndexRouting {
    /// Existing installation entrance; shares the same compiler and catalog cache.
    pub(crate) fn compile<'a>(
        definitions: impl Iterator<Item = &'a DerivedIndexDefinition>,
        plans: &AspectContractPlanCatalog,
    ) -> Self {
        compilation::compile(definitions, plans, &mut |_, _| Ok(()))
            .expect("installed routing fits the admitted native geometry")
    }
    pub(crate) fn compile_for_request<'a>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        definitions: impl Iterator<Item = &'a DerivedIndexDefinition>,
        plans: &AspectContractPlanCatalog,
    ) -> Result<Self, super::DerivedIndexMaintenanceDenialKind> {
        let (mut routing, memory) = funding::run(execution, |pay| {
            pay(
                1,
                (std::mem::size_of::<Self>() + 2 * std::mem::size_of::<usize>()) as u64,
            )?;
            compilation::compile(definitions, plans, pay)
        })?;
        routing._memory = memory;
        Ok(routing)
    }

    pub(crate) fn maintenance_routes(
        &self,
    ) -> &BTreeSet<crate::storage::substrate::maintenance::MaintenanceRoute> {
        &self.maintenance
    }
    pub(crate) fn entities(&self, kind: KindId) -> &[DerivedIndexId] {
        self.entities.get(&kind).map_or(&[], Vec::as_slice)
    }
    pub(crate) fn relations(&self, kind: KindId) -> &[DerivedIndexId] {
        self.relations.get(&kind).map_or(&[], Vec::as_slice)
    }
    pub(crate) fn entity_fields(&self, kind: KindId) -> &[AspectFieldLocator] {
        self.entity_fields.get(&kind).map_or(&[], Vec::as_slice)
    }
    pub(crate) fn entity_fields_in_aspect(
        &self,
        kind: KindId,
        aspect: &AspectKey,
    ) -> &[AspectFieldLocator] {
        self.entity_aspect_fields
            .get(&kind)
            .and_then(|fields| fields.get(aspect))
            .map_or(&[], Vec::as_slice)
    }
    pub(crate) fn topology(&self, kind: KindId, entity: bool) -> &[DerivedIndexId] {
        let routes = if entity {
            &self.entity_topology
        } else {
            &self.relation_topology
        };
        routes.get(&kind).map_or(&[], Vec::as_slice)
    }
    pub(crate) fn field(
        &self,
        entity: bool,
        kind: KindId,
        aspect: &AspectKey,
        field: &FieldKey,
    ) -> &[DerivedIndexId] {
        self.fields
            .get(&(entity, kind, aspect.clone(), field.clone()))
            .map_or(&[], Vec::as_slice)
    }
    pub(crate) fn kind_count(&self) -> usize {
        self.entities.len().max(self.relations.len())
    }
}
