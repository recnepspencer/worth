use super::application::Value;
use worth_query_host::facade::declaration::{
    authoring::{
        AspectFieldSelector, AuthoredQueryBundleRequest, AuthoredResultShapeField,
        DetailQueryBuilder, DetailResultShapeBuilder, RootEntityKey,
    },
    binding::QueryBindingDescriptor,
    canonicalization::canonicalize_request,
};
use worth_query_host::facade::{application_contribution, domain};

use super::{
    application::Schema,
    operation::{Apply, Input},
    producer::Producer,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanarReadinessDomain;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanarReadinessOperation;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanarReadinessFamily;

worth_query_host::facade::worth_query_conditional_node!(
    pub PlanarReadyNode in PlanarReadinessDomain, PlanarReadinessOperation,
    PlanarReadinessFamily => operation "matrix-output-ready"
);

pub struct Readiness;

impl Readiness {
    fn conditional_binding(
        definition: &domain::WorthQueryDomainOperationDefinition<
            PlanarReadinessDomain,
            PlanarReadinessOperation,
            PlanarReadinessFamily,
        >,
    ) -> domain::WorthQueryApplicationConditionalOperationBinding<
        Schema,
        Apply,
        Input,
        PlanarReadinessDomain,
        PlanarReadinessOperation,
        PlanarReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            Apply::reference::<Schema>(),
            definition.reference(),
        )
    }
}

impl application_contribution::WorthQueryApplicationConditionalBinding<Schema> for Readiness {
    type Configuration = ();
    type Installed = ();
    type Operation = Apply;

    const IDENTITY: &'static str = "worth.query.certification.matrix-initial-readiness.v1";
    const REQUIRED_PRODUCERS: &'static [&'static str] = &["matrix-producer"];

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
            .installed_operation(Apply::reference::<Schema>())
            .unwrap();
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &binding)
            .unwrap()
            .bind_node(PlanarReadyNode::reference())
            .unwrap();
        installation.bind_output_readiness::<Producer, _, _, _, _, _, _>(node, 0)
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
        Value::at("Value"),
        domain::WorthQueryDomainOperationIdentity::new("matrix-output-readiness", 1),
        "matrix-output-ready",
        canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("node").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("matrix-readiness").unwrap(),
        "matrix-readiness-v1",
    )?
    .field_dependency(
        declaration,
        Value::at("Value"),
        application_contribution::WorthQueryReadinessFieldUse::DeliveryAndEligibility,
    )?
    .build_typed()
}

pub(super) fn canonical_query(
) -> worth_query_host::facade::declaration::canonicalization::CanonicalQueryBundle {
    let query = DetailQueryBuilder::new(RootEntityKey::new(Value::at("Value").entity()).unwrap())
        .project(
            AspectFieldSelector::new(Value::at("Value").aspect(), Value::at("Value").field())
                .unwrap(),
        )
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(
            AuthoredResultShapeField::new(
                Value::at("Value").aspect(),
                Value::at("Value").field(),
                "length",
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
