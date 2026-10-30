//! Typed readiness follows the actual declared arithmetic output field.
use super::super::schema::*;
use super::calculation::{Calculate, CalculateAspectNumber, Producer};
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
    pub ReadyNode in ReadinessDomain, ReadinessOperation, ReadinessFamily => operation "numeric-aspect-calculation-ready"
);
pub struct Readiness;
type Contract =
    WorthQueryOutputReadinessContract<ReadinessDomain, ReadinessOperation, ReadinessFamily>;
type Binding = domain::WorthQueryApplicationConditionalOperationBinding<
    NumericSchema,
    CalculateAspectNumber,
    Calculate,
    ReadinessDomain,
    ReadinessOperation,
    ReadinessFamily,
>;
fn binding(contract: &Contract) -> Binding {
    Binding::declare(
        CalculateAspectNumber::reference(),
        contract.definition().reference(),
    )
}
fn contract(
    declaration: &ErasedApplicationSchemaDeclaration,
) -> Result<Contract, WorthQueryOutputReadinessContractDenial> {
    WorthQueryOutputReadinessContractBuilder::for_field(
        declaration,
        Calculated::reference(),
        domain::WorthQueryDomainOperationIdentity::new("numeric-aspect-calculation-readiness", 1),
        "numeric-aspect-calculation-ready",
        canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("aspect-calculated-number").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("numeric-aspect-calculation-readiness")
            .unwrap(),
        "numeric-aspect-calculation-readiness-v1",
    )?
    .field_dependency(
        declaration,
        Calculated::reference(),
        WorthQueryReadinessFieldUse::DeliveryAndEligibility,
    )?
    .build_typed()
}
impl WorthQueryApplicationConditionalBinding<NumericSchema> for Readiness {
    type Configuration = ();
    type Installed = ();
    type Operation = CalculateAspectNumber;
    const IDENTITY: &'static str = "numeric-aspect.readiness.v1";
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
            .installed_operation(CalculateAspectNumber::reference())
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
                Calculated::reference(),
            )
            .unwrap();
        installation.bind_output_readiness::<Producer, _, _, _, _, _, _>(node, ordinal)
    }
}
fn canonical_query() -> worth_query_host::facade::declaration::canonicalization::CanonicalQueryBundle
{
    let query = DetailQueryBuilder::new(RootEntityKey::new("Number").unwrap())
        .project(AspectFieldSelector::new("NumberFacts", "Calculated").unwrap())
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(
            AuthoredResultShapeField::new("NumberFacts", "Calculated", "calculated_number")
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
