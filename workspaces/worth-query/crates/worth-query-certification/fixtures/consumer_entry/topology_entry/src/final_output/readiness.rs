use super::*;

pub struct PlanarFinalOutputReadiness<Schema>(PhantomData<fn() -> Schema>);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanarFinalReadinessDomain;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanarFinalReadinessOperation;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanarFinalReadinessFamily;

worth_query_host::facade::worth_query_conditional_node!(
    pub PlanarFinalReadyNode in PlanarFinalReadinessDomain, PlanarFinalReadinessOperation,
    PlanarFinalReadinessFamily => operation "planar-final-output-ready"
);

impl<Schema: TopologySchemaBinding> PlanarFinalOutputReadiness<Schema> {
    fn conditional_binding(
        definition: &domain::WorthQueryDomainOperationDefinition<
            PlanarFinalReadinessDomain,
            PlanarFinalReadinessOperation,
            PlanarFinalReadinessFamily,
        >,
    ) -> domain::WorthQueryApplicationConditionalOperationBinding<
        Schema,
        PublishFinalPlanarOutput,
        FinalPlanarMutation,
        PlanarFinalReadinessDomain,
        PlanarFinalReadinessOperation,
        PlanarFinalReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            PublishFinalPlanarOutput::reference::<Schema>(),
            definition.reference(),
        )
    }
}

impl<Schema: TopologySchemaBinding>
    application_contribution::WorthQueryApplicationConditionalBinding<Schema>
    for PlanarFinalOutputReadiness<Schema>
{
    type Configuration = ();
    type Installed = ();
    type Operation = PublishFinalPlanarOutput;

    const IDENTITY: &'static str = "worth.query.certification.planar-final-output-readiness.v1";

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
            .installed_operation(PublishFinalPlanarOutput::reference::<Schema>())
            .unwrap();
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &binding)
            .unwrap()
            .bind_node(PlanarFinalReadyNode::reference())
            .unwrap();
        installation
            .bind_output_readiness::<PlanarFinalOutputProducer<Schema>, _, _, _, _, _, _>(node, 0)
    }
}

fn operation_contract(
    declaration: &worth_query_decl::facade::application_schema::ErasedApplicationSchemaDeclaration,
) -> Result<
    worth_query_host::facade::application_contribution::WorthQueryOutputReadinessContract<
        PlanarFinalReadinessDomain,
        PlanarFinalReadinessOperation,
        PlanarFinalReadinessFamily,
    >,
    worth_query_host::facade::application_contribution::WorthQueryOutputReadinessContractDenial,
> {
    worth_query_host::facade::application_contribution::WorthQueryOutputReadinessContractBuilder::for_field(declaration,
super::super::readiness::delivery_field(),
domain::WorthQueryDomainOperationIdentity::new("planar-final-output-readiness", 1),
"planar-final-output-ready",
super::super::readiness::canonical_query(),
domain::WorthQueryOperationProjectionRole::new("anchor").unwrap(),
domain::WorthQueryExecutionStrategyName::new("planar-final-readiness").unwrap(),
"planar-final-readiness-v1")?
.field_dependency(declaration, super::super::readiness::delivery_field(), worth_query_host::facade::application_contribution::WorthQueryReadinessFieldUse::DeliveryAndEligibility)?
.build_typed()
}
