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
    fn conditional_binding() -> domain::WorthQueryApplicationConditionalOperationBinding<
        Schema,
        MutatePlanar,
        PlanarMutation,
        PlanarReadinessDomain,
        PlanarReadinessOperation,
        PlanarReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            MutatePlanar::reference::<Schema>(),
            operation_definition().reference(),
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
        _declaration: &worth_query_host::facade::declaration::application_schema::ErasedApplicationSchemaDeclaration,
    ) -> Result<
        application_contribution::WorthQueryApplicationConditionalPackageContract,
        worth_query_host::facade::application_contribution::WorthQueryOutputReadinessContractDenial,
    > {
        Ok({
            application_contribution::WorthQueryApplicationConditionalPackageContract::new(
                operation_definition().into_portable(),
                Self::conditional_binding().portable().clone(),
                PlanarReadyNode::reference().node_identity(),
            )
        })
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
            .installed_operation(MutatePlanar::reference::<Schema>())
            .unwrap();
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &Self::conditional_binding())
            .unwrap()
            .bind_node(PlanarReadyNode::reference())
            .unwrap();
        installation
            .bind_output_readiness::<InitialPlanarProducer<Schema>, _, _, _, _, _, _>(node, 0)
    }
}
pub(super) fn operation_definition() -> domain::WorthQueryDomainOperationDefinition<
    PlanarReadinessDomain,
    PlanarReadinessOperation,
    PlanarReadinessFamily,
> {
    let dependency = output_change_dependency();
    application_contribution::WorthQueryOutputReadinessContractBuilder::new(
        domain::WorthQueryDomainOperationIdentity::new("planar-output-readiness", 1),
        "planar-output-ready",
        output_change_projection(),
        canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("anchor").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("planar-readiness").unwrap(),
        "planar-readiness-v1",
    )
    .semantic_reads([output_change_projection()])
    .dependencies([dependency.clone()])
    .readiness_dependencies([dependency])
    .build()
    .expect("planar readiness declaration is canonical")
}

include!("readiness/source.rs");
