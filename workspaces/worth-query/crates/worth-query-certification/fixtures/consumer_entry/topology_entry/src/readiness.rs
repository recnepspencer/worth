use super::runtime_domain as domain;
use super::{InitialPlanarProducer, MutatePlanar, PlanarMutation, TopologySchemaBinding};
use worth_query_host::facade::application_contribution;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanarReadinessDomain;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanarReadinessOperation;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanarReadinessFamily;

worth_query_host::facade::worth_query_conditional_node!(
    pub PlanarReadyNode in PlanarReadinessDomain, PlanarReadinessOperation,
    PlanarReadinessFamily => operation "planar-output-ready"
);

pub struct InitialPlanarReadiness<Schema>(std::marker::PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> InitialPlanarReadiness<Schema> {
    fn conditional_binding(
        definition: &domain::WorthQueryDomainOperationDefinition<
            PlanarReadinessDomain,
            PlanarReadinessOperation,
            PlanarReadinessFamily,
        >,
    ) -> domain::WorthQueryApplicationConditionalOperationBinding<
        Schema,
        MutatePlanar,
        PlanarMutation,
        PlanarReadinessDomain,
        PlanarReadinessOperation,
        PlanarReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            MutatePlanar::reference::<Schema>(),
            definition.reference(),
        )
    }
}

impl<Schema: TopologySchemaBinding>
    application_contribution::WorthQueryApplicationConditionalBinding<Schema>
    for InitialPlanarReadiness<Schema>
{
    type Configuration = ();
    type Installed = ();
    type Operation = MutatePlanar;

    const IDENTITY: &'static str = "worth.query.certification.planar-initial-readiness.v1";
    const REQUIRED_PRODUCERS: &'static [&'static str] =
        &["worth.query.certification.planar-initial.v1"];

    fn package_contract(
        declaration: &worth_query_host::facade::declaration::application_schema::ErasedApplicationSchemaDeclaration,
    ) -> Result<
        application_contribution::WorthQueryApplicationConditionalPackageContract,
        worth_query_host::facade::application_contribution::WorthQueryOutputReadinessContractDenial,
    > {
        let contract = operation_contract(declaration)?;
        let binding = Self::conditional_binding(contract.definition());
        Ok(contract.into_package_contract(binding.portable().clone()))
    }

    fn install(
        _: (),
        _: &application_contribution::WorthQueryApplicationConditionalProducerAccess<'_, Schema>,
        installation: &mut worth_query_host::facade::primary_graph::WorthQueryConditionalApplicationRuntimeInstallation<Schema>,
    ) -> Result<
        (),
        worth_query_host::facade::primary_graph::WorthQueryConditionalRuntimeInstallationDenial,
    > {
        let contract = operation_contract(installation.installed_schema().installed_declaration())?;
        let binding = Self::conditional_binding(contract.definition());

        let operation = installation
            .installed_schema()
            .installed_operation(MutatePlanar::reference::<Schema>())
            .unwrap();
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &binding)
            .unwrap()
            .bind_node(PlanarReadyNode::reference())
            .unwrap();
        installation
            .bind_output_readiness::<InitialPlanarProducer<Schema>, _, _, _, _, _, _>(node, 0)
    }
}
pub(super) fn operation_contract(
    declaration: &worth_query_decl::facade::application_schema::ErasedApplicationSchemaDeclaration,
) -> Result<
    application_contribution::WorthQueryOutputReadinessContract<
        PlanarReadinessDomain,
        PlanarReadinessOperation,
        PlanarReadinessFamily,
    >,
    application_contribution::WorthQueryOutputReadinessContractDenial,
> {
    application_contribution::WorthQueryOutputReadinessContractBuilder::for_field(
        declaration,
        delivery_field(),
        domain::WorthQueryDomainOperationIdentity::new("planar-output-readiness", 1),
        "planar-output-ready",
        canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("anchor").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("planar-readiness").unwrap(),
        "planar-readiness-v1",
    )?
    .field_dependency(
        declaration,
        delivery_field(),
        application_contribution::WorthQueryReadinessFieldUse::DeliveryAndEligibility,
    )?
    .build_typed()
}

include!("readiness/source.rs");
