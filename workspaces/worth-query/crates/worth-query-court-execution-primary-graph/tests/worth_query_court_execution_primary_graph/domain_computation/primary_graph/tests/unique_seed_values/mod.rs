//! Installation seeds obey the unique law: one bootstrap writes each value of
//! a unique field once, and a refused seed reserves no value.

use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_declaration::facade::application_schema::ApplicationSchemaDeclaration;
use worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationBuilder;
use worth_query_declaration::facade::application_schema::U64ApplicationValueBinding;
use worth_query_installation::facade::WorthQueryInstallationAdmissionProfile;
use worth_query_installation::facade::WorthQueryInstallationGeneration;
use worth_query_installation::facade::WorthQueryPortableDomainIdentity;
use worth_query_installation::facade::WorthQueryPortableDomainPackage;

use worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::resources::test_product_world_resources;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntityKey;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntitySeed;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntimeInstaller;
use worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphBootstrap;
use worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphInstallationDenialKind;

pub struct UniqueSeedSchema;
worth_query_declaration::worth_query_entity!(Member for UniqueSeedSchema);
worth_query_declaration::worth_query_aspect!(
    MemberProfile for UniqueSeedSchema, Member;
    identity = AspectIdentity(0x5EED_0001),
    revision = AspectContractRevision(1),
);
worth_query_declaration::worth_query_field!(
    MemberHandle for UniqueSeedSchema, Member, MemberProfile:
    u64 => U64ApplicationValueBinding, read_only, equality
);
worth_query_declaration::worth_query_field!(
    MemberTier for UniqueSeedSchema, Member, MemberProfile:
    u64 => U64ApplicationValueBinding, read_write, equality
);

impl ApplicationSchema for UniqueSeedSchema {
    const OWNER: &'static str = "unique-seed-values";
    const NAME: &'static str = "UniqueSeedSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration() -> Result<
        ApplicationSchemaDeclaration<Self>,
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        ApplicationSchemaDeclarationBuilder::for_schema()
            .entity(Member::reference())
            .aspect(Member::reference(), MemberProfile::reference())
            .unique_field(Member::reference(), MemberHandle::reference())
            .field(Member::reference(), MemberTier::reference())
            .build()
    }
}

#[test]
fn two_seeds_of_one_unique_value_are_refused() {
    let mut bootstrap = prepared_bootstrap();
    bootstrap.bind_entity(member("ada", 7, 1)).unwrap();
    let denial = bootstrap.bind_entity(member("bob", 7, 2)).unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryPrimaryGraphInstallationDenialKind::DuplicateUniqueSeedValue
    );
    bootstrap
        .bind_entity(member("cy", 8, 2))
        .expect("a refused seed reserves no unique value");
}

#[test]
fn a_field_that_is_not_unique_may_repeat_across_seeds() {
    let mut bootstrap = prepared_bootstrap();
    bootstrap.bind_entity(member("ada", 7, 1)).unwrap();
    bootstrap
        .bind_entity(member("bob", 8, 1))
        .expect("only the unique field is checked");
}

pub fn member(
    key: &str,
    handle: u64,
    tier: u64,
) -> WorthQueryApplicationEntitySeed<UniqueSeedSchema, Member> {
    WorthQueryApplicationEntitySeed::new(
        Member::reference(),
        WorthQueryApplicationEntityKey::new(key).unwrap(),
    )
    .field(MemberHandle::reference(), handle)
    .field(MemberTier::reference(), tier)
}

pub fn prepared_bootstrap() -> WorthQueryPrimaryGraphBootstrap<UniqueSeedSchema> {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let phase = &active_phase;

        let declaration = UniqueSeedSchema::declaration().unwrap();
        let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
            UniqueSeedSchema::OWNER,
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
        authority
            .prepare_primary_graph(
                &phase.bootstrap_for_test(),
                &runtime,
                &schema,
                test_product_world_resources(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system()),
            )
            .unwrap()
    })
}
