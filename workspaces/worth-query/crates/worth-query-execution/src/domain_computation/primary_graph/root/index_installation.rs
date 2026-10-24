use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

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
    let lineage = &mut aftermath_causality.lineage;
    for (name, locator, index_id) in [
        (
            "correction-reversal-root",
            &lineage.reversed_root,
            &mut lineage.reversal_root_index,
        ),
        (
            "correction-reversed-source",
            &lineage.reversed_source,
            &mut lineage.reversed_source_index,
        ),
        (
            "correction-consumed-reversal",
            &lineage.consumed_reversal,
            &mut lineage.consumed_reversal_index,
        ),
    ] {
        *index_id = installation
            .install(DerivedIndexDefinition {
                index_id: worth_relational::facade::indexes::DerivedIndexId(0),
                name: format!("worth-query-provider.{name}"),
                kind: DerivedIndexKind::EntityField {
                    field_locator: locator.clone(),
                },
                branch_scoped: false,
            })?
            .index_id;
    }
    super::super::workflow::schema::register_indexes(layout.workflow_mut(), |definition| {
        installation.install(definition)
    })?;
    let read_indexes = super::super::application_discovery::WorkflowReadIndexes::install(
        layout.workflow(),
        |definition| installation.install(definition),
    )?;
    layout.set_workflow_reads(read_indexes);
    Ok(())
}

/// The complete installed inventory, in the native identity order used by maintenance.
pub(super) fn primary_index_inventory(
    layout: &WorthQueryPrimaryGraphLayout,
) -> Arc<[worth_relational::facade::indexes::DerivedIndexId]> {
    let principal_identity_index_ids = layout
        .principal_bindings()
        .map(|(_, binding)| binding.index_id)
        .collect::<BTreeSet<_>>();
    principal_identity_index_ids
        .iter()
        .copied()
        .chain(layout.equality_index_ids())
        .chain(layout.continuation_ordering_index_ids())
        .chain(layout.capability_grant_join_index_ids())
        .chain(
            layout
                .workflow_reads()
                .into_iter()
                .flat_map(|indexes| [indexes.instances, indexes.transitions]),
        )
        .chain(std::iter::once(layout.provider_idempotency().key_index_id))
        .chain(std::iter::once(
            layout.provider_aftermath_causality().key_index_id,
        ))
        .chain([
            layout
                .provider_aftermath_causality()
                .lineage
                .reversal_root_index,
            layout
                .provider_aftermath_causality()
                .lineage
                .reversed_source_index,
            layout
                .provider_aftermath_causality()
                .lineage
                .consumed_reversal_index,
        ])
        .chain(std::iter::once(
            layout.workflow().instance.identity_index_id,
        ))
        .chain(std::iter::once(layout.workflow().lineage.identity_index_id))
        .chain(std::iter::once(
            layout.workflow().definition.content_identity_index_id,
        ))
        .chain(std::iter::once(
            layout.workflow().captured_input.identity_index_id,
        ))
        .chain(std::iter::once(
            layout.workflow().transition.identity_index_id,
        ))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .into()
}

#[cfg(test)]
#[path = "index_installation_tests.rs"]
mod tests;
