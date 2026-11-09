use super::{
    producer::ReplacementProducer, ReplacePlanarVertex, TopologySchemaBinding, VertexReplacement,
};
use worth_query_host::facade::{application_contribution, domain};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ReplacementReadinessDomain;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ReplacementReadinessOperation;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ReplacementReadinessFamily;

worth_query_host::facade::worth_query_conditional_node!(
    pub(super) ReplacementReadyNode in ReplacementReadinessDomain, ReplacementReadinessOperation,
    ReplacementReadinessFamily => operation "mixed-replacement-output-ready"
);

pub(super) struct ReplacementReadiness<Schema>(std::marker::PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> ReplacementReadiness<Schema> {
    fn conditional_binding(
        definition: &domain::WorthQueryDomainOperationDefinition<
            ReplacementReadinessDomain,
            ReplacementReadinessOperation,
            ReplacementReadinessFamily,
        >,
    ) -> domain::WorthQueryApplicationConditionalOperationBinding<
        Schema,
        ReplacePlanarVertex,
        VertexReplacement,
        ReplacementReadinessDomain,
        ReplacementReadinessOperation,
        ReplacementReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            ReplacePlanarVertex::reference::<Schema>(),
            definition.reference(),
        )
    }
}

impl<Schema: TopologySchemaBinding>
    application_contribution::WorthQueryApplicationConditionalBinding<Schema>
    for ReplacementReadiness<Schema>
{
    type Configuration = ();
    type Installed = ();
    type Operation = ReplacePlanarVertex;
    const IDENTITY: &'static str = "worth.query.certification.mixed-replacement-readiness.v1";
    const REQUIRED_PRODUCERS: &'static [&'static str] =
        &["worth.query.certification.mixed-replacement-producer.v1"];

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
            .installed_operation(ReplacePlanarVertex::reference::<Schema>())
            .unwrap();
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &binding)
            .unwrap()
            .bind_node(ReplacementReadyNode::reference())
            .unwrap();
        installation.bind_output_readiness::<ReplacementProducer<Schema>, _, _, _, _, _, _>(node, 0)
    }
}

fn operation_contract(
    declaration: &worth_query_decl::facade::application_schema::ErasedApplicationSchemaDeclaration,
) -> Result<
    application_contribution::WorthQueryOutputReadinessContract<
        ReplacementReadinessDomain,
        ReplacementReadinessOperation,
        ReplacementReadinessFamily,
    >,
    application_contribution::WorthQueryOutputReadinessContractDenial,
> {
    application_contribution::WorthQueryOutputReadinessContractBuilder::for_field(
        declaration,
        crate::readiness::delivery_field(),
        domain::WorthQueryDomainOperationIdentity::new("mixed-replacement-output-readiness", 1),
        "mixed-replacement-output-ready",
        crate::readiness::canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("replacement").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("mixed-replacement-readiness").unwrap(),
        "mixed-replacement-readiness-v1",
    )?
    .field_dependency(
        declaration,
        crate::readiness::delivery_field(),
        application_contribution::WorthQueryReadinessFieldUse::DeliveryAndEligibility,
    )?
    .build_typed()
}
