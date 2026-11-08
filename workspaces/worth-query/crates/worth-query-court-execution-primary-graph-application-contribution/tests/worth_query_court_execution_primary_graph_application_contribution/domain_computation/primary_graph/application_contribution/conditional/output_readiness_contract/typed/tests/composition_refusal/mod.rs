//! The real composed constructor refuses mismatched field meaning before effects.
use crate::domain_computation::primary_graph::application_contribution::conditional::output_readiness_contract::typed::tests::*;
use schema::ApplicationSchemaContributionAuthoring;
use std::cell::Cell;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;

worth_query_declaration::worth_query_application_schema!(schema Composed {
    owner: Composed, version: (1,0), members: |schema| {
        schema.contributions().register::<Contribution>()?
    }
});
impl Fixture for Composed {}
impl schema::ApplicationSchemaComposition for Composed {
    type Contributions = (Contribution,);
}
pub struct Contribution;
impl schema::ApplicationSchemaContribution<Composed> for Contribution {
    const IDENTITY: schema::ApplicationSchemaContributionIdentity =
        schema::ApplicationSchemaContributionIdentity::new("tests.typed-readiness.fields.v1");
    fn register_members(
        schema: schema::ApplicationSchemaDeclarationBuilder<Composed>,
    ) -> schema::ApplicationSchemaDeclarationBuilder<Composed> {
        schema
            .entity(Entity::reference())
            .aspect(Entity::reference(), Identity::reference())
            .field(Entity::reference(), Key::reference())
            .optional_aspect(Entity::reference(), Publication::reference())
            .field(Entity::reference(), Record::reference())
            .field(Entity::reference(), Sibling::reference())
    }
}
impl
    worth_query_execution::facade::application_contribution::WorthQueryApplicationContribution<
        Composed,
    > for Contribution
{
    type Configuration = Arc<AtomicUsize>;
    fn contracts(
        contracts: &mut worth_query_execution::facade::application_contribution::WorthQueryApplicationContributionContracts<Composed>,
    ) -> Result<
        (),
        worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphInstallationDenial,
    > {
        contracts.conditional::<WrongReadiness>()?;
        Ok(())
    }
    fn configure(
        calls: Self::Configuration,
        _: &mut worth_query_execution::facade::application_contribution::WorthQueryApplicationContributionSetup<'_, Composed>,
    ) -> Result<
        (),
        worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphInstallationDenial,
    > {
        calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}
pub struct WrongReadiness;
impl
    worth_query_execution::facade::application_contribution::WorthQueryApplicationConditionalBinding<
        Composed,
    > for WrongReadiness
{
    type Configuration = ();
    type Installed = ();
    type Operation = ();
    const IDENTITY: &'static str = "tests.typed-readiness.wrong.v1";
    fn package_contract(
        declaration: &schema::ErasedApplicationSchemaDeclaration,
    ) -> Result<
        worth_query_execution::facade::application_contribution::WorthQueryApplicationConditionalPackageContract,
        WorthQueryOutputReadinessContractDenial,
    >{
        let _ = builder(declaration).field_dependency(
            declaration,
            crate::domain_computation::primary_graph::application_contribution::conditional::output_readiness_contract::typed::tests::changed::Record::reference::<Composed>(),
            WorthQueryReadinessFieldUse::DeliveryAndEligibility,
        )?;
        panic!("a field with a different nested wire grammar unexpectedly passed declaration admission");
    }
    fn install(
        _: (),
        _: &worth_query_execution::facade::application_contribution::WorthQueryApplicationConditionalProducerAccess<'_, Composed>,
        _: &mut worth_query_execution::facade::domain_computation::WorthQueryConditionalApplicationRuntimeInstallation<Composed>,
    ) -> Result<(), worth_query_execution::facade::domain_computation::WorthQueryConditionalRuntimeInstallationDenial>{
        Ok(())
    }
}

#[test]
fn composed_field_refusal_preserves_configuration_and_seed() {
    let declaration = Composed::declaration().unwrap();
    // The same complete schema is independently a valid native installation.
    let package = domain::WorthQueryPortableDomainPackage::new(
        domain::WorthQueryPortableDomainIdentity::new(Composed::OWNER, 1, 0),
    )
    .application_schema(declaration.clone())
    .validate()
    .unwrap();
    domain::WorthQueryInstallationAdmissionProfile::new("readiness", "positive-schema")
        .admit(package)
        .unwrap();
    let configured = Arc::new(AtomicUsize::new(0));
    let seeded = Cell::new(0);
    let limits = worth_query_execution::facade::application_installation::WorthQueryApplicationLimits::new(
        worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::resources::test_product_world_resources(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system()),
        worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryResourceProfile::new(
            std::num::NonZeroUsize::new(128).unwrap()),
        worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
    );
    let result = worth_query_execution::facade::application_installation::in_memory(
        declaration,
        (configured.clone(),),
        worth_foundational::facade::ApplicationBuildIdentity::parse(
            "11111111-1111-1111-1111-111111111111",
        )
        .unwrap(),
        limits,
        |_, _, _| {
            seeded.set(seeded.get() + 1);
            Ok(())
        },
    );
    let Err(worth_query_execution::facade::application_installation::WorthQueryApplicationPreparationDenial::Contributions(
        cause,
    )) = result
    else {
        panic!("typed declaration refusal must precede all installation effects")
    };
    assert_eq!(
        cause.kind(),
        worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphInstallationDenialKind::ReadinessContractRejected
    );
    assert_eq!(
        cause.readiness_denial().unwrap().field_locus(),
        Some(&schema::ApplicationFieldBindingLocus::new(
            "Entity",
            "Publication",
            "Record"
        ))
    );
    assert_eq!(configured.load(Ordering::SeqCst), 0);
    assert_eq!(seeded.get(), 0);
}
