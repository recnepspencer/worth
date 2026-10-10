use std::collections::BTreeMap;
use std::time::Duration;

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationScalarValueBinding, U64ApplicationValueBinding,
};
use worth_query_declaration::facade::authentication::{
    WorthQueryExternalPrincipalIdentityBinding, WorthQueryPrincipalMappingStatus,
    WorthQueryPrincipalMappingStatusBinding,
};
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    AspectFieldPatch, CreateIntent, CreatedEntityRef, EntityMutationIntent, EntityReference,
    EntitySpec, MutationIntent, RelationSpec, UpdateEntityFieldsIntent, WorkerIntentBatch,
};

use super::fixture::{external_identity, installed_world, live_scope};
use crate::domain_computation::primary_graph::{
    WorthQueryPrincipalResolutionDenialKind, WorthQueryPrincipalResolutionMode,
};

mod unknown_disabled_cancelled_and_cross_runtime_resolution_fail_clo;

#[test]
fn ambiguous_index_and_changed_mapping_revoke_application_principal_proof() {
    let world = installed_world(&[("alice", WorthQueryPrincipalMappingStatus::Enabled)]);
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            append_duplicate_mapping(&host_custody.borrowed_request_for_test(), &world, "alice")
        })
        .expect("the fixture read owns its installed host call");
    let scope = live_scope();
    let ambiguous = world.authenticate("alice", Duration::from_secs(60), &scope);
    world
        .application
        .with_host_advancement(|phase| {
            assert_eq!(
                world
                    .selected_product(&(phase).test_host_request(),)
                    .resolve_authenticated_principal(
                        &phase,
                        &world.binding,
                        &ambiguous,
                        &scope,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap_err()
                    .kind(),
                WorthQueryPrincipalResolutionDenialKind::AmbiguousPrincipal
            )
        })
        .expect("the fixture read owns its installed host call");

    let fresh_world = installed_world(&[("carol", WorthQueryPrincipalMappingStatus::Enabled)]);
    let fresh_scope = live_scope();
    let external = fresh_world.authenticate("carol", Duration::from_secs(60), &fresh_scope);
    let principal = fresh_world
        .application
        .with_host_advancement(|phase| {
            fresh_world
                .selected_product(&(phase).test_host_request())
                .resolve_authenticated_principal(
                    &phase,
                    &fresh_world.binding,
                    &external,
                    &fresh_scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    fresh_world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            disable_mapping(
                &host_custody.borrowed_request_for_test(),
                &fresh_world,
                principal.mapping_entity_id(),
            )
        })
        .expect("the fixture read owns its installed host call");
    fresh_world
        .application
        .with_host_advancement(|phase| {
            assert_eq!(
                fresh_world
                    .selected_product(&(phase).test_host_request(),)
                    .validate_authenticated_principal(&phase, &principal, &fresh_scope)
                    .unwrap_err()
                    .kind(),
                WorthQueryPrincipalResolutionDenialKind::StalePrincipalProof
            )
        })
        .expect("the fixture read owns its installed host call");
}

#[test]
fn application_principal_proof_expires_with_its_external_authentication() {
    let world = installed_world(&[("dana", WorthQueryPrincipalMappingStatus::Enabled)]);
    let scope = live_scope();
    let external = world.authenticate("dana", Duration::from_millis(20), &scope);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    std::thread::sleep(Duration::from_millis(30));
    world
        .application
        .with_host_advancement(|phase| {
            assert_eq!(
                world
                    .selected_product(&(phase).test_host_request(),)
                    .validate_authenticated_principal(&phase, &principal, &scope)
                    .unwrap_err()
                    .kind(),
                WorthQueryPrincipalResolutionDenialKind::ExpiredAuthentication
            )
        })
        .expect("the fixture read owns its installed host call");
}

#[test]
fn changed_typed_principal_identity_revokes_the_resolved_proof() {
    let world = installed_world(&[("erin", WorthQueryPrincipalMappingStatus::Enabled)]);
    let scope = live_scope();
    let external = world.authenticate("erin", Duration::from_secs(60), &scope);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(*principal.principal_identity(), 1);
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            change_principal_identity(
                &host_custody.borrowed_request_for_test(),
                &world,
                principal.principal_entity_id(),
                2,
            )
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            assert_eq!(
                world
                    .selected_product(&(phase).test_host_request(),)
                    .validate_authenticated_principal(&phase, &principal, &scope)
                    .unwrap_err()
                    .kind(),
                WorthQueryPrincipalResolutionDenialKind::StalePrincipalProof
            )
        })
        .expect("the fixture read owns its installed host call");
}

