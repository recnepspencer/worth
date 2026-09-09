use std::collections::BTreeMap;
use std::time::Duration;

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus;
use worth_query_installation::facade::TypedApplicationValue;
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

#[test]
fn unknown_disabled_cancelled_and_cross_runtime_resolution_fail_closed() {
    let enabled = installed_world(&[("alice", WorthQueryPrincipalMappingStatus::Enabled)]);
    let scope = live_scope();
    let unknown = enabled.authenticate("unknown", Duration::from_secs(60), &scope);
    assert_eq!(
        enabled
            .selected_product()
            .resolve_authenticated_principal(
                &enabled.binding,
                unknown,
                &scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap_err()
            .kind(),
        WorthQueryPrincipalResolutionDenialKind::UnknownPrincipal
    );

    let disabled = installed_world(&[("bob", WorthQueryPrincipalMappingStatus::Disabled)]);
    let disabled_scope = live_scope();
    let disabled_external = disabled.authenticate("bob", Duration::from_secs(60), &disabled_scope);
    assert_eq!(
        disabled
            .selected_product()
            .resolve_authenticated_principal(
                &disabled.binding,
                disabled_external,
                &disabled_scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap_err()
            .kind(),
        WorthQueryPrincipalResolutionDenialKind::DisabledPrincipal
    );

    let cancellation = WorthQueryCancellationSource::new();
    let cancelled_external = enabled.authenticate("alice", Duration::from_secs(60), &live_scope());
    cancellation.cancel();
    let cancelled_scope = WorthQueryRequestScope::new(
        std::time::Instant::now() + Duration::from_secs(60),
        cancellation.token(),
    );
    assert_eq!(
        enabled
            .selected_product()
            .resolve_authenticated_principal(
                &enabled.binding,
                cancelled_external,
                &cancelled_scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap_err()
            .kind(),
        WorthQueryPrincipalResolutionDenialKind::Cancelled
    );

    let foreign = installed_world(&[("alice", WorthQueryPrincipalMappingStatus::Enabled)]);
    let foreign_external = enabled.authenticate("alice", Duration::from_secs(60), &live_scope());
    assert_eq!(
        foreign
            .selected_product()
            .resolve_authenticated_principal(
                &foreign.binding,
                foreign_external,
                &live_scope(),
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap_err()
            .kind(),
        WorthQueryPrincipalResolutionDenialKind::ForeignRuntime
    );
}

#[test]
fn ambiguous_index_and_changed_mapping_revoke_application_principal_proof() {
    let mut world = installed_world(&[("alice", WorthQueryPrincipalMappingStatus::Enabled)]);
    append_duplicate_mapping(&mut world, "alice");
    let scope = live_scope();
    let ambiguous = world.authenticate("alice", Duration::from_secs(60), &scope);
    assert_eq!(
        world
            .selected_product()
            .resolve_authenticated_principal(
                &world.binding,
                ambiguous,
                &scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap_err()
            .kind(),
        WorthQueryPrincipalResolutionDenialKind::AmbiguousPrincipal
    );

    let mut fresh_world = installed_world(&[("carol", WorthQueryPrincipalMappingStatus::Enabled)]);
    let fresh_scope = live_scope();
    let external = fresh_world.authenticate("carol", Duration::from_secs(60), &fresh_scope);
    let principal = fresh_world
        .selected_product()
        .resolve_authenticated_principal(
            &fresh_world.binding,
            external,
            &fresh_scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    disable_mapping(&mut fresh_world, principal.mapping_entity_id());
    assert_eq!(
        fresh_world
            .selected_product()
            .validate_authenticated_principal(&principal, &fresh_scope)
            .unwrap_err()
            .kind(),
        WorthQueryPrincipalResolutionDenialKind::StalePrincipalProof
    );
}

#[test]
fn application_principal_proof_expires_with_its_external_authentication() {
    let world = installed_world(&[("dana", WorthQueryPrincipalMappingStatus::Enabled)]);
    let scope = live_scope();
    let external = world.authenticate("dana", Duration::from_millis(20), &scope);
    let principal = world
        .selected_product()
        .resolve_authenticated_principal(
            &world.binding,
            external,
            &scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    std::thread::sleep(Duration::from_millis(30));
    assert_eq!(
        world
            .selected_product()
            .validate_authenticated_principal(&principal, &scope)
            .unwrap_err()
            .kind(),
        WorthQueryPrincipalResolutionDenialKind::ExpiredAuthentication
    );
}

#[test]
fn changed_typed_principal_identity_revokes_the_resolved_proof() {
    let mut world = installed_world(&[("erin", WorthQueryPrincipalMappingStatus::Enabled)]);
    let scope = live_scope();
    let external = world.authenticate("erin", Duration::from_secs(60), &scope);
    let principal = world
        .selected_product()
        .resolve_authenticated_principal(
            &world.binding,
            external,
            &scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    assert_eq!(*principal.principal_identity(), 1);
    change_principal_identity(&mut world, principal.principal_entity_id(), 2);
    assert_eq!(
        world
            .selected_product()
            .validate_authenticated_principal(&principal, &scope)
            .unwrap_err()
            .kind(),
        WorthQueryPrincipalResolutionDenialKind::StalePrincipalProof
    );
}

fn change_principal_identity(
    world: &mut super::fixture::IdentityWorld,
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
        identity.into_foundational_value(),
    )]));
    super::fixture::publish_relational_mutation_on_application(
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
    world: &mut super::fixture::IdentityWorld,
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
        WorthQueryPrincipalMappingStatus::Disabled.into_foundational_value(),
    )]));
    super::fixture::publish_relational_mutation_on_application(
        &world.application,
        WorkerIntentBatch::new("disable-mapping").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: mapping_id,
                fields,
            }),
        )),
    );
}

fn append_duplicate_mapping(world: &mut super::fixture::IdentityWorld, subject: &str) {
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
                external_identity(subject).into_foundational_value(),
            ),
            (
                layout.status_locator,
                WorthQueryPrincipalMappingStatus::Enabled.into_foundational_value(),
            ),
        ]));
        let batch = WorkerIntentBatch::new("duplicate-mapping")
            .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id,
                kind_id: principal.kind_id,
                client_key: principal_key,
                fields: AspectFieldPatch::from(BTreeMap::from([(
                    layout.principal_identity_locator.clone(),
                    99_u64.into_foundational_value(),
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
        super::fixture::publish_relational_mutation_on_application(&world.application, batch);
    }
}
