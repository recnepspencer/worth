//! Conditional readiness bound to the real existing-family producer.
use super::super::producer::preserve::{Input, Preserve, Producer};
use super::*;

worth_query_host::facade::worth_query_conditional_node!(
    pub PreserveReadyNode in DerivedReadinessDomain,
    DerivedReadinessOperation,
    DerivedReadinessFamily => operation "canonical-derived-preserve-ready"
);

pub struct PreserveReadiness;
impl PreserveReadiness {
    fn conditional_binding() -> domain::WorthQueryApplicationConditionalOperationBinding<
        CanonicalSchema,
        Preserve,
        Input,
        DerivedReadinessDomain,
        DerivedReadinessOperation,
        DerivedReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            Preserve::reference(),
            operation_definition().reference(),
        )
    }
}
impl application_contribution::WorthQueryApplicationConditionalBinding<CanonicalSchema>
    for PreserveReadiness
{
    type Configuration = ();
    type Installed = ();
    type Operation = Preserve;
    const IDENTITY: &'static str = "canonical-transition.preserve.readiness.v1";
    const REQUIRED_PRODUCERS: &'static [&'static str] = &[
        <Producer as application_contribution::WorthQueryApplicationProducerBinding<
            CanonicalSchema,
        >>::IDENTITY,
    ];
    fn package_contract(
    ) -> application_contribution::WorthQueryApplicationConditionalPackageContract {
        application_contribution::WorthQueryApplicationConditionalPackageContract::new(
            operation_definition().into_portable(),
            Self::conditional_binding().portable().clone(),
            PreserveReadyNode::reference().node_identity(),
        )
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
            .installed_operation(Preserve::reference())
            .expect("preserve operation is installed");
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &Self::conditional_binding())
            .expect("preserve conditional binding is installed")
            .bind_node(PreserveReadyNode::reference())
            .expect("preserve readiness node is installed");
        installation.bind_output_readiness::<Producer, _, _, _, _, _, _>(node, 0)
    }
}
fn operation_definition() -> domain::WorthQueryDomainOperationDefinition<
    DerivedReadinessDomain,
    DerivedReadinessOperation,
    DerivedReadinessFamily,
> {
    let dependency = output_dependency();
    application_contribution::WorthQueryOutputReadinessContractBuilder::new(
        domain::WorthQueryDomainOperationIdentity::new("canonical-derived-preserve-readiness", 1),
        "canonical-derived-preserve-ready",
        output_projection(),
        canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("published-output").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("canonical-derived-preserve-readiness")
            .unwrap(),
        32,
        32,
        "canonical-derived-preserve-readiness-v1",
    )
    .semantic_reads([output_projection()])
    .dependencies([dependency.clone()])
    .readiness_dependencies([dependency])
    .build()
    .expect("preserve readiness declaration is canonical")
}
