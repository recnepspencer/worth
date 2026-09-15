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
    fn conditional_binding() -> domain::WorthQueryApplicationConditionalOperationBinding<
        Schema,
        PublishFinalPlanarOutput,
        FinalPlanarMutation,
        PlanarFinalReadinessDomain,
        PlanarFinalReadinessOperation,
        PlanarFinalReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            PublishFinalPlanarOutput::reference::<Schema>(),
            final_readiness_definition().reference(),
        )
    }
}

impl<Schema: TopologySchemaBinding>
    application_contribution::WorthQueryApplicationConditionalBinding<Schema>
    for PlanarFinalOutputReadiness<Schema>
{
    type Configuration = ();
    type Installed = ();

    const IDENTITY: &'static str = "worth.query.certification.planar-final-output-readiness.v1";
    const REQUIRED_PRODUCERS: &'static [&'static str] =
        &["worth.query.certification.planar-final-output-producer.v1"];

    fn package_contract(
    ) -> application_contribution::WorthQueryApplicationConditionalPackageContract {
        application_contribution::WorthQueryApplicationConditionalPackageContract::new(
            final_readiness_definition().into_portable(),
            Self::conditional_binding().portable().clone(),
            PlanarFinalReadyNode::reference().node_identity(),
        )
    }

    fn install(
        _: (),
        _: &application_contribution::WorthQueryApplicationConditionalProducerAccess<'_, Schema>,
        installation: &mut worth_query_host::facade::primary_graph::WorthQueryConditionalApplicationRuntimeInstallation<Schema>,
    ) -> Result<
        (),
        worth_query_host::facade::primary_graph::WorthQueryConditionalRuntimeInstallationDenial,
    > {
        let operation = installation
            .installed_schema()
            .installed_operation(PublishFinalPlanarOutput::reference::<Schema>())
            .unwrap();
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &Self::conditional_binding())
            .unwrap()
            .bind_node(PlanarFinalReadyNode::reference())
            .unwrap();
        installation
            .bind_output_readiness::<PlanarFinalOutputProducer<Schema>, _, _, _, _, _, _>(node, 0)
    }
}

pub(crate) fn declare_final_output<Schema: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let operation = PublishFinalPlanarOutput::reference::<Schema>();
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_decision_fact_budget(operation, 16)
        .operation_projection_work_budget(operation, 16)
        .operation_read_entity(operation, Body::reference())
        .operation_read_field(operation, BodyKey::reference())
        .operation_read_field(operation, Length::reference())
        .operation_write(operation, Length::reference())
        .operation_write(operation, BodyKey::reference())
        .operation_write(operation, super::PositionX::reference())
        .operation_write(operation, super::PositionY::reference())
        .operation_create(operation, Body::reference())
        .operation_link(operation, PlanarSuccessor::reference())
        .application_mutation_binding::<FinalPlanarMutationBinding<Schema>>()
}

fn final_readiness_definition() -> domain::WorthQueryDomainOperationDefinition<
    PlanarFinalReadinessDomain,
    PlanarFinalReadinessOperation,
    PlanarFinalReadinessFamily,
> {
    let dependency = super::super::readiness::output_change_dependency();
    application_contribution::WorthQueryOutputReadinessContractBuilder::new(
        domain::WorthQueryDomainOperationIdentity::new("planar-final-output-readiness", 1),
        "planar-final-output-ready",
        super::super::readiness::output_change_projection(),
        super::super::readiness::canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("anchor").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("planar-final-readiness").unwrap(),
        128,
        128,
        "planar-final-readiness-v1",
    )
    .semantic_reads([super::super::readiness::output_change_projection()])
    .dependencies([dependency.clone()])
    .readiness_dependencies([dependency])
    .build()
    .expect("final planar readiness declaration is canonical")
}
