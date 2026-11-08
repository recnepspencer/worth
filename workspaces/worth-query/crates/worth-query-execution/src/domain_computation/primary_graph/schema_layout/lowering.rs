//! One compiler lowers ordinary and original-Source application layouts.
use super::*;
impl WorthQueryPrimaryGraphLayout {
    pub(in crate::domain_computation::primary_graph) fn lower(
        schema: &ErasedApplicationSchemaDeclaration,
        native_contracts: &WorthQueryInstalledApplicationSchemaContractCatalog,
        existing_registry: &RelationalSchemaRegistry,
        value_bindings: &worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog,
    ) -> Result<(Self, RelationalSchemaRegistry), WorthQueryPrimaryGraphInstallationDenial> {
        Self::lower_with_roles(
            schema,
            native_contracts,
            existing_registry,
            value_bindings,
            None,
            source_registration::SourceRegistration::new(None),
        )
    }

    pub(in crate::domain_computation::primary_graph) fn lower_with_original(
        schema: &ErasedApplicationSchemaDeclaration,
        native_contracts: &WorthQueryInstalledApplicationSchemaContractCatalog,
        existing_registry: &RelationalSchemaRegistry,
        value_bindings: &worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog,
        original: &RelationalSchemaRegistry,
        maximum_kind: Option<KindId>,
        maximum_aspect: Option<worth_foundational::facade::AspectIdentity>,
    ) -> Result<(Self, RelationalSchemaRegistry), WorthQueryPrimaryGraphInstallationDenial> {
        let roles = physical_identity::PhysicalIdentityBindings::readmit(
            schema,
            native_contracts,
            original,
            maximum_kind,
            maximum_aspect,
        )?;
        let result = Self::lower_with_roles(
            schema,
            native_contracts,
            existing_registry,
            value_bindings,
            roles.as_ref(),
            source_registration::SourceRegistration::new(Some(original)),
        )?;
        if let Some(roles) = &roles {
            roles.require_original(&result.1, original)?;
        }
        Ok(result)
    }

    pub(in crate::domain_computation::primary_graph) fn lower_successor(
        schema: &ErasedApplicationSchemaDeclaration,
        native_contracts: &WorthQueryInstalledApplicationSchemaContractCatalog,
        base: &RelationalSchemaRegistry,
        value_bindings: &worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog,
        predecessor: &Self,
    ) -> Result<(Self, RelationalSchemaRegistry), WorthQueryPrimaryGraphInstallationDenial> {
        Self::lower_with_roles(
            schema,
            native_contracts,
            base,
            value_bindings,
            Some(&predecessor.physical_roles),
            source_registration::SourceRegistration::new(None),
        )
    }

