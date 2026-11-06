//! The ordinary readiness route required by the real producer installation.
use super::*;
use worth_query_decl::facade::{
    authoring::*, binding::QueryBindingDescriptor, canonicalization::*,
};
use worth_query_host::facade::domain;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Domain;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Operation;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Family;
worth_query_host::facade::worth_query_conditional_node!(pub ReadyNode in Domain, Operation, Family => operation "authored-ownership.producer.v1");
pub struct Readiness;
type Contract = WorthQueryOutputReadinessContract<Domain, Operation, Family>;
fn canonical_query() -> CanonicalQueryBundle {
    let query = DetailQueryBuilder::new(RootEntityKey::new("Node").unwrap())
        .project(AspectFieldSelector::new("Values", "Value").unwrap())
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(AuthoredResultShapeField::new("Values", "Value", "value").unwrap())
        .build()
        .unwrap()
        .into_raw();
    canonicalize_request(
        AuthoredQueryBundleRequest::for_ordinary_read(query, shape, QueryBindingDescriptor::new())
            .unwrap(),
    )
    .unwrap()
}
fn contract<S: Scenario>(
    declaration: &ErasedApplicationSchemaDeclaration,
) -> Result<Contract, WorthQueryOutputReadinessContractDenial> {
    WorthQueryOutputReadinessContractBuilder::for_field(
        declaration,
        Value::at("Value"),
        domain::WorthQueryDomainOperationIdentity::new("authored-ownership.producer.v1", 1),
        "authored-ownership.producer.v1",
        canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("root").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("authored-ownership.producer.v1").unwrap(),
        "authored-ownership.producer.v1",
    )?
    .field_dependency(
        declaration,
        Value::at("Value"),
        WorthQueryReadinessFieldUse::DeliveryAndEligibility,
    )?
    .build_typed()
}
fn binding<S: Scenario>(
    contract: &Contract,
) -> domain::WorthQueryApplicationConditionalOperationBinding<
    S,
    producer::Operation,
    query::Input,
    Domain,
    Operation,
    Family,
> {
    domain::WorthQueryApplicationConditionalOperationBinding::declare(
        ApplicationOperationRef::<S, producer::Operation, query::Input>::from_declaration(),
        contract.definition().reference(),
    )
}
impl<S: Scenario> WorthQueryApplicationConditionalBinding<S> for Readiness {
    type Configuration = ();
    type Installed = ();
    type Operation = producer::Operation;
    const IDENTITY: &'static str = "authored-ownership.producer.v1";
    const REQUIRED_PRODUCERS: &'static [&'static str] = &["authored-ownership.producer.v1"];
    fn package_contract(
        declaration: &ErasedApplicationSchemaDeclaration,
    ) -> Result<
        WorthQueryApplicationConditionalPackageContract,
        WorthQueryOutputReadinessContractDenial,
    > {
        let contract = contract::<S>(declaration)?;
        let binding = binding::<S>(&contract);
        Ok(contract.into_package_contract(binding.portable().clone()))
    }
    fn install(
        _: (),
        _: &WorthQueryApplicationConditionalProducerAccess<'_, S>,
        installation: &mut WorthQueryConditionalApplicationRuntimeInstallation<S>,
    ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
        let contract = contract::<S>(S::declaration().unwrap().erased()).unwrap();
        let binding = binding::<S>(&contract);
        let operation = installation
            .installed_schema()
            .installed_operation(ApplicationOperationRef::<
                S,
                producer::Operation,
                query::Input,
            >::from_declaration())
            .unwrap();
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &binding)
            .unwrap()
            .bind_node(ReadyNode::reference())
            .unwrap();
        let ordinal = contract
            .delivery_ordinal(
                installation.installed_schema(),
                node.declaration(),
                Value::at("Value"),
            )
            .unwrap();
        installation.bind_output_readiness::<producer::Producer<S>, _, _, _, _, _, _>(node, ordinal)
    }
}
