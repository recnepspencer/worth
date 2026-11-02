//! Typed readiness consumes the generated Edge identity and its complete declared codec.
use super::{
    super::super::schema::Catalog,
    super::schema::EdgeIdentity,
    operation::{Generate, GenerateGeometry},
    producer::Producer,
};
use worth_query_host::facade::{
    application_contribution::*,
    declaration::{
        application_schema::*, authoring::*, binding::QueryBindingDescriptor,
        canonicalization::canonicalize_request,
    },
    domain,
};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReadinessDomain;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReadinessOperation;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReadinessFamily;
worth_query_host::facade::worth_query_conditional_node!(
    pub ReadyNode in ReadinessDomain, ReadinessOperation, ReadinessFamily => operation "generated-geometry-ready"
);
pub struct Readiness;
type Contract =
    WorthQueryOutputReadinessContract<ReadinessDomain, ReadinessOperation, ReadinessFamily>;
type Binding = domain::WorthQueryApplicationConditionalOperationBinding<
    Catalog,
    GenerateGeometry,
    Generate,
    ReadinessDomain,
    ReadinessOperation,
    ReadinessFamily,
>;
fn binding(contract: &Contract) -> Binding {
    Binding::declare(
        GenerateGeometry::reference(),
        contract.definition().reference(),
    )
}
fn contract(
    declaration: &ErasedApplicationSchemaDeclaration,
) -> Result<Contract, WorthQueryOutputReadinessContractDenial> {
    WorthQueryOutputReadinessContractBuilder::for_field(
        declaration,
        EdgeIdentity::reference(),
        domain::WorthQueryDomainOperationIdentity::new("generated-geometry-readiness", 1),
        "generated-geometry-ready",
        canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("generated-edge").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("generated-geometry-readiness").unwrap(),
        "generated-geometry-readiness-v1",
    )?
    .field_dependency(
        declaration,
        EdgeIdentity::reference(),
        WorthQueryReadinessFieldUse::DeliveryAndEligibility,
    )?
    .build_typed()
}
impl WorthQueryApplicationConditionalBinding<Catalog> for Readiness {
    type Configuration = ();
    type Installed = ();
    type Operation = GenerateGeometry;
    const IDENTITY: &'static str = "maintained-generated-geometry.readiness.v1";
    const REQUIRED_PRODUCERS: &'static [&'static str] =
        &[<Producer as WorthQueryApplicationProducerBinding<Catalog>>::IDENTITY];
    fn package_contract(
        declaration: &ErasedApplicationSchemaDeclaration,
    ) -> Result<
        WorthQueryApplicationConditionalPackageContract,
        WorthQueryOutputReadinessContractDenial,
    > {
        let contract = contract(declaration)?;
        let binding = binding(&contract);
        Ok(contract.into_package_contract(binding.portable().clone()))
    }
    fn install(
        _: (),
        _: &WorthQueryApplicationConditionalProducerAccess<'_, Catalog>,
        installation: &mut worth_query_host::facade::primary_graph::WorthQueryConditionalApplicationRuntimeInstallation<Catalog>,
    ) -> Result<
        (),
        worth_query_host::facade::primary_graph::WorthQueryConditionalRuntimeInstallationDenial,
    > {
        let contract = contract(installation.installed_schema().installed_declaration()).unwrap();
        let operation = installation
            .installed_schema()
            .installed_operation(GenerateGeometry::reference())
            .unwrap();
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &binding(&contract))
            .unwrap()
            .bind_node(ReadyNode::reference())
            .unwrap();
        let ordinal = contract
            .delivery_ordinal(
                installation.installed_schema(),
                node.declaration(),
                EdgeIdentity::reference(),
            )
            .unwrap();
        installation.bind_output_readiness::<Producer, _, _, _, _, _, _>(node, ordinal)
    }
}
fn canonical_query() -> worth_query_host::facade::declaration::canonicalization::CanonicalQueryBundle
{
    let query = DetailQueryBuilder::new(RootEntityKey::new("Edge").unwrap())
        .project(AspectFieldSelector::new("EdgeFacts", "EdgeIdentity").unwrap())
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(AuthoredResultShapeField::new("EdgeFacts", "EdgeIdentity", "name").unwrap())
        .build()
        .unwrap()
        .into_raw();
    canonicalize_request(
        AuthoredQueryBundleRequest::for_ordinary_read(query, shape, QueryBindingDescriptor::new())
            .unwrap(),
    )
    .unwrap()
}