    // The physical-role handoff is distinct from registered occurrence retention.
    // Actual successor preparation consumes A's compiler-issued roles here with
    // genuine B declarations and a fresh B Source registration posture.
    pub(in crate::domain_computation::primary_graph) fn lower_with_roles(
        schema: &ErasedApplicationSchemaDeclaration,
        native_contracts: &WorthQueryInstalledApplicationSchemaContractCatalog,
        existing_registry: &RelationalSchemaRegistry,
        value_bindings: &worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog,
        previous_roles: Option<&physical_identity::PhysicalIdentityBindings>,
        sources: source_registration::SourceRegistration<'_>,
    ) -> Result<(Self, RelationalSchemaRegistry), WorthQueryPrimaryGraphInstallationDenial> {
        let mut roles = physical_identity::PhysicalIdentityAllocation::new(
            previous_roles,
            existing_registry,
            native_contracts,
        )?;
        let (entity_kinds, relation_kinds) = lower_kind_ids(schema, existing_registry, &mut roles)?;
        let (schema_id, schema_version_id) = relational_schema_basis(schema, existing_registry)?;
        let mut registry = content::lower(RelationalSchemaRegistry::new(), value_bindings)?;
        let mut aspect_contracts: BTreeMap<
            String,
            BTreeMap<AspectKey, DeclaredAspectContractBinding>,
        > = BTreeMap::new();
        let lowered_contracts = lower_application_contract_bindings(native_contracts);
        let relation_payload =
            relation_payload::ApplicationRelationPayloadLayout::lower(native_contracts)?;
        let mut contracts_by_entity = lowered_contracts.by_entity;
        let mut contracts_by_relation = lowered_contracts.by_relation;

        for (entity, kind_id) in &entity_kinds {
            let aspects = contracts_by_entity.remove(entity).unwrap_or_default();
            for binding in &aspects {
                aspect_contracts
                    .entry(entity.clone())
                    .or_default()
                    .insert(binding.aspect_key(), binding.clone());
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
            registry = register_application_relation(
                registry,
                &schema_id,
                schema_version_id,
                relation,
                required_kind(&relation_kinds, relation)?,
                required_kind(&entity_kinds, from)?,
                required_kind(&entity_kinds, to)?,
                integrity,
                &entity_kinds,
                contracts_by_relation.remove(relation).unwrap_or_default(),
            )?;
        }
        let provider_kind = roles.platform_first(|| {
            next_provider_kind_id(
                existing_registry,
                entity_kinds.values().copied(),
                relation_kinds.values().copied(),
            )
        })?;
        let (registry, platform_entities) = lower_platform_entities(
            registry,
            &schema_id,
            schema_version_id,
            native_contracts.maximum_aspect_identity(),
            provider_kind,
            entity_kinds.values().copied().collect(),
            &mut roles,
        )?;
        roles.reserve_registry(&registry);
        let principal_bindings = lower_principal_bindings(schema, &entity_kinds, &relation_kinds)?;
        let live_control_entity_kinds = live_control_roles::lower(schema, &entity_kinds)?;
        let (registry, projected_domains) = projected_member_domain::lower(
            sources,
            registry,
            schema,
            (&schema_id, schema_version_id),
            &entity_kinds,
        )?;
        let (registry, derived_relations) = derived_relation::lower(
            sources,
            registry,
            schema,
            (&schema_id, schema_version_id),
            &entity_kinds,
            &relation_kinds,
            &projected_domains,
        )?;
        let (registry, component_sets) = aggregate_identity::lower(
            registry,
            schema,
            native_contracts.maximum_aspect_identity(),
            &schema_id,
            schema_version_id,
            &entity_kinds,
            &relation_kinds,
            &mut roles,
        )?;
        let relation_layouts = lower_relation_layouts(schema, &entity_kinds, &relation_kinds)?;
        let (registry, bootstrap_publication) = bootstrap_publication::lower(
            registry,
            native_contracts.maximum_aspect_identity(),
            &schema_id,
            schema_version_id,
            &mut roles,
        )?;
        let signing = signed_entities::lower(schema, &entity_kinds)?;
        let signing_lifecycle = signing_lifecycle::SigningLifecycleLayout::lower(
            schema,
            &entity_kinds,
            &relation_kinds,
        )?;
        let continuation_orderings =
            lower_continuation_orderings(schema, &entity_kinds, &relation_layouts)?;
        let capability_grant_joins =
            lower_capability_grant_joins(schema, &entity_kinds, &relation_layouts)?;
        let fields = lower_fields(schema, &entity_kinds)?;
        let (registry, managed_sources) = managed_source::lower(
            sources,
            registry,
            schema,
            (&schema_id, schema_version_id),
            &entity_kinds,
            &relation_kinds,
            &fields,
        )?;
        let authorization_selectors =
            authorization_selectors::WorthQueryAuthorizationSelectors::lower(schema);
        let unique_fields = unique_fields::WorthQueryUniqueFieldNames::lower(&fields);
        let equality_field_keys =
            field_capability_keys(fields.values().filter(|layout| layout.equality_queryable));
        let projection_field_keys = field_capability_keys(fields.values());

        let physical_roles = roles.finish(&registry, entity_kinds, relation_kinds)?;
        Ok((
            Self {
                principal_bindings,
                live_control_entity_kinds,
                authorization_selectors,
                physical_roles,
                relation_kinds: relation_layouts,
                component_sets,
                derived_relations,
                managed_sources,
                projected_domains,
                signing,
                signing_lifecycle,
                fields,
                relation_payload,
                unique_fields,
                aspect_contracts,
                equality_field_keys,
                projection_field_keys,
                continuation_orderings,
                capability_grant_joins,
                provider_idempotency: platform_entities.provider_idempotency,
                provider_dispatch_outbox: platform_entities.provider_dispatch_outbox,
                provider_inbound_completion: platform_entities.provider_inbound_completion,
                provider_aftermath_causality: platform_entities.provider_aftermath_causality,
                program_activation: platform_entities.program_activation,
                workflow: platform_entities.workflow,
                workflow_reads: None,
                bootstrap_publication,
            },
            registry,
        ))
    }
}
