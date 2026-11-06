//! Installation seeds obey the unique law: one bootstrap writes each value of
//! a unique field once, and a refused seed reserves no value.

#[path = "unique_seed_values/records.rs"]
mod storage_model;
pub use storage_model::*;
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder,
};
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};

use crate::domain_computation::execution_runtime::product_world::test_product_world_resources;
use crate::domain_computation::execution_runtime::WorthQueryExecutionRuntimeInstaller;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
    WorthQueryPrimaryGraphBootstrap, WorthQueryPrimaryGraphInstallationDenialKind,
};

pub struct UniqueSeedSchema;

impl ApplicationSchema for UniqueSeedSchema {
    const OWNER: &'static str = "unique-seed-values";
    const NAME: &'static str = "UniqueSeedSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration() -> Result<
        ApplicationSchemaDeclaration<Self>,
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        ApplicationSchemaDeclarationBuilder::from_fragment(UniqueRecords)
            .expect("owned record model is valid")
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

fn member(
    key: &str,
    handle: u64,
    tier: u64,
) -> WorthQueryApplicationEntitySeed<UniqueSeedSchema, Member> {
    WorthQueryApplicationEntitySeed::new(
        Member::reference(),
        WorthQueryApplicationEntityKey::new(key).unwrap(),
    )
    .field(MemberHandle::at("MemberHandle"), handle)
    .field(MemberTier::at("MemberTier"), tier)
}

fn prepared_bootstrap() -> WorthQueryPrimaryGraphBootstrap<UniqueSeedSchema> {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
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
