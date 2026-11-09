use super::CountedTotal;
use worth_query_host::facade::declaration::{
    authoring::{
        AspectFieldSelector, AuthoredQueryBundleRequest, AuthoredResultShapeField,
        DetailQueryBuilder, DetailResultShapeBuilder, RootEntityKey,
    },
    binding::QueryBindingDescriptor,
    canonicalization::canonicalize_request,
};
use worth_query_host::facade::{application_contribution, domain};

use super::{Input, Produce, Producer, TopologySchemaBinding};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CountedReadinessDomain;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CountedReadinessOperation;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CountedReadinessFamily;

worth_query_host::facade::worth_query_conditional_node!(
    pub CountedReadyNode in CountedReadinessDomain, CountedReadinessOperation,
    CountedReadinessFamily => operation "counted-output-ready"
);

pub struct Readiness<Schema>(std::marker::PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> Readiness<Schema> {
    fn conditional_binding(
        definition: &domain::WorthQueryDomainOperationDefinition<
            CountedReadinessDomain,
            CountedReadinessOperation,
            CountedReadinessFamily,
        >,
    ) -> domain::WorthQueryApplicationConditionalOperationBinding<
        Schema,
        Produce,
        Input,
        CountedReadinessDomain,
        CountedReadinessOperation,
        CountedReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            Produce::reference::<Schema>(),
            definition.reference(),
        )
    }
}

impl<Schema: TopologySchemaBinding>
    application_contribution::WorthQueryApplicationConditionalBinding<Schema>
    for Readiness<Schema>
{
    type Configuration = ();
    type Installed = ();
    type Operation = Produce;

    const IDENTITY: &'static str = "worth.query.certification.counted-readiness.v1";
    const REQUIRED_PRODUCERS: &'static [&'static str] = &["quote-count.producer"];

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
            .installed_operation(Produce::reference::<Schema>())
            .unwrap();
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &binding)
            .unwrap()
            .bind_node(CountedReadyNode::reference())
            .unwrap();
        installation.bind_output_readiness::<Producer<Schema>, _, _, _, _, _, _>(node, 0)
    }
}
pub(super) fn operation_contract(
    declaration: &worth_query_decl::facade::application_schema::ErasedApplicationSchemaDeclaration,
) -> Result<
    application_contribution::WorthQueryOutputReadinessContract<
        CountedReadinessDomain,
        CountedReadinessOperation,
        CountedReadinessFamily,
    >,
    application_contribution::WorthQueryOutputReadinessContractDenial,
> {
    application_contribution::WorthQueryOutputReadinessContractBuilder::for_field(
        declaration,
        CountedTotal::at("CountedTotal"),
        domain::WorthQueryDomainOperationIdentity::new("counted-output-readiness", 1),
        "counted-output-ready",
        canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("anchor").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("counted-readiness").unwrap(),
        "counted-readiness-v1",
    )?
    .field_dependency(
        declaration,
        CountedTotal::at("CountedTotal"),
        application_contribution::WorthQueryReadinessFieldUse::DeliveryAndEligibility,
    )?
    .build_typed()
}

pub(super) fn canonical_query(
) -> worth_query_host::facade::declaration::canonicalization::CanonicalQueryBundle {
    let query = DetailQueryBuilder::new(
        RootEntityKey::new(CountedTotal::at("CountedTotal").entity()).unwrap(),
    )
    .project(
        AspectFieldSelector::new(
            CountedTotal::at("CountedTotal").aspect(),
            CountedTotal::at("CountedTotal").field(),
        )
        .unwrap(),
    )
    .build()
    .unwrap()
    .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(
            AuthoredResultShapeField::new(
                CountedTotal::at("CountedTotal").aspect(),
                CountedTotal::at("CountedTotal").field(),
                "total",
            )
            .unwrap(),
        )
        .build()
        .unwrap()
        .into_raw();
    canonicalize_request(
        AuthoredQueryBundleRequest::for_ordinary_read(query, shape, QueryBindingDescriptor::new())
            .unwrap(),
    )
    .unwrap()
}
