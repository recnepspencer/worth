use std::collections::{BTreeMap, BTreeSet};

use worth_foundational::facade::{
    AspectContract, AspectFieldLocator, AspectKey, AspectShape, FieldKey, FieldRequirement,
};
use worth_query_installation::facade::{
    ErasedApplicationSchemaDeclaration, WorthQueryInstalledApplicationSchemaContractCatalog,
};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::indexes::DerivedIndexId;
use worth_relational::facade::schema::RelationalSchemaRegistry;

use super::WorthQueryPrimaryGraphInstallationDenial;

mod application_layout_lowering;
mod capability_grant_join;
mod continuation_ordering;
mod installation_primitives;
mod platform_entity_lowering;
mod platform_identity_allocator;
mod principal_binding;
mod program_activation;
mod provider_aftermath_causality;
mod provider_dispatch_outbox;
mod provider_idempotency;
mod registry_lowering;

use super::workflow::schema::WorthQueryWorkflowLayout;
use crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxLayout;
use application_layout_lowering::{field_capability_keys, lower_fields, lower_relation_layouts};
use capability_grant_join::{lower_capability_grant_joins, WorthQueryCapabilityGrantJoinLayout};
use continuation_ordering::{
    lower_continuation_orderings, WorthQueryPrimaryContinuationOrderingLayout,
};
pub(in crate::domain_computation::primary_graph) use installation_primitives::{
    contract_space_exhausted, invalid_member, kind_space_exhausted, planned_field_locator,
    relational_schema_denial, required_kind, valid_aspect_key, valid_field_key,
};
use platform_entity_lowering::lower_platform_entities;
use principal_binding::lower_principal_bindings;
pub(in crate::domain_computation) use principal_binding::WorthQueryPrimaryPrincipalBindingLayout;
pub(in crate::domain_computation::primary_graph) use program_activation::WorthQueryProgramActivationLayout;
pub(super) use provider_aftermath_causality::WorthQueryAftermathCausalityLayout;
pub(super) use provider_idempotency::WorthQueryProviderIdempotencyLayout;
use registry_lowering::{
    lower_application_contract_bindings, lower_kind_ids, next_provider_kind_id,
    relational_schema_basis,
};
pub(in crate::domain_computation::primary_graph) use registry_lowering::{
    register_entity, register_relation,
};

#[derive(Debug)]
pub(in crate::domain_computation) struct WorthQueryPrimaryGraphLayout {
    principal_bindings: BTreeMap<String, WorthQueryPrimaryPrincipalBindingLayout>,
    entity_kinds: BTreeMap<String, KindId>,
    relation_kinds: BTreeMap<String, WorthQueryPrimaryRelationLayout>,
    application_entity_kinds: BTreeSet<KindId>,
    application_relation_kinds: BTreeSet<KindId>,
    fields: BTreeMap<(String, String, String), WorthQueryPrimaryFieldLayout>,
    aspect_contracts: BTreeMap<(String, AspectKey), AspectContract>,
    equality_field_keys: BTreeMap<AspectKey, BTreeSet<FieldKey>>,
    projection_field_keys: BTreeMap<AspectKey, BTreeSet<FieldKey>>,
    continuation_orderings: Vec<WorthQueryPrimaryContinuationOrderingLayout>,
    capability_grant_joins: BTreeMap<(String, String), WorthQueryCapabilityGrantJoinLayout>,
    provider_idempotency: WorthQueryProviderIdempotencyLayout,
    provider_dispatch_outbox: WorthQueryDispatchOutboxLayout,
    provider_aftermath_causality: WorthQueryAftermathCausalityLayout,
    program_activation: WorthQueryProgramActivationLayout,
    workflow: WorthQueryWorkflowLayout,
}

#[derive(Clone, Debug)]
pub(in crate::domain_computation) struct WorthQueryPrimaryRelationLayout {
    pub(in crate::domain_computation) kind: KindId,
    pub(in crate::domain_computation) from: KindId,
    pub(in crate::domain_computation) to: KindId,
    pub(in crate::domain_computation) source_max: Option<u64>,
}

#[derive(Clone, Debug)]
pub(in crate::domain_computation) struct WorthQueryPrimaryFieldLayout {
    pub(super) entity_kind: KindId,
    pub(super) locator: AspectFieldLocator,
    pub(super) equality_index_id: Option<DerivedIndexId>,
}

