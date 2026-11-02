//! The external producer installs the same real planar change projection in its own node.
use super::*;
use crate::readiness::{canonical_query, output_change_dependency, output_change_projection};
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
    fn binding() -> domain::WorthQueryApplicationConditionalOperationBinding<
        S,
        EscapePlanar,
        EscapingPlanarInput,
        EscapingReadinessDomain,
        EscapingReadinessOperation,
        EscapingReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            EscapePlanar::reference::<S>(),
            definition().reference(),
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
        _: &worth_query_host::facade::declaration::application_schema::ErasedApplicationSchemaDeclaration,
    ) -> Result<
        contribution::WorthQueryApplicationConditionalPackageContract,
        contribution::WorthQueryOutputReadinessContractDenial,
    > {
        Ok(
            contribution::WorthQueryApplicationConditionalPackageContract::new(
                definition().into_portable(),
                Self::binding().portable().clone(),
                EscapingReadyNode::reference().node_identity(),
            ),
        )
    }
    fn install(
        _: (),
        _: &contribution::WorthQueryApplicationConditionalProducerAccess<'_, S>,
        installation: &mut worth_query_host::facade::primary_graph::WorthQueryConditionalApplicationRuntimeInstallation<S>,
    ) -> Result<
        (),
        worth_query_host::facade::primary_graph::WorthQueryConditionalRuntimeInstallationDenial,
    > {
        let operation = installation
            .installed_schema()
            .installed_operation(EscapePlanar::reference::<S>())
            .unwrap();
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &Self::binding())
            .unwrap()
            .bind_node(EscapingReadyNode::reference())
            .unwrap();
        installation.bind_output_readiness::<EscapingPlanarProducer<S>, _, _, _, _, _, _>(node, 0)
    }
}
fn definition() -> domain::WorthQueryDomainOperationDefinition<
    EscapingReadinessDomain,
    EscapingReadinessOperation,
    EscapingReadinessFamily,
> {
    let dependency = output_change_dependency();
    contribution::WorthQueryOutputReadinessContractBuilder::new(
        domain::WorthQueryDomainOperationIdentity::new("escaping-planar-readiness", 1),
        "escaping-planar-ready",
        output_change_projection(),
        canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("anchor").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("escaping-planar-readiness").unwrap(),
        "escaping-planar-readiness-v1",
    )
    .semantic_reads([output_change_projection()])
    .dependencies([dependency.clone()])
    .readiness_dependencies([dependency])
    .build()
    .unwrap()
}
