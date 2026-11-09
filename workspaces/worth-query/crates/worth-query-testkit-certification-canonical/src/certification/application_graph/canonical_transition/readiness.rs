//! Native readiness contract for the installed canonical-derived producer.

pub use worth_query_host::facade::declaration::{
    authoring::{
        AspectFieldSelector, AuthoredQueryBundleRequest, AuthoredResultShapeField,
        DetailQueryBuilder, DetailResultShapeBuilder, RootEntityKey,
    },
    binding::QueryBindingDescriptor,
    canonicalization::canonicalize_request,
};
pub use worth_query_host::facade::{application_contribution, domain};

pub use super::{
    producer::{Input, Produce, Producer},
    schema::{CanonicalSchema, DerivedValueField},
};

#[path = "readiness/preserve.rs"]
pub mod preserve;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DerivedReadinessDomain;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DerivedReadinessOperation;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DerivedReadinessFamily;

worth_query_host::facade::worth_query_conditional_node!(
    pub DerivedReadyNode in DerivedReadinessDomain,
    DerivedReadinessOperation,
    DerivedReadinessFamily => operation "canonical-derived-ready"
);

pub struct DerivedReadiness;

impl DerivedReadiness {
    pub fn conditional_binding() -> domain::WorthQueryApplicationConditionalOperationBinding<
        CanonicalSchema,
        Produce,
        Input,
        DerivedReadinessDomain,
        DerivedReadinessOperation,
        DerivedReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            Produce::reference(),
            operation_definition().reference(),
        )
    }
}

impl application_contribution::WorthQueryApplicationConditionalBinding<CanonicalSchema>
    for DerivedReadiness
{
    type Configuration = ();
    type Installed = ();
    type Operation = Produce;

    const IDENTITY: &'static str = "canonical-transition.readiness.v1";
    const REQUIRED_PRODUCERS: &'static [&'static str] = &[
        <Producer as application_contribution::WorthQueryApplicationProducerBinding<
            CanonicalSchema,
        >>::IDENTITY,
    ];

    fn package_contract(
        _declaration: &worth_query_host::facade::declaration::application_schema::ErasedApplicationSchemaDeclaration,
    ) -> Result<
        application_contribution::WorthQueryApplicationConditionalPackageContract,
        worth_query_host::facade::application_contribution::WorthQueryOutputReadinessContractDenial,
    > {
        let contract = readiness_contract(_declaration)?;
        let binding = Self::conditional_binding();
        Ok(contract.into_package_contract(binding.portable().clone()))
    }

    fn install(
        _: (),
        _: &application_contribution::WorthQueryApplicationConditionalProducerAccess<
            '_,
            CanonicalSchema,
        >,
        installation: &mut worth_query_host::facade::primary_graph::WorthQueryConditionalApplicationRuntimeInstallation<CanonicalSchema>,
    ) -> Result<
        (),
        worth_query_host::facade::primary_graph::WorthQueryConditionalRuntimeInstallationDenial,
    > {
        let operation = installation
            .installed_schema()
            .installed_operation(Produce::reference())
            .expect("the canonical derived publication operation is installed");
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &Self::conditional_binding())
            .expect("the canonical derived conditional binding is installed")
            .bind_node(DerivedReadyNode::reference())
            .expect("the published-output readiness node is installed");
        let contract =
            readiness_contract(installation.installed_schema().installed_declaration()).unwrap();
        let ordinal = contract
            .delivery_ordinal(
                installation.installed_schema(),
                node.declaration(),
                DerivedValueField::at("DerivedValueField"),
            )
            .unwrap();
        installation.bind_output_readiness::<Producer, _, _, _, _, _, _>(node, ordinal)
    }
}

type Contract = application_contribution::WorthQueryOutputReadinessContract<
    DerivedReadinessDomain,
    DerivedReadinessOperation,
    DerivedReadinessFamily,
>;
pub fn readiness_contract(
    declaration: &worth_query_host::facade::declaration::application_schema::ErasedApplicationSchemaDeclaration,
) -> Result<Contract, application_contribution::WorthQueryOutputReadinessContractDenial> {
    application_contribution::WorthQueryOutputReadinessContractBuilder::for_field(
        declaration,
        DerivedValueField::at("DerivedValueField"),
        domain::WorthQueryDomainOperationIdentity::new("canonical-derived-readiness", 1),
        "canonical-derived-ready",
        canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("published-output").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("canonical-derived-readiness").unwrap(),
        "canonical-derived-readiness-v1",
    )?
    .field_dependency(
        declaration,
        DerivedValueField::at("DerivedValueField"),
        application_contribution::WorthQueryReadinessFieldUse::DeliveryAndEligibility,
    )?
    .build_typed()
}
pub fn operation_definition() -> domain::WorthQueryDomainOperationDefinition<
    DerivedReadinessDomain,
    DerivedReadinessOperation,
    DerivedReadinessFamily,
> {
    readiness_contract(<CanonicalSchema as worth_query_decl::facade::application_schema::ApplicationSchema>::declaration().unwrap().erased())
        .unwrap()
        .definition()
        .clone()
}

pub fn canonical_query(
) -> worth_query_host::facade::declaration::canonicalization::CanonicalQueryBundle {
    let query = DetailQueryBuilder::new(RootEntityKey::new("Derived").unwrap())
        .project(
            AspectFieldSelector::new(
                DerivedValueField::at("DerivedValueField").aspect(),
                "DerivedValueField",
            )
            .unwrap(),
        )
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(
            AuthoredResultShapeField::new(
                DerivedValueField::at("DerivedValueField").aspect(),
                "DerivedValueField",
                "published_derived_value",
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
