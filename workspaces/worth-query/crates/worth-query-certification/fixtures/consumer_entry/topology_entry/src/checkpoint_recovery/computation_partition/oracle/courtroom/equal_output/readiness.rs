//! Readiness for the ordinary equal-output producer.
pub(super) mod root;
use super::*;
use worth_query_host::facade::application_contribution;
use worth_query_host::facade::domain;
use worth_query_host::facade::primary_graph::{
    WorthQueryConditionalApplicationRuntimeInstallation,
    WorthQueryConditionalRuntimeInstallationDenial,
};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RegionOutputDomain;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RegionOutputReadinessOperation;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RegionOutputReadinessFamily;

worth_query_host::facade::worth_query_conditional_node!(
    pub(super) RegionOutputReadyNode in RegionOutputDomain, RegionOutputReadinessOperation,
    RegionOutputReadinessFamily => operation "region-output-ready"
);

pub(super) struct Readiness;

impl Readiness {
    fn conditional_binding() -> domain::WorthQueryApplicationConditionalOperationBinding<
        Schema,
        dependent_publication::PublishDependent,
        dependent_publication::Input,
        RegionOutputDomain,
        RegionOutputReadinessOperation,
        RegionOutputReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            dependent_publication::PublishDependent::reference(),
            operation_definition().reference(),
        )
    }
}

impl application_contribution::WorthQueryApplicationConditionalBinding<Schema> for Readiness {
    type Configuration = ();
    type Installed = ();
    type Operation = dependent_publication::PublishDependent;
    const IDENTITY: &'static str = "courtroom-equal-output-readiness";
    const REQUIRED_PRODUCERS: &'static [&'static str] = &["courtroom-counted-dependent-producer"];

    fn package_contract(
    ) -> application_contribution::WorthQueryApplicationConditionalPackageContract {
        application_contribution::WorthQueryApplicationConditionalPackageContract::new(
            operation_definition().into_portable(),
            Self::conditional_binding().portable().clone(),
            RegionOutputReadyNode::reference().node_identity(),
        )
    }

    fn install(
        _: (),
        _: &application_contribution::WorthQueryApplicationConditionalProducerAccess<'_, Schema>,
        installation: &mut WorthQueryConditionalApplicationRuntimeInstallation<Schema>,
    ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
        let operation = installation
            .installed_schema()
            .installed_operation(dependent_publication::PublishDependent::reference())
            .unwrap();
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &Self::conditional_binding())
            .unwrap()
            .bind_node(RegionOutputReadyNode::reference())
            .unwrap();
        installation.bind_output_readiness::<counted_producer::Producer, _, _, _, _, _, _>(node, 0)
    }
}

fn operation_definition() -> domain::WorthQueryDomainOperationDefinition<
    RegionOutputDomain,
    RegionOutputReadinessOperation,
    RegionOutputReadinessFamily,
> {
    let dependency = crate::readiness::output_change_dependency();
    application_contribution::WorthQueryOutputReadinessContractBuilder::new(
        domain::WorthQueryDomainOperationIdentity::new("region-output-readiness", 1),
        "region-output-ready",
        crate::readiness::output_change_projection(),
        crate::readiness::canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("anchor").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("region-output-readiness").unwrap(),
        128,
        128,
        "region-output-readiness-v1",
    )
    .semantic_reads([crate::readiness::output_change_projection()])
    .dependencies([dependency.clone()])
    .readiness_dependencies([dependency])
    .build()
    .expect("the region output's readiness declaration is canonical")
}
