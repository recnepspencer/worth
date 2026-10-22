//! The real composed constructor refuses mismatched field meaning before effects.
use super::*;
use crate::domain_computation::primary_graph as graph;
use crate::facade::application_contribution as host;
use schema::ApplicationSchemaContributionAuthoring;
use std::{
    cell::Cell,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

worth_query_declaration::worth_query_application_schema!(schema Composed {
    owner: Composed, version: (1,0), members: |schema| {
        schema.contributions().register::<Contribution>()?
    }
});
impl Fixture for Composed {}
impl schema::ApplicationSchemaComposition for Composed {
    type Contributions = (Contribution,);
}
struct Contribution;
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
impl host::WorthQueryApplicationContribution<Composed> for Contribution {
    type Configuration = Arc<AtomicUsize>;
    fn contracts(
        contracts: &mut host::WorthQueryApplicationContributionContracts<Composed>,
    ) -> Result<(), graph::WorthQueryPrimaryGraphInstallationDenial> {
        contracts.conditional::<WrongReadiness>()?;
        Ok(())
    }
    fn configure(
        calls: Self::Configuration,
        _: &mut host::WorthQueryApplicationContributionSetup<'_, Composed>,
    ) -> Result<(), graph::WorthQueryPrimaryGraphInstallationDenial> {
        calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}
struct WrongReadiness;
impl host::WorthQueryApplicationConditionalBinding<Composed> for WrongReadiness {
    type Configuration = ();
    type Installed = ();
    type Operation = ();
    const IDENTITY: &'static str = "tests.typed-readiness.wrong.v1";
    fn package_contract(
        declaration: &schema::ErasedApplicationSchemaDeclaration,
    ) -> Result<
        host::WorthQueryApplicationConditionalPackageContract,
        WorthQueryOutputReadinessContractDenial,
    > {
        let _ = builder(declaration).field_dependency(
            declaration,
            changed::Record::reference::<Composed>(),
            WorthQueryReadinessFieldUse::DeliveryAndEligibility,
        )?;
        panic!("a field with a different nested wire grammar unexpectedly passed declaration admission");
    }
    fn install(
        _: (),
        _: &host::WorthQueryApplicationConditionalProducerAccess<'_, Composed>,
        _: &mut graph::WorthQueryConditionalApplicationRuntimeInstallation<Composed>,
    ) -> Result<(), graph::WorthQueryConditionalRuntimeInstallationDenial> {
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
    let limits = graph::application_installation::WorthQueryInMemoryApplicationLimits::new(
        crate::domain_computation::execution_runtime::product_world::test_product_world_resources(),
        crate::domain_computation::execution_runtime::WorthQueryApplicationQueryResourceProfile::new(
            std::num::NonZeroUsize::new(128).unwrap()),
        worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
    );
    let result = graph::application_installation::in_memory(
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
    let Err(graph::application_installation::WorthQueryInMemoryApplicationDenial::Contributions(
        cause,
    )) = result
    else {
        panic!("typed declaration refusal must precede all installation effects")
    };
    assert_eq!(
        cause.kind(),
        graph::WorthQueryPrimaryGraphInstallationDenialKind::ReadinessContractRejected
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