fn change_principal_identity(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &super::fixture::IdentityWorld,
    principal_id: worth_relational::facade::identity::EntityId,
    identity: u64,
) {
    let graph = world.application.runtime.primary_graph().unwrap();
    let layout = graph
        .layout
        .principal_binding(world.binding.binding())
        .unwrap()
        .clone();
    let fields = AspectFieldPatch::from(BTreeMap::from([(
        layout.principal_identity_locator,
        U64ApplicationValueBinding::encode(&identity).unwrap(),
    )]));
    super::fixture::publish_relational_mutation_on_application(
        execution,
        &world.application,
        WorkerIntentBatch::new("change-principal-identity").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: principal_id,
                fields,
            }),
        )),
    );
}

fn disable_mapping(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &super::fixture::IdentityWorld,
    mapping_id: worth_relational::facade::identity::EntityId,
) {
    let graph = world.application.runtime.primary_graph().unwrap();
    let layout = graph
        .layout
        .principal_binding(world.binding.binding())
        .unwrap()
        .clone();
    let fields = AspectFieldPatch::from(BTreeMap::from([(
        layout.status_locator,
        WorthQueryPrincipalMappingStatusBinding::encode(
            &WorthQueryPrincipalMappingStatus::Disabled,
        )
        .unwrap(),
    )]));
    super::fixture::publish_relational_mutation_on_application(
        execution,
        &world.application,
        WorkerIntentBatch::new("disable-mapping").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: mapping_id,
                fields,
            }),
        )),
    );
}

fn append_duplicate_mapping(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &super::fixture::IdentityWorld,
    subject: &str,
) {
    let graph = world.application.runtime.primary_graph().unwrap();
    let layout = graph
        .layout
        .principal_binding(world.binding.binding())
        .unwrap()
        .clone();
    {
        let partition_id = PartitionId::main();
        let principal_key = ClientKey::raw("duplicate-principal");
        let mapping_key = ClientKey::raw("duplicate-mapping");
        let principal = CreatedEntityRef {
            partition_id,
            kind_id: layout.principal_kind,
            client_key: principal_key.clone(),
        };
        let mapping = CreatedEntityRef {
            partition_id,
            kind_id: layout.mapping_kind,
            client_key: mapping_key.clone(),
        };
        let fields = AspectFieldPatch::from(BTreeMap::from([
            (
                layout.identity_locator.clone(),
                WorthQueryExternalPrincipalIdentityBinding::encode(&external_identity(subject))
                    .unwrap(),
            ),
            (
                layout.status_locator,
                WorthQueryPrincipalMappingStatusBinding::encode(
                    &WorthQueryPrincipalMappingStatus::Enabled,
                )
                .unwrap(),
            ),
        ]));
        let batch = WorkerIntentBatch::new("duplicate-mapping")
            .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id,
                kind_id: principal.kind_id,
                client_key: principal_key,
                fields: AspectFieldPatch::from(BTreeMap::from([(
                    layout.principal_identity_locator.clone(),
                    U64ApplicationValueBinding::encode(&99_u64).unwrap(),
                )])),
            })))
            .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id,
                kind_id: mapping.kind_id,
                client_key: mapping_key,
                fields,
            })))
            .push(MutationIntent::Create(CreateIntent::Relation(
                RelationSpec {
                    partition_id,
                    kind_id: layout.relation_kind,
                    client_key: ClientKey::raw("duplicate-target"),
                    source: EntityReference::Created(mapping),
                    target: EntityReference::Created(principal),
                    fields: AspectFieldPatch::default(),
                },
            )));
        super::fixture::publish_relational_mutation_on_application(
            execution,
            &world.application,
            batch,
        );
    }
}
