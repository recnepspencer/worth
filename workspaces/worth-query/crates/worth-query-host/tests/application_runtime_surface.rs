use worth_query_declaration::{
    worth_query_application_schema, worth_query_aspect, worth_query_entity, worth_query_field,
    worth_query_principal_binding, worth_query_relation,
};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::declaration::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryPrincipalMappingStatus,
};
use worth_query_host::facade::domain::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};
use worth_query_host::facade::primary_graph::WorthQueryApplicationPrincipalKey;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
    WorthQueryPrincipalResolutionMode,
};
use worth_query_host::facade::runtime::WorthQueryExecutionRuntimeInstaller;

worth_query_application_schema! {
    pub schema HostIdentitySchema {
        owner: host_identity_test,
        version: (1, 0),
        members: |schema| {
            schema
                .entity(ExternalMapping::reference())
                .entity(Principal::reference())
                .entity(Account::reference())
                .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
                .aspect(Principal::reference(), PrincipalIdentity::reference())
                .field(ExternalMapping::reference(), ExternalIdentityField::reference())
                .field(ExternalMapping::reference(), MappingStatusField::reference())
                .field(Principal::reference(), PrincipalIdentityField::reference())
                .aspect(Account::reference(), AccountIdentity::reference())
                .field(Account::reference(), AccountNumber::reference())
                .relation(
                    MappingTarget::reference(),
                    ExternalMapping::reference(),
                    Principal::reference(),
                )
                .principal_binding(IdentityBinding::reference())
        }
    }
}

worth_query_entity!(pub ExternalMapping in HostIdentitySchema);
worth_query_entity!(pub Principal in HostIdentitySchema);
worth_query_entity!(pub Account in HostIdentitySchema);
worth_query_aspect!(pub ExternalIdentity in HostIdentitySchema, ExternalMapping; identity = AspectIdentity(0x9161103e), revision = AspectContractRevision(1),);
worth_query_field!(
    pub ExternalIdentityField in HostIdentitySchema, ExternalMapping, ExternalIdentity:
    WorthQueryExternalPrincipalIdentity, read_only, equality
);
worth_query_aspect!(pub PrincipalIdentity in HostIdentitySchema, Principal; identity = AspectIdentity(0x9161103f), revision = AspectContractRevision(1),);
worth_query_field!(
    pub PrincipalIdentityField in HostIdentitySchema, Principal, PrincipalIdentity:
    u64, read_only, equality
);
worth_query_field!(
    pub MappingStatusField in HostIdentitySchema, ExternalMapping, ExternalIdentity:
    WorthQueryPrincipalMappingStatus, read_write, equality
);
worth_query_relation!(
    pub MappingTarget in HostIdentitySchema,
    ExternalMapping => Principal
);
worth_query_principal_binding!(
    pub IdentityBinding in HostIdentitySchema,
    mapping ExternalMapping {
        identity: ExternalIdentityField,
        status: MappingStatusField,
        target: MappingTarget => Principal,
        principal_identity: PrincipalIdentityField
    }
);
worth_query_aspect!(pub AccountIdentity in HostIdentitySchema, Account; identity = AspectIdentity(0x91611040), revision = AspectContractRevision(1),);
worth_query_field!(
    pub AccountNumber in HostIdentitySchema, Account, AccountIdentity:
    String, read_only, equality
);

#[test]
fn host_facade_publishes_a_narrow_primary_graph_application_runtime() {
    let declaration = HostIdentitySchema::declaration().unwrap();
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "host_identity_test",
        1,
        0,
    ))
    .application_schema(declaration.clone())
    .validate()
    .unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("host-support", "host-config")
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
    let binding = schema
        .principal_binding(IdentityBinding::reference())
        .unwrap();
    let mut graph = authority
        .prepare_primary_graph(
            &runtime,
            &schema,
            worth_query_execution::facade::integration::product_world_resources_for_test(1_024),
        )
        .unwrap();
    graph
        .bind_principal(
            &binding,
            WorthQueryApplicationPrincipalKey::new("host-principal").unwrap(),
            1_u64,
            WorthQueryExternalPrincipalIdentity::new("https://issuer.example", "subject").unwrap(),
            WorthQueryPrincipalMappingStatus::Enabled,
        )
        .unwrap();
    graph
        .bind_entity(
            WorthQueryApplicationEntitySeed::new(
                Account::reference(),
                WorthQueryApplicationEntityKey::new("account-row").unwrap(),
            )
            .field(AccountNumber::reference(), "account-001".to_string()),
        )
        .unwrap();

    let application = graph
        .publish_application_runtime(
            runtime,
            authority,
            schema,
            worth_query_host::facade::primary_graph::SignalConditionalEvaluationBudget::development(
            ),
        )
        .unwrap();

    assert_eq!(application.publication().principal_binding_count(), 1);
    assert_eq!(application.publication().identity_index_count(), 1);
    assert_eq!(
        application.publication().application_equality_index_count(),
        4
    );
    assert_eq!(
        application.installed_schema().schema_name(),
        "HostIdentitySchema"
    );
    let cancellation = WorthQueryCancellationSource::new();
    let request_scope = WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(60),
        cancellation.token(),
    );
    let account = application
        .on_branch(application.current_world())
        .select()
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            AccountNumber::reference(),
            "account-001".to_string(),
            &request_scope,
            WorthQueryPrincipalResolutionMode::Certification,
        )
        .unwrap();
    assert_eq!(account.examined_candidate_count(), 1);
    assert_eq!(
        account.binding_identity(),
        application.publication().binding_identity()
    );
}
use std::time::{Duration, Instant};
