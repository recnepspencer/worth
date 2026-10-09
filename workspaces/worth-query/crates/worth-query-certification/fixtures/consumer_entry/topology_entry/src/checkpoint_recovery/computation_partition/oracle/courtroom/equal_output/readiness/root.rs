//! Readiness for the ordinary equal-output producer.
use super::super::*;
use worth_query_host::facade::application_contribution;
use worth_query_host::facade::domain;
use worth_query_host::facade::primary_graph::{
    WorthQueryConditionalApplicationRuntimeInstallation,
    WorthQueryConditionalRuntimeInstallationDenial,
};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RootDomain;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RootReadinessOperation;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RootReadinessFamily;

worth_query_host::facade::worth_query_conditional_node!(
    pub(super) RootReadyNode in RootDomain, RootReadinessOperation,
    RootReadinessFamily => operation "courtroom-equal-root-ready"
);

pub(in super::super) struct Readiness;

impl Readiness {
    fn conditional_binding() -> domain::WorthQueryApplicationConditionalOperationBinding<
        Schema,
        publication::PublishRoot,
        publication::Input,
        RootDomain,
        RootReadinessOperation,
        RootReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            publication::PublishRoot::reference::<Schema>(),
            operation_definition().reference(),
        )
    }
}

impl application_contribution::WorthQueryApplicationConditionalBinding<Schema> for Readiness {
    type Configuration = ();
    type Installed = ();
    type Operation = publication::PublishRoot;
    const IDENTITY: &'static str = "courtroom-equal-root-readiness";
    const REQUIRED_PRODUCERS: &'static [&'static str] = &[PRODUCER];

    fn package_contract(
    ) -> application_contribution::WorthQueryApplicationConditionalPackageContract {
        application_contribution::WorthQueryApplicationConditionalPackageContract::new(
            operation_definition().into_portable(),
            Self::conditional_binding().portable().clone(),
            RootReadyNode::reference().node_identity(),
        )
    }

    fn install(
        _: (),
        _: &application_contribution::WorthQueryApplicationConditionalProducerAccess<'_, Schema>,
        installation: &mut WorthQueryConditionalApplicationRuntimeInstallation<Schema>,
    ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
        let operation = installation
            .installed_schema()
            .installed_operation(publication::PublishRoot::reference::<Schema>())
            .unwrap();
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &Self::conditional_binding())
            .unwrap()
            .bind_node(RootReadyNode::reference())
            .unwrap();
        installation.bind_output_readiness::<Producer, _, _, _, _, _, _>(node, 0)
    }
}

fn operation_definition() -> domain::WorthQueryDomainOperationDefinition<
    RootDomain,
    RootReadinessOperation,
    RootReadinessFamily,
> {
    let dependency = crate::readiness::output_change_dependency();
    application_contribution::WorthQueryOutputReadinessContractBuilder::new(
        domain::WorthQueryDomainOperationIdentity::new("courtroom-equal-root-readiness", 1),
        "courtroom-equal-root-ready",
        crate::readiness::output_change_projection(),
        crate::readiness::canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("anchor").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("courtroom-equal-root-readiness").unwrap(),
        128,
        128,
        "courtroom-equal-root-readiness-v1",
    )
    .semantic_reads([crate::readiness::output_change_projection()])
    .dependencies([dependency.clone()])
    .readiness_dependencies([dependency])
    .build()
    .expect("the region output's readiness declaration is canonical")
}
