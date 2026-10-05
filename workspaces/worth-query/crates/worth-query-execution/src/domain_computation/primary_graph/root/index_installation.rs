use std::collections::BTreeMap;

use worth_relational::facade::indexes::{
    DerivedIndexDefinition, DerivedIndexDefinitionLookup, DerivedIndexKind,
};
use worth_relational::facade::runtime::RelationalRuntime;

use super::super::schema_layout::WorthQueryPrimaryGraphLayout;

#[derive(Clone, Copy)]
pub(super) enum IndexInstallationPosture {
    Register,
    RequireRecovered,
}

enum IndexInstallation<'runtime> {
    Register(&'runtime RelationalRuntime),
    RequireRecovered(DerivedIndexDefinitionLookup),
}

impl<'runtime> IndexInstallation<'runtime> {
    fn new(runtime: &'runtime RelationalRuntime, posture: IndexInstallationPosture) -> Self {
        match posture {
            IndexInstallationPosture::Register => Self::Register(runtime),
            IndexInstallationPosture::RequireRecovered => {
                Self::RequireRecovered(runtime.index_access().definition_lookup_snapshot())
            }
        }
    }

    fn install(
        &self,
        definition: DerivedIndexDefinition,
    ) -> Result<DerivedIndexDefinition, String> {
        match self {
            Self::Register(runtime) => Ok(runtime.index_authority().register(definition)),
            Self::RequireRecovered(lookup) => {
                lookup.matching_definition(&definition).ok_or_else(|| {
                    format!(
                        "recovered runtime omitted Query index definition '{}'",
                        definition.name
                    )
                })
            }
        }
    }
}

pub(super) fn register_primary_graph_indexes(
    layout: &mut WorthQueryPrimaryGraphLayout,
    runtime: &RelationalRuntime,
    posture: IndexInstallationPosture,
) -> Result<(), String> {
    let installation = IndexInstallation::new(runtime, posture);
    let mut indexes_by_locator = BTreeMap::new();
    for (binding, binding_layout) in layout.principal_bindings_mut() {
        let installed = installation.install(DerivedIndexDefinition {
            index_id: worth_relational::facade::indexes::DerivedIndexId(0),
            name: format!("application-principal.{binding}"),
            kind: DerivedIndexKind::EntityField {
                field_locator: binding_layout.identity_locator.clone(),
            },
            branch_scoped: false,
        })?;
        binding_layout.index_id = installed.index_id;
        indexes_by_locator.insert(binding_layout.identity_locator.clone(), installed.index_id);
    }
    for ((entity, aspect, field), field_layout) in layout.equality_fields_mut() {
        let index_id = if let Some(index_id) = indexes_by_locator.get(&field_layout.locator) {
            *index_id
        } else {
            let installed = installation.install(DerivedIndexDefinition {
                index_id: worth_relational::facade::indexes::DerivedIndexId(0),
                name: format!("application-entity.{entity}.{aspect}.{field}"),
                kind: DerivedIndexKind::EntityField {
                    field_locator: field_layout.locator.clone(),
                },
                branch_scoped: false,
            })?;
            indexes_by_locator.insert(field_layout.locator.clone(), installed.index_id);
            installed.index_id
        };
        field_layout.equality_index_id = Some(index_id);
    }
    layout.register_continuation_orderings(|definition| {
        installation
            .install(definition)
            .map(|installed| installed.index_id)
    })?;
    layout.register_capability_grant_joins(|definition| {
        installation
            .install(definition)
            .map(|installed| installed.index_id)
    })?;
    let provider_idempotency = layout.provider_idempotency_mut();
    let installed = installation.install(DerivedIndexDefinition {
        index_id: worth_relational::facade::indexes::DerivedIndexId(0),
        name: "worth-query-provider.idempotency-key".to_owned(),
        kind: DerivedIndexKind::EntityField {
            field_locator: provider_idempotency.key_locator.clone(),
        },
        branch_scoped: false,
    })?;
    provider_idempotency.key_index_id = installed.index_id;
    let aftermath_causality = layout.provider_aftermath_causality_mut();
    let installed = installation.install(DerivedIndexDefinition {
        index_id: worth_relational::facade::indexes::DerivedIndexId(0),
        name: "worth-query-provider.aftermath-causality-key".to_owned(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aftermath_causality.key_locator.clone(),
        },
        branch_scoped: false,
    })?;
    aftermath_causality.key_index_id = installed.index_id;
    super::super::workflow::schema::register_indexes(layout.workflow_mut(), |definition| {
        installation.install(definition)
    })?;
    Ok(())
}

#[cfg(test)]
#[path = "index_installation_tests.rs"]
mod tests;
