use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus;
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};

use crate::domain_computation::execution_runtime::WorthQueryExecutionRuntimeInstaller;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationPrincipalKey, WorthQueryPrimaryGraphInstallationDenialKind,
};

use super::fixture::{
    external_identity, installed_world_with_policy_fact, IdentityBinding, IdentityExecutionSchema,
};
use worth_foundational::facade::AspectIdentity;
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder,
    U64ApplicationValueBinding,
};

#[test]
fn authenticated_bootstrap_registers_the_exact_installed_native_contract() {
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

    let bootstrap = authority.prepare_primary_graph(&runtime, &schema, crate::domain_computation::execution_runtime::product_world::test_product_world_resources()).unwrap();
    let registered = bootstrap
        .graph
        .registered_entity_aspect("Principal", "PrincipalIdentity")
        .unwrap();

    assert_eq!(&registered.contract, installed.contract());
    assert_eq!(&registered.binding, installed.binding());
    assert_eq!(
        bootstrap.graph.registered_provider_aspect_identities(),
        [
            AspectIdentity(maximum_application_identity + 1),
            AspectIdentity(maximum_application_identity + 2),
            AspectIdentity(maximum_application_identity + 3),
        ]
    );
}

struct ExhaustedProviderIdentitySchema;
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
        .prepare_primary_graph(&runtime, &schema, crate::domain_computation::execution_runtime::product_world::test_product_world_resources())
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
    let mut bootstrap = authority.prepare_primary_graph(&runtime, &schema, crate::domain_computation::execution_runtime::product_world::test_product_world_resources()).unwrap();

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
    let publication = bootstrap.publish(&mut runtime, &authority).unwrap();
    assert_eq!(publication.principal_binding_count(), 2);
}