impl WorthQueryPrimaryGraphLayout {
    pub(super) fn lower(
        schema: &ErasedApplicationSchemaDeclaration,
        native_contracts: &WorthQueryInstalledApplicationSchemaContractCatalog,
        existing_registry: &RelationalSchemaRegistry,
    ) -> Result<(Self, RelationalSchemaRegistry), WorthQueryPrimaryGraphInstallationDenial> {
        let (entity_kinds, relation_kinds) = lower_kind_ids(schema, existing_registry)?;
        let (schema_id, schema_version_id) = relational_schema_basis(schema, existing_registry)?;
        let mut registry = RelationalSchemaRegistry::new();
        let mut aspect_contracts = BTreeMap::new();
        let lowered_contracts = lower_application_contract_bindings(native_contracts);
        let mut contracts_by_entity = lowered_contracts.by_entity;

        for (entity, kind_id) in &entity_kinds {
            let aspects = contracts_by_entity.remove(entity).unwrap_or_default();
            for binding in &aspects {
                aspect_contracts.insert(
                    (entity.clone(), binding.aspect_key()),
                    binding.contract.clone(),
                );
            }
            registry = register_entity(
                registry,
                &schema_id,
                schema_version_id,
                entity,
                *kind_id,
                aspects,
            )?;
        }
        for member in schema.members() {
            let worth_query_installation::facade::ApplicationSchemaMember::Relation {
                relation,
                from,
                to,
                integrity,
            } = member
            else {
                continue;
            };
            registry = register_relation(
                registry,
                &schema_id,
                schema_version_id,
                relation,
                required_kind(&relation_kinds, relation)?,
                required_kind(&entity_kinds, from)?,
                required_kind(&entity_kinds, to)?,
                *integrity,
            )?;
        }
        let provider_kind = next_provider_kind_id(
            existing_registry,
            entity_kinds.values().copied(),
            relation_kinds.values().copied(),
        )?;
        let (registry, platform_entities) = lower_platform_entities(
            registry,
            &schema_id,
            schema_version_id,
            native_contracts,
            provider_kind,
        )?;
        let principal_bindings = lower_principal_bindings(schema, &entity_kinds, &relation_kinds)?;
        let relation_layouts = lower_relation_layouts(schema, &entity_kinds, &relation_kinds)?;
        let application_entity_kinds = entity_kinds.values().copied().collect();
        let application_relation_kinds = relation_layouts
            .values()
            .map(|layout| layout.kind)
            .collect();
        let continuation_orderings =
            lower_continuation_orderings(schema, &entity_kinds, &relation_layouts)?;
        let capability_grant_joins =
            lower_capability_grant_joins(schema, &entity_kinds, &relation_layouts)?;
        let fields = lower_fields(schema, &entity_kinds)?;
        let equality_field_keys = field_capability_keys(
            fields
                .values()
                .filter(|layout| layout.equality_index_id.is_some()),
        );
        let projection_field_keys = field_capability_keys(fields.values());

        Ok((
            Self {
                principal_bindings,
                entity_kinds,
                relation_kinds: relation_layouts,
                application_entity_kinds,
                application_relation_kinds,
                fields,
                aspect_contracts,
                equality_field_keys,
                projection_field_keys,
                continuation_orderings,
                capability_grant_joins,
                provider_idempotency: platform_entities.provider_idempotency,
                provider_dispatch_outbox: platform_entities.provider_dispatch_outbox,
                provider_aftermath_causality: platform_entities.provider_aftermath_causality,
                program_activation: platform_entities.program_activation,
                workflow: platform_entities.workflow,
            },
            registry,
        ))
    }

    pub(in crate::domain_computation) fn principal_binding(
        &self,
        name: &str,
    ) -> Option<&WorthQueryPrimaryPrincipalBindingLayout> {
        self.principal_bindings.get(name)
    }

    pub(super) fn principal_bindings(
        &self,
    ) -> impl Iterator<Item = (&str, &WorthQueryPrimaryPrincipalBindingLayout)> {
        self.principal_bindings
            .iter()
            .map(|(name, layout)| (name.as_str(), layout))
    }

    pub(super) fn principal_bindings_mut(
        &mut self,
    ) -> impl Iterator<Item = (&str, &mut WorthQueryPrimaryPrincipalBindingLayout)> {
        self.principal_bindings
            .iter_mut()
            .map(|(name, layout)| (name.as_str(), layout))
    }

    pub(in crate::domain_computation) fn entity_kind(&self, entity: &str) -> Option<KindId> {
        self.entity_kinds.get(entity).copied()
    }

    pub(in crate::domain_computation::primary_graph) fn entity_name(
        &self,
        kind: KindId,
    ) -> Option<&str> {
        self.entity_kinds
            .iter()
            .find_map(|(name, candidate)| (*candidate == kind).then_some(name.as_str()))
    }

    pub(in crate::domain_computation) fn relation(
        &self,
        relation: &str,
    ) -> Option<&WorthQueryPrimaryRelationLayout> {
        self.relation_kinds.get(relation)
    }

