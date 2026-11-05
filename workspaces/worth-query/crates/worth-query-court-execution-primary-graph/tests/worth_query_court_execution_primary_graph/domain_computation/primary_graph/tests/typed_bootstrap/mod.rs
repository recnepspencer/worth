use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus;
use worth_query_installation::facade::WorthQueryInstallationAdmissionProfile;
use worth_query_installation::facade::WorthQueryInstallationGeneration;
use worth_query_installation::facade::WorthQueryPortableDomainIdentity;
use worth_query_installation::facade::WorthQueryPortableDomainPackage;

use worth_query_execution::facade::domain_computation::WorthQueryApplicationPrincipalKey;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntimeInstaller;
use worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphInstallationDenialKind;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::external_identity;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_world_with_policy_fact;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityBinding;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use worth_foundational::facade::AspectIdentity;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_declaration::facade::application_schema::ApplicationSchemaDeclaration;
use worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationBuilder;
use worth_query_declaration::facade::application_schema::U64ApplicationValueBinding;

#[test]
fn authenticated_bootstrap_registers_the_exact_installed_native_contract() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let phase = &active_phase;

        let declaration = IdentityExecutionSchema::declaration().unwrap();
        let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
            "identity_execution_test",
            1,
            0,
        ))
        .application_schema(declaration.clone())
        .validate()
        .unwrap();
        let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
            .admit(package)
            .unwrap();
        let installation = WorthQueryExecutionRuntimeInstaller::new()
            .install(WorthQueryInstallationGeneration::initial(), [admitted])
            .unwrap();
        let (runtime, authority) = installation.into_parts();
        let schema = runtime
            .installed_packages()
            .bind_application_schema(declaration)
            .unwrap();
        let installed = schema
            .native_contracts()
            .aspect("Principal", "PrincipalIdentity")
            .unwrap();
        let maximum_application_identity = schema
            .native_contracts()
            .maximum_aspect_identity()
            .unwrap()
            .0;

        let bootstrap = authority.prepare_primary_graph(&phase.bootstrap_for_test(), &runtime, &schema, worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::resources::test_product_world_resources(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system())).unwrap();
        let registered = (*bootstrap
            .graph_for_test())
            .registered_entity_aspect_for_test(
                &active_phase.borrowed_request_for_test(),
                "Principal",
                "PrincipalIdentity",
            )
            .unwrap();

        assert_eq!(&registered.contract, installed.contract());
        assert_eq!(&registered.binding, installed.binding());
        assert_eq!(
            (*bootstrap
                .graph_for_test())
                .registered_provider_aspect_identities_for_test(&active_phase.borrowed_request_for_test()),
            [
                AspectIdentity(maximum_application_identity + 1),
                AspectIdentity(maximum_application_identity + 2),
                AspectIdentity(maximum_application_identity + 3),
            ]
        );
    })
}

pub struct ExhaustedProviderIdentitySchema;
worth_query_declaration::worth_query_entity!(ExhaustedEntity for ExhaustedProviderIdentitySchema);
worth_query_declaration::worth_query_aspect!(
    ExhaustedAspect for ExhaustedProviderIdentitySchema, ExhaustedEntity;
    identity = AspectIdentity(u64::MAX - 2),
    revision = AspectContractRevision(1),
);
worth_query_declaration::worth_query_field!(
    ExhaustedField for ExhaustedProviderIdentitySchema, ExhaustedEntity, ExhaustedAspect:
    u64 => U64ApplicationValueBinding, read_only, no_equality
);

impl ApplicationSchema for ExhaustedProviderIdentitySchema {
    const OWNER: &'static str = "provider-identity-exhaustion";
    const NAME: &'static str = "ExhaustedProviderIdentitySchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration() -> Result<
        ApplicationSchemaDeclaration<Self>,
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        ApplicationSchemaDeclarationBuilder::for_schema()
            .entity(ExhaustedEntity::reference())
            .aspect(ExhaustedEntity::reference(), ExhaustedAspect::reference())
            .field(ExhaustedEntity::reference(), ExhaustedField::reference())
            .build()
    }
}

#[test]
fn provider_identity_exhaustion_denies_before_relational_installation() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let phase = &active_phase;

        let declaration = ExhaustedProviderIdentitySchema::declaration().unwrap();
        let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
            ExhaustedProviderIdentitySchema::OWNER,
            1,
            0,
        ))
        .application_schema(declaration.clone())
        .validate()
        .unwrap();
        let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
            .admit(package)
            .unwrap();
        let installation = WorthQueryExecutionRuntimeInstaller::new()
            .install(WorthQueryInstallationGeneration::initial(), [admitted])
            .unwrap();
        let (runtime, authority) = installation.into_parts();
        let schema = runtime
            .installed_packages()
            .bind_application_schema(declaration)
            .unwrap();
        let denial = authority
        .prepare_primary_graph(&phase.bootstrap_for_test(), &runtime, &schema, worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::resources::test_product_world_resources(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system()))
        .err()
        .unwrap();

        assert_eq!(
            denial.kind(),
            WorthQueryPrimaryGraphInstallationDenialKind::InvalidSchemaMember
        );
        assert_eq!(
            denial.subject(),
            "application schema exhausts Relational aspect-contract identity space"
        );
    })
}

