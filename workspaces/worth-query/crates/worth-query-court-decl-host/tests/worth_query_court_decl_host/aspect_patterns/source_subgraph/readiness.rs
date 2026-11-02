//! Readiness selects the root; the Native witness includes all three roles.
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
worth_query_host::facade::worth_query_conditional_node!(pub Node in Domain, Operation, Family => operation "source-subgraph.producer.v1");
pub struct Readiness;
type Contract = WorthQueryOutputReadinessContract<Domain, Operation, Family>;
fn canonical_query() -> CanonicalQueryBundle {
    let query = DetailQueryBuilder::new(RootEntityKey::new("Generated").unwrap())
        .project(AspectFieldSelector::new("GeneratedFacts", "GeneratedValue").unwrap())
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(AuthoredResultShapeField::new("GeneratedFacts", "GeneratedValue", "value").unwrap())
        .build()
        .unwrap()
        .into_raw();
    canonicalize_request(
        AuthoredQueryBundleRequest::for_ordinary_read(query, shape, QueryBindingDescriptor::new())
            .unwrap(),
    )
    .unwrap()
}
fn contract(
    declaration: &ErasedApplicationSchemaDeclaration,
) -> Result<Contract, WorthQueryOutputReadinessContractDenial> {
    WorthQueryOutputReadinessContractBuilder::for_field(
        declaration,
        GeneratedValue::reference(),
        domain::WorthQueryDomainOperationIdentity::new("source-subgraph.producer.v1", 1),
        "source-subgraph.producer.v1",
        canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("root").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("source-subgraph.producer.v1").unwrap(),
        "source-subgraph.producer.v1",
    )?
    .field_dependency(
        declaration,
        GeneratedValue::reference(),
        WorthQueryReadinessFieldUse::DeliveryAndEligibility,
    )?
    .build_typed()
}
fn binding(
    contract: &Contract,
) -> domain::WorthQueryApplicationConditionalOperationBinding<
    Schema,
    generation::Generate,
    generation::Input,
    Domain,
    Operation,
    Family,
> {
    domain::WorthQueryApplicationConditionalOperationBinding::declare(
        generation::Generate::reference(),
        contract.definition().reference(),
    )
}
impl WorthQueryApplicationConditionalBinding<Schema> for Readiness {
    type Configuration = ();
    type Installed = ();
    type Operation = generation::Generate;
    const IDENTITY: &'static str = "source-subgraph.producer.v1";
    const REQUIRED_PRODUCERS: &'static [&'static str] = &["source-subgraph.producer.v1"];
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
        _: &WorthQueryApplicationConditionalProducerAccess<'_, Schema>,
        installation: &mut WorthQueryConditionalApplicationRuntimeInstallation<Schema>,
    ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
        let contract = contract(Schema::declaration().unwrap().erased()).unwrap();
        let binding = binding(&contract);
        let operation = installation
            .installed_schema()
            .installed_operation(generation::Generate::reference())
            .unwrap();
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &binding)
            .unwrap()
            .bind_node(Node::reference())
            .unwrap();
        let ordinal = contract
            .delivery_ordinal(
                installation.installed_schema(),
                node.declaration(),
                GeneratedValue::reference(),
            )
            .unwrap();
        installation.bind_output_readiness::<producer::Producer, _, _, _, _, _, _>(node, ordinal)
    }
}
