//! The external producer installs the same real planar change projection in its own node.
use super::*;
use crate::readiness::canonical_query;
use worth_query_host::facade::{application_contribution as contribution, domain};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EscapingReadinessDomain;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EscapingReadinessOperation;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EscapingReadinessFamily;
worth_query_host::facade::worth_query_conditional_node!(
    pub EscapingReadyNode in EscapingReadinessDomain, EscapingReadinessOperation,
    EscapingReadinessFamily => operation "escaping-planar-ready"
);
pub struct EscapingPlanarReadiness<S>(PhantomData<fn() -> S>);
impl<S: TopologySchemaBinding> EscapingPlanarReadiness<S> {
    fn binding(
        definition: &domain::WorthQueryDomainOperationDefinition<
            EscapingReadinessDomain,
            EscapingReadinessOperation,
            EscapingReadinessFamily,
        >,
    ) -> domain::WorthQueryApplicationConditionalOperationBinding<
        S,
        EscapePlanar,
        EscapingPlanarInput,
        EscapingReadinessDomain,
        EscapingReadinessOperation,
        EscapingReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            EscapePlanar::reference::<S>(),
            definition.reference(),
        )
    }
}
impl<S: TopologySchemaBinding> contribution::WorthQueryApplicationConditionalBinding<S>
    for EscapingPlanarReadiness<S>
{
    type Configuration = ();
    type Installed = ();
    type Operation = EscapePlanar;
    const IDENTITY: &'static str = "worth.query.certification.escaping-planar-readiness.v1";
    const REQUIRED_PRODUCERS: &'static [&'static str] =
        &["worth.query.certification.escaping-planar-producer.v1"];
    fn package_contract(
        declaration: &worth_query_host::facade::declaration::application_schema::ErasedApplicationSchemaDeclaration,
    ) -> Result<
        contribution::WorthQueryApplicationConditionalPackageContract,
        contribution::WorthQueryOutputReadinessContractDenial,
    > {
        let contract = operation_contract(declaration)?;
        let binding = Self::binding(contract.definition());
        Ok(contract.into_package_contract(binding.portable().clone()))
    }
    fn install(
        _: (),
        _: &contribution::WorthQueryApplicationConditionalProducerAccess<'_, S>,
        installation: &mut worth_query_host::facade::primary_graph::WorthQueryConditionalApplicationRuntimeInstallation<S>,
    ) -> Result<
        (),
        worth_query_host::facade::primary_graph::WorthQueryConditionalRuntimeInstallationDenial,
    > {
        let contract = operation_contract(installation.installed_schema().installed_declaration())?;
        let binding = Self::binding(contract.definition());

        let operation = installation
            .installed_schema()
            .installed_operation(EscapePlanar::reference::<S>())
            .unwrap();
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &binding)
            .unwrap()
            .bind_node(EscapingReadyNode::reference())
            .unwrap();
        installation.bind_output_readiness::<EscapingPlanarProducer<S>, _, _, _, _, _, _>(node, 0)
    }
}
fn operation_contract(
    declaration: &worth_query_decl::facade::application_schema::ErasedApplicationSchemaDeclaration,
) -> Result<
    worth_query_host::facade::application_contribution::WorthQueryOutputReadinessContract<
        EscapingReadinessDomain,
        EscapingReadinessOperation,
        EscapingReadinessFamily,
    >,
    worth_query_host::facade::application_contribution::WorthQueryOutputReadinessContractDenial,
> {
    worth_query_host::facade::application_contribution::WorthQueryOutputReadinessContractBuilder::for_field(declaration,
crate::readiness::delivery_field(),
domain::WorthQueryDomainOperationIdentity::new("escaping-planar-readiness", 1),
"escaping-planar-ready",
canonical_query(),
domain::WorthQueryOperationProjectionRole::new("anchor").unwrap(),
domain::WorthQueryExecutionStrategyName::new("escaping-planar-readiness").unwrap(),
"escaping-planar-readiness-v1")?
.field_dependency(declaration, crate::readiness::delivery_field(), worth_query_host::facade::application_contribution::WorthQueryReadinessFieldUse::DeliveryAndEligibility)?
.build_typed()
}
