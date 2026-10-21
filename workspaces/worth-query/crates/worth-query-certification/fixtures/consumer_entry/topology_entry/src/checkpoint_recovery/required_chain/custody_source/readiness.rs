use super::{
    binding::{PublishValueSource, ValueSourceInput},
    TopologySchemaBinding, ValueSourceProducer,
};
use worth_query_host::facade::{application_contribution, domain};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::checkpoint_recovery::required_chain) struct ValueSourceReadinessDomain;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::checkpoint_recovery::required_chain) struct ValueSourceReadinessOperation;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::checkpoint_recovery::required_chain) struct ValueSourceReadinessFamily;

worth_query_host::facade::worth_query_conditional_node!(
    pub(in crate::checkpoint_recovery::required_chain) ValueSourceReadyNode in ValueSourceReadinessDomain, ValueSourceReadinessOperation,
    ValueSourceReadinessFamily => operation "consumed-value-source-output-ready"
);

pub(in crate::checkpoint_recovery::required_chain) struct ValueSourceReadiness<Schema>(
    std::marker::PhantomData<fn() -> Schema>,
);

impl<Schema: TopologySchemaBinding> ValueSourceReadiness<Schema> {
    fn conditional_binding() -> domain::WorthQueryApplicationConditionalOperationBinding<
        Schema,
        PublishValueSource,
        ValueSourceInput,
        ValueSourceReadinessDomain,
        ValueSourceReadinessOperation,
        ValueSourceReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            PublishValueSource::reference::<Schema>(),
            operation_definition().reference(),
        )
    }
}

impl<Schema: TopologySchemaBinding>
    application_contribution::WorthQueryApplicationConditionalBinding<Schema>
    for ValueSourceReadiness<Schema>
{
    type Configuration = ();
    type Installed = ();
    type Operation = PublishValueSource;
    const IDENTITY: &'static str = "worth.query.certification.consumed-value-source-readiness.v1";
    const REQUIRED_PRODUCERS: &'static [&'static str] =
        &["worth.query.certification.consumed-value-source-producer.v1"];

    fn package_contract(
    ) -> application_contribution::WorthQueryApplicationConditionalPackageContract {
        application_contribution::WorthQueryApplicationConditionalPackageContract::new(
            operation_definition().into_portable(),
            Self::conditional_binding().portable().clone(),
            ValueSourceReadyNode::reference().node_identity(),
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
            .installed_operation(PublishValueSource::reference::<Schema>())
            .unwrap();
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &Self::conditional_binding())
            .unwrap()
            .bind_node(ValueSourceReadyNode::reference())
            .unwrap();
        installation.bind_output_readiness::<ValueSourceProducer<Schema>, _, _, _, _, _, _>(node, 0)
    }
}

fn operation_definition() -> domain::WorthQueryDomainOperationDefinition<
    ValueSourceReadinessDomain,
    ValueSourceReadinessOperation,
    ValueSourceReadinessFamily,
> {
    let dependency = crate::readiness::output_change_dependency();
    application_contribution::WorthQueryOutputReadinessContractBuilder::new(
        domain::WorthQueryDomainOperationIdentity::new("consumed-value-source-output-readiness", 1),
        "consumed-value-source-output-ready",
        crate::readiness::output_change_projection(),
        crate::readiness::canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("anchor").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("consumed-value-source-readiness").unwrap(),
        "consumed-value-source-readiness-v1",
    )
    .semantic_reads([crate::readiness::output_change_projection()])
    .dependencies([dependency.clone()])
    .readiness_dependencies([dependency])
    .build()
    .expect("chain readiness declaration is canonical")
}
