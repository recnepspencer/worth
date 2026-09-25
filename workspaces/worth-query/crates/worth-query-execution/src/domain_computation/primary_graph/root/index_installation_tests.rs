use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};
use worth_relational::facade::indexes::{DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind};
use worth_relational::facade::runtime::RelationalRuntimeApi;
use worth_relational::facade::schema::RelationalSchemaRegistry;

use super::*;
use crate::domain_computation::execution_runtime::WorthQueryExecutionRuntimeInstaller;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;

#[test]
fn recovered_index_binding_stays_exact_as_unrelated_definition_inventory_grows() {
    let mut query_definition_count = None;
    for unrelated in [1_000, 10_000] {
        let runtime = RelationalRuntimeApi::builder().build();
        let mut source_layout = installed_layout();
        let unrelated_locator = source_layout.provider_idempotency().key_locator.clone();
        for ordinal in 0..unrelated {
            runtime.index_authority().register(DerivedIndexDefinition {
                index_id: DerivedIndexId(0),
                name: format!("unrelated-index-{ordinal}"),
                kind: DerivedIndexKind::EntityField {
                    field_locator: unrelated_locator.clone(),
                },
                branch_scoped: false,
            });
        }
        register_primary_graph_indexes(
            &mut source_layout,
            &runtime,
            IndexInstallationPosture::Register,
        )
        .unwrap();
        let (binding, installed) = source_layout.principal_bindings().next().unwrap();
        let name = format!("application-principal.{binding}");
        let recovered_id = installed.index_id;
        assert!(recovered_id.0 > unrelated as u64);
        let snapshot_started = Instant::now();
        let lookup = runtime.index_access().definition_lookup_snapshot();
        let snapshot_elapsed = snapshot_started.elapsed();
        let local_query_definitions = lookup.definition_count() - unrelated;
        if let Some(expected) = query_definition_count {
            assert_eq!(local_query_definitions, expected);
        } else {
            query_definition_count = Some(local_query_definitions);
        }
        assert_eq!(lookup.candidate_count_for_name(&name), 1);

        let mut recovered_layout = installed_layout();
        let binding_started = Instant::now();
        register_primary_graph_indexes(
            &mut recovered_layout,
            &runtime,
            IndexInstallationPosture::RequireRecovered,
        )
        .unwrap();
        let binding_elapsed = binding_started.elapsed();
        eprintln!(
            "Query recovered index binding: unrelated={unrelated} query_definitions={local_query_definitions} snapshot={snapshot_elapsed:?} binding={binding_elapsed:?}"
        );
        assert_eq!(
            recovered_layout
                .principal_bindings()
                .next()
                .unwrap()
                .1
                .index_id,
            recovered_id,
            "recovery retains the owner-issued numeric identity at every inventory size"
        );
    }
}

#[test]
fn recovered_index_name_without_matching_kind_cannot_bind() {
    let runtime = RelationalRuntimeApi::builder().build();
    let mut expected = installed_layout();
    let (binding, principal) = expected.principal_bindings().next().unwrap();
    let name = format!("application-principal.{binding}");
    assert_ne!(
        principal.identity_locator,
        expected.provider_idempotency().key_locator
    );
    runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: name.clone(),
        kind: DerivedIndexKind::EntityField {
            field_locator: expected.provider_idempotency().key_locator.clone(),
        },
        branch_scoped: false,
    });

    let denial = register_primary_graph_indexes(
        &mut expected,
        &runtime,
        IndexInstallationPosture::RequireRecovered,
    )
    .unwrap_err();
    assert!(denial.contains(&name));
}

fn installed_layout() -> WorthQueryPrimaryGraphLayout {
    let declaration = IdentityExecutionSchema::declaration().unwrap();
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        IdentityExecutionSchema::OWNER,
        IdentityExecutionSchema::MAJOR,
        IdentityExecutionSchema::MINOR,
    ))
    .application_schema(declaration.clone())
    .validate()
    .unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
        .admit(package)
        .unwrap();
    let (runtime, _) = WorthQueryExecutionRuntimeInstaller::new()
        .install(WorthQueryInstallationGeneration::initial(), [admitted])
        .unwrap()
        .into_parts();
    let installed = runtime
        .installed_packages()
        .bind_application_schema(declaration)
        .unwrap();
    WorthQueryPrimaryGraphLayout::lower(
        installed.installed_declaration(),
        installed.native_contracts(),
        &RelationalSchemaRegistry::new(),
    )
    .unwrap()
    .0
}
use std::time::Instant;