    pub(super) fn field(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> Option<&WorthQueryPrimaryFieldLayout> {
        self.fields
            .get(&(entity.to_owned(), aspect.to_owned(), field.to_owned()))
    }

    pub(in crate::domain_computation) fn is_application_entity_kind(&self, kind: KindId) -> bool {
        self.application_entity_kinds.contains(&kind)
    }

    pub(in crate::domain_computation) fn is_application_relation_kind(&self, kind: KindId) -> bool {
        self.application_relation_kinds.contains(&kind)
    }

    #[cfg(test)]
    pub(in crate::domain_computation) fn application_entity_kind_without_create_scope(
        &self,
        touches: &worth_query_installation::facade::WorthQueryOperationTouchContract,
    ) -> Option<KindId> {
        self.entity_kinds
            .iter()
            .find(|(entity, _)| {
                !touches.scopes().iter().any(|scope| matches!(
                    scope,
                    worth_query_installation::facade::WorthQueryOperationTouchScope::CreateEntity(scope)
                        if scope.entity() == *entity
                ))
            })
            .map(|(_, kind)| *kind)
    }

    pub(in crate::domain_computation) fn field_locator(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> Option<&AspectFieldLocator> {
        self.fields
            .get(&(entity.to_string(), aspect.to_string(), field.to_string()))
            .map(|layout| &layout.locator)
    }

    pub(in crate::domain_computation) fn aspect_contract(
        &self,
        entity: &str,
        aspect: &AspectKey,
    ) -> Option<&AspectContract> {
        self.aspect_contracts
            .get(&(entity.to_string(), aspect.clone()))
    }

    pub(in crate::domain_computation) fn field_is_optional(
        &self,
        entity: &str,
        locator: &AspectFieldLocator,
    ) -> bool {
        let Some(field) = locator.field_path().fields().first() else {
            return false;
        };
        let Some(contract) = self.aspect_contract(entity, locator.aspect().aspect_key()) else {
            return false;
        };
        let AspectShape::Struct(shape) = contract.shape() else {
            return false;
        };
        shape
            .field(field)
            .is_some_and(|field| field.requirement() == FieldRequirement::Optional)
    }

    pub(in crate::domain_computation) fn equality_field(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> Option<&WorthQueryPrimaryFieldLayout> {
        self.fields
            .get(&(entity.to_string(), aspect.to_string(), field.to_string()))
            .filter(|layout| layout.equality_index_id.is_some())
    }

    pub(super) fn equality_fields_mut(
        &mut self,
    ) -> impl Iterator<Item = (&(String, String, String), &mut WorthQueryPrimaryFieldLayout)> {
        self.fields
            .iter_mut()
            .filter(|(_, layout)| layout.equality_index_id.is_some())
    }

    pub(super) fn equality_index_ids(&self) -> impl Iterator<Item = DerivedIndexId> + '_ {
        self.fields
            .values()
            .filter_map(|field| field.equality_index_id)
    }

    pub(super) fn supports_equality_field(&self, aspect: &AspectKey, field: &FieldKey) -> bool {
        self.equality_field_keys
            .get(aspect)
            .is_some_and(|fields| fields.contains(field))
    }

    pub(super) fn supports_projection_field(&self, aspect: &AspectKey, field: &FieldKey) -> bool {
        self.projection_field_keys
            .get(aspect)
            .is_some_and(|fields| fields.contains(field))
    }

    pub(super) const fn provider_idempotency(&self) -> &WorthQueryProviderIdempotencyLayout {
        &self.provider_idempotency
    }

    pub(super) fn provider_idempotency_mut(&mut self) -> &mut WorthQueryProviderIdempotencyLayout {
        &mut self.provider_idempotency
    }

    pub(super) const fn provider_dispatch_outbox(&self) -> &WorthQueryDispatchOutboxLayout {
        &self.provider_dispatch_outbox
    }

    pub(super) const fn provider_aftermath_causality(&self) -> &WorthQueryAftermathCausalityLayout {
        &self.provider_aftermath_causality
    }

    pub(super) fn provider_aftermath_causality_mut(
        &mut self,
    ) -> &mut WorthQueryAftermathCausalityLayout {
        &mut self.provider_aftermath_causality
    }

    pub(in crate::domain_computation::primary_graph) const fn program_activation(
        &self,
    ) -> &WorthQueryProgramActivationLayout {
        &self.program_activation
    }

    pub(in crate::domain_computation::primary_graph) const fn workflow(
        &self,
    ) -> &WorthQueryWorkflowLayout {
        &self.workflow
    }

    pub(super) fn workflow_mut(&mut self) -> &mut WorthQueryWorkflowLayout {
        &mut self.workflow
    }
}