#[test]
fn invalidation_keeps_no_window_so_a_finite_world_history_installs() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        use worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::resources::test_product_world_resources;
        use worth_query_execution::facade::integration::WorthQueryProductWorldResources;
        use worth_query_execution::facade::domain_computation::WorthQueryInvalidationResources;
        let declaration = IdentityExecutionSchema::declaration().unwrap();
        let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
            "identity_execution_test",
            1,
            0,
        ))
        .application_schema(declaration.clone())
        .validate()
        .unwrap();
        let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
            .admit(package)
            .unwrap();
        let installation = WorthQueryExecutionRuntimeInstaller::new()
            .install(WorthQueryInstallationGeneration::initial(), [admitted])
            .unwrap();
        let (runtime, authority) = installation.into_parts();
        let schema = runtime
            .installed_packages()
            .bind_application_schema(declaration)
            .unwrap();
        // Invalidation keeps no position the World no longer keeps a commit for.
        // Since ee07da2b0e it keeps no retention window at all, so even the
        // shortest finite World history admits it; nothing is refused.
        let (budgets, clock, _defaults, execution) = test_product_world_resources(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system()).into_parts_for_test();
        assert!(
            budgets.retained_composite_commits().get().is_some(),
            "explicit finite history policy"
        );
        let invalidation = WorthQueryInvalidationResources::new();
        let resources =
            WorthQueryProductWorldResources::new(budgets, clock, invalidation, *execution.policy_for_test());
        if let Err(denial) = authority.prepare_primary_graph(
            &phase.bootstrap_for_test(),
            &runtime,
            &schema,
            resources,
        ) {
            panic!(
                "a windowless invalidation fits any history: {:?}",
                denial.kind()
            );
        }
    });
}

#[test]
fn typed_policy_facts_publish_atomically_with_principal_identity() {
    let world = installed_world_with_policy_fact(
        &[("alice", WorthQueryPrincipalMappingStatus::Enabled)],
        true,
    );

    assert_eq!(world.application.publication().principal_binding_count(), 1);
    assert_eq!(world.application.publication().policy_entity_count(), 1);
    assert_eq!(world.application.publication().policy_relation_count(), 1);
}

#[test]
fn duplicate_typed_principal_identity_denies_without_poisoning_bootstrap() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let phase = &active_phase;

        let declaration = IdentityExecutionSchema::declaration().unwrap();
        let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
            "identity_execution_test",
            1,
            0,
        ))
        .application_schema(declaration.clone())
        .validate()
        .unwrap();
        let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
            .admit(package)
            .unwrap();
        let installation = WorthQueryExecutionRuntimeInstaller::new()
            .install(WorthQueryInstallationGeneration::initial(), [admitted])
            .unwrap();
        let (mut runtime, authority) = installation.into_parts();
        let schema = runtime
            .installed_packages()
            .bind_application_schema(declaration)
            .unwrap();
        let binding = schema
            .principal_binding(IdentityBinding::reference())
            .unwrap();
        let mut bootstrap = authority.prepare_primary_graph(&phase.bootstrap_for_test(), &runtime, &schema, worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::resources::test_product_world_resources(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system())).unwrap();

        bootstrap
            .bind_principal(
                &binding,
                WorthQueryApplicationPrincipalKey::new("principal-alice").unwrap(),
                7_u64,
                external_identity("alice"),
                WorthQueryPrincipalMappingStatus::Enabled,
            )
            .unwrap();
        let denial = bootstrap
            .bind_principal(
                &binding,
                WorthQueryApplicationPrincipalKey::new("principal-bob").unwrap(),
                7_u64,
                external_identity("bob"),
                WorthQueryPrincipalMappingStatus::Enabled,
            )
            .unwrap_err();
        assert_eq!(
            denial.kind(),
            WorthQueryPrimaryGraphInstallationDenialKind::DuplicatePrincipalIdentity
        );

        bootstrap
            .bind_principal(
                &binding,
                WorthQueryApplicationPrincipalKey::new("principal-bob").unwrap(),
                8_u64,
                external_identity("bob"),
                WorthQueryPrincipalMappingStatus::Enabled,
            )
            .expect("a denied duplicate identity must not reserve unrelated seed keys");
        let publication = bootstrap
            .publish_for_test(&phase.bootstrap_for_test(), &mut runtime, &authority)
            .unwrap();
        assert_eq!(publication.principal_binding_count(), 2);
    })
}
