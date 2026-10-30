//! Readiness observes the actual Scalar result, independently of its Struct sibling.
use super::super::schema::*;
use super::calculation::{Calculate, CalculateScalar, Producer};
use super::Total;
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
worth_query_host::facade::worth_query_conditional_node!(pub ReadyNode in ReadinessDomain, ReadinessOperation, ReadinessFamily => operation "numeric-scalar-ready");
pub struct Readiness;
type Contract =
    WorthQueryOutputReadinessContract<ReadinessDomain, ReadinessOperation, ReadinessFamily>;
type Binding = domain::WorthQueryApplicationConditionalOperationBinding<
    NumericSchema,
    CalculateScalar,
    Calculate,
    ReadinessDomain,
    ReadinessOperation,
    ReadinessFamily,
>;
fn binding(contract: &Contract) -> Binding {
    Binding::declare(
        CalculateScalar::reference(),
        contract.definition().reference(),
    )
}
fn contract(
    declaration: &ErasedApplicationSchemaDeclaration,
) -> Result<Contract, WorthQueryOutputReadinessContractDenial> {
    WorthQueryOutputReadinessContractBuilder::for_field(
        declaration,
        Total::reference(),
        domain::WorthQueryDomainOperationIdentity::new("numeric-scalar-readiness", 1),
        "numeric-scalar-ready",
        canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("scalar-calculated-number").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("numeric-scalar-readiness").unwrap(),
        "numeric-scalar-readiness-v1",
    )?
    .field_dependency(
        declaration,
        Total::reference(),
        WorthQueryReadinessFieldUse::DeliveryAndEligibility,
    )?
    .field_dependency(
        declaration,
        Calculated::reference(),
        WorthQueryReadinessFieldUse::Eligibility,
    )?
    .build_typed()
}
impl WorthQueryApplicationConditionalBinding<NumericSchema> for Readiness {
    type Configuration = ();
    type Installed = ();
    type Operation = CalculateScalar;
    const IDENTITY: &'static str = "numeric-scalar.readiness.v1";
    const REQUIRED_PRODUCERS: &'static [&'static str] =
        &[<Producer as WorthQueryApplicationProducerBinding<NumericSchema>>::IDENTITY];
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
        _: &WorthQueryApplicationConditionalProducerAccess<'_, NumericSchema>,
        installation: &mut worth_query_host::facade::primary_graph::WorthQueryConditionalApplicationRuntimeInstallation<NumericSchema>,
    ) -> Result<
        (),
        worth_query_host::facade::primary_graph::WorthQueryConditionalRuntimeInstallationDenial,
    > {
        let contract = contract(installation.installed_schema().installed_declaration()).unwrap();
        let operation = installation
            .installed_schema()
            .installed_operation(CalculateScalar::reference())
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
                Total::reference(),
            )
            .unwrap();
        installation.bind_output_readiness::<Producer, _, _, _, _, _, _>(node, ordinal)
    }
}
fn canonical_query() -> worth_query_host::facade::declaration::canonicalization::CanonicalQueryBundle
{
    let query = DetailQueryBuilder::new(RootEntityKey::new("Number").unwrap())
        .project(AspectFieldSelector::new("ScalarResult", "Total").unwrap())
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(AuthoredResultShapeField::new("ScalarResult", "Total", "scalar_total").unwrap())
        .build()
        .unwrap()
        .into_raw();
    canonicalize_request(
        AuthoredQueryBundleRequest::for_ordinary_read(query, shape, QueryBindingDescriptor::new())
            .unwrap(),
    )
    .unwrap()
}
