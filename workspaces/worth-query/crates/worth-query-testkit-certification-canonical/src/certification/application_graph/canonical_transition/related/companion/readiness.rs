//! Native readiness contract for the installed canonical-companion producer.

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
    super::schema::CanonicalSchema,
    producer::{CreateCompanion, Input, Producer},
    schema::CompanionValueField,
};

#[path = "preserve_readiness.rs"]
pub mod preserve;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompanionReadinessDomain;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompanionReadinessOperation;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompanionReadinessFamily;

worth_query_host::facade::worth_query_conditional_node!(
    pub CompanionReadyNode in CompanionReadinessDomain,
    CompanionReadinessOperation,
    CompanionReadinessFamily => operation "canonical-companion-ready"
);

pub struct CompanionReadiness;

impl CompanionReadiness {
    pub fn conditional_binding() -> domain::WorthQueryApplicationConditionalOperationBinding<
        CanonicalSchema,
        CreateCompanion,
        Input,
        CompanionReadinessDomain,
        CompanionReadinessOperation,
        CompanionReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            CreateCompanion::reference(),
            operation_definition().reference(),
        )
    }
}

impl application_contribution::WorthQueryApplicationConditionalBinding<CanonicalSchema>
    for CompanionReadiness
{
    type Configuration = ();
    type Installed = ();
    type Operation = CreateCompanion;

    const IDENTITY: &'static str = "canonical-companion.readiness.v1";
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
            .installed_operation(CreateCompanion::reference())
            .expect("the canonical companion publication operation is installed");
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &Self::conditional_binding())
            .expect("the canonical companion conditional binding is installed")
            .bind_node(CompanionReadyNode::reference())
            .expect("the published-output readiness node is installed");
        let contract =
            readiness_contract(installation.installed_schema().installed_declaration()).unwrap();
        let ordinal = contract
            .delivery_ordinal(
                installation.installed_schema(),
                node.declaration(),
                CompanionValueField::at("CompanionValueField"),
            )
            .unwrap();
        installation.bind_output_readiness::<Producer, _, _, _, _, _, _>(node, ordinal)
    }
}

type Contract = application_contribution::WorthQueryOutputReadinessContract<
    CompanionReadinessDomain,
    CompanionReadinessOperation,
    CompanionReadinessFamily,
>;
pub fn readiness_contract(
    declaration: &worth_query_host::facade::declaration::application_schema::ErasedApplicationSchemaDeclaration,
) -> Result<Contract, application_contribution::WorthQueryOutputReadinessContractDenial> {
    application_contribution::WorthQueryOutputReadinessContractBuilder::for_field(
        declaration,
        CompanionValueField::at("CompanionValueField"),
        domain::WorthQueryDomainOperationIdentity::new("canonical-companion-readiness", 1),
        "canonical-companion-ready",
        canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("published-output").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("canonical-companion-readiness").unwrap(),
        "canonical-companion-readiness-v1",
    )?
    .field_dependency(
        declaration,
        CompanionValueField::at("CompanionValueField"),
        application_contribution::WorthQueryReadinessFieldUse::DeliveryAndEligibility,
    )?
    .build_typed()
}
pub fn operation_definition() -> domain::WorthQueryDomainOperationDefinition<
    CompanionReadinessDomain,
    CompanionReadinessOperation,
    CompanionReadinessFamily,
> {
    readiness_contract(CanonicalSchema::declaration().unwrap().erased())
        .unwrap()
        .definition()
        .clone()
}

pub fn canonical_query(
) -> worth_query_host::facade::declaration::canonicalization::CanonicalQueryBundle {
    let query = DetailQueryBuilder::new(RootEntityKey::new("Companion").unwrap())
        .project(
            AspectFieldSelector::new(
                CompanionValueField::at("CompanionValueField").aspect(),
                "CompanionValueField",
            )
            .unwrap(),
        )
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(
            AuthoredResultShapeField::new(
                CompanionValueField::at("CompanionValueField").aspect(),
                "CompanionValueField",
                "published_companion_value",
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
