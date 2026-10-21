//! Conditional readiness bound to the real existing-family producer.
use super::super::producer::preserve::{Input, PreserveCompanion, Producer};
use super::*;

worth_query_host::facade::worth_query_conditional_node!(
    pub PreserveReadyNode in CompanionReadinessDomain,
    CompanionReadinessOperation,
    CompanionReadinessFamily => operation "canonical-companion-preserve-ready"
);

pub struct PreserveReadiness;
impl PreserveReadiness {
    fn conditional_binding() -> domain::WorthQueryApplicationConditionalOperationBinding<
        CanonicalSchema,
        PreserveCompanion,
        Input,
        CompanionReadinessDomain,
        CompanionReadinessOperation,
        CompanionReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            PreserveCompanion::reference(),
            operation_definition().reference(),
        )
    }
}
impl application_contribution::WorthQueryApplicationConditionalBinding<CanonicalSchema>
    for PreserveReadiness
{
    type Configuration = ();
    type Installed = ();
    type Operation = PreserveCompanion;
    const IDENTITY: &'static str = "canonical-companion.preserve.readiness.v1";
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
            .installed_operation(PreserveCompanion::reference())
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
    CompanionReadinessDomain,
    CompanionReadinessOperation,
    CompanionReadinessFamily,
> {
    let dependency = output_dependency();
    application_contribution::WorthQueryOutputReadinessContractBuilder::new(
        domain::WorthQueryDomainOperationIdentity::new("canonical-companion-preserve-readiness", 1),
        "canonical-companion-preserve-ready",
        output_projection(),
        canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("published-output").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("canonical-companion-preserve-readiness")
            .unwrap(),
        "canonical-companion-preserve-readiness-v1",
    )
    .semantic_reads([output_projection()])
    .dependencies([dependency.clone()])
    .readiness_dependencies([dependency])
    .build()
    .expect("preserve readiness declaration is canonical")
}
