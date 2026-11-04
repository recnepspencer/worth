pub use std::collections::BTreeMap;
pub use std::time::Duration;
pub use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionRefusalKind as PrincipalRefusal;

pub use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
pub use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
pub use worth_query_declaration::facade::application_schema::ApplicationScalarValueBinding;
pub use worth_query_declaration::facade::application_schema::U64ApplicationValueBinding;
pub use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentityBinding;
pub use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus;
pub use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatusBinding;
pub use worth_relational::facade::identity::PartitionId;
pub use worth_relational::facade::symbols::ClientKey;
pub use worth_relational::facade::transactions::AspectFieldPatch;
pub use worth_relational::facade::transactions::CreateIntent;
pub use worth_relational::facade::transactions::CreatedEntityRef;
pub use worth_relational::facade::transactions::EntityMutationIntent;
pub use worth_relational::facade::transactions::EntityReference;
pub use worth_relational::facade::transactions::EntitySpec;
pub use worth_relational::facade::transactions::MutationIntent;
pub use worth_relational::facade::transactions::RelationSpec;
pub use worth_relational::facade::transactions::UpdateEntityFieldsIntent;
pub use worth_relational::facade::transactions::WorkerIntentBatch;

pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::external_identity;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionDenialKind;
pub use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

#[test]
fn ambiguous_index_and_changed_mapping_revoke_application_principal_proof() {
    let world = installed_world(&[("alice", WorthQueryPrincipalMappingStatus::Enabled)]);
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
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
                    .selected_product(&(phase).test_host_request_for_test(),)
                    .resolve_authenticated_principal(
                        &phase,
                        &world.binding,
                        &ambiguous,
                        &scope,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap_err()
                    .kind(),
                WorthQueryPrincipalResolutionDenialKind::Refused(
                    PrincipalRefusal::AmbiguousPrincipal
                )
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
                .selected_product(&(phase).test_host_request_for_test())
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
            let host_custody = phase.test_host_request_for_test();
            disable_mapping(
                &host_custody.borrowed_request_for_test(),
                &fresh_world,
                principal.mapping_entity_id_for_test(),
            )
        })
        .expect("the fixture read owns its installed host call");
    fresh_world
        .application
        .with_host_advancement(|phase| {
            assert_eq!(
                fresh_world
                    .selected_product(&(phase).test_host_request_for_test(),)
                    .validate_authenticated_principal(&phase, &principal, &fresh_scope)
                    .unwrap_err()
                    .kind(),
                WorthQueryPrincipalResolutionDenialKind::Refused(
                    PrincipalRefusal::StalePrincipalProof
                )
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
                .selected_product(&(phase).test_host_request_for_test())
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
                    .selected_product(&(phase).test_host_request_for_test(),)
                    .validate_authenticated_principal(&phase, &principal, &scope)
                    .unwrap_err()
                    .kind(),
                WorthQueryPrincipalResolutionDenialKind::Refused(
                    PrincipalRefusal::ExpiredAuthentication
                )
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
                .selected_product(&(phase).test_host_request_for_test())
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
            let host_custody = phase.test_host_request_for_test();
            change_principal_identity(
                &host_custody.borrowed_request_for_test(),
                &world,
                principal.principal_entity_id_for_test(),
                2,
            )
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            assert_eq!(
                world
                    .selected_product(&(phase).test_host_request_for_test(),)
                    .validate_authenticated_principal(&phase, &principal, &scope)
                    .unwrap_err()
                    .kind(),
                WorthQueryPrincipalResolutionDenialKind::Refused(
                    PrincipalRefusal::StalePrincipalProof
                )
            )
        })
        .expect("the fixture read owns its installed host call");
}

pub fn change_principal_identity(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::IdentityWorld,
    principal_id: worth_relational::facade::identity::EntityId,
    identity: u64,
) {
    let graph = (*world.application.runtime_for_test())
        .primary_graph()
        .unwrap();
    let layout = (*graph.layout_for_test())
        .principal_binding_for_test(world.binding.binding())
        .unwrap()
        .clone();
    let fields = AspectFieldPatch::from(BTreeMap::from([(
        layout.principal_identity_locator_into_for_test(),
        U64ApplicationValueBinding::encode(&identity).unwrap(),
    )]));
    worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation_on_application(
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

pub fn disable_mapping(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::IdentityWorld,
    mapping_id: worth_relational::facade::identity::EntityId,
) {
    let graph = (*world.application.runtime_for_test())
        .primary_graph()
        .unwrap();
    let layout = (*graph.layout_for_test())
        .principal_binding_for_test(world.binding.binding())
        .unwrap()
        .clone();
    let fields = AspectFieldPatch::from(BTreeMap::from([(
        layout.status_locator_into_for_test(),
        WorthQueryPrincipalMappingStatusBinding::encode(
            &WorthQueryPrincipalMappingStatus::Disabled,
        )
        .unwrap(),
    )]));
    worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation_on_application(
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

pub fn append_duplicate_mapping(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::IdentityWorld,
    subject: &str,
) {
    let graph = (*world.application.runtime_for_test())
        .primary_graph()
        .unwrap();
    let layout = (*graph.layout_for_test())
        .principal_binding_for_test(world.binding.binding())
        .unwrap()
        .clone();
    {
        let partition_id = PartitionId::main();
        let principal_key = ClientKey::raw("duplicate-principal");
        let mapping_key = ClientKey::raw("duplicate-mapping");
        let principal = CreatedEntityRef {
            partition_id,
            kind_id: (*layout.principal_kind_for_test()),
            client_key: principal_key.clone(),
        };
        let mapping = CreatedEntityRef {
            partition_id,
            kind_id: (*layout.mapping_kind_for_test()),
            client_key: mapping_key.clone(),
        };
        // Read before the status locator moves the layout out.
        let principal_identity_locator = (*layout.principal_identity_locator_for_test()).clone();
        let relation_kind = (*layout.relation_kind_for_test());
        let fields = AspectFieldPatch::from(BTreeMap::from([
            (
                (*layout.identity_locator_for_test()).clone(),
                WorthQueryExternalPrincipalIdentityBinding::encode(&external_identity(subject))
                    .unwrap(),
            ),
            (
                layout.status_locator_into_for_test(),
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
                    principal_identity_locator,
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
                    kind_id: relation_kind,
                    client_key: ClientKey::raw("duplicate-target"),
                    source: EntityReference::Created(mapping),
                    target: EntityReference::Created(principal),
                    fields: AspectFieldPatch::default(),
                },
            )));
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation_on_application(
            execution,
            &world.application,
            batch,
        );
    }
}

#[path = "unknown_disabled_cancelled_and_cross_runtime_resolution_fail_clo/mod.rs"]
pub mod unknown_disabled_cancelled_and_cross_runtime_resolution_fail_clo;
