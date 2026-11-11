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
    fn conditional_binding(
        definition: &domain::WorthQueryDomainOperationDefinition<
            RegionOutputDomain,
            RegionOutputReadinessOperation,
            RegionOutputReadinessFamily,
        >,
    ) -> domain::WorthQueryApplicationConditionalOperationBinding<
        Schema,
        dependent_publication::PublishDependent,
        dependent_publication::Input,
        RegionOutputDomain,
        RegionOutputReadinessOperation,
        RegionOutputReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            dependent_publication::PublishDependent::reference(),
            definition.reference(),
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
        declaration: &worth_query_host::facade::declaration::application_schema::ErasedApplicationSchemaDeclaration,
    ) -> Result<
        application_contribution::WorthQueryApplicationConditionalPackageContract,
        worth_query_host::facade::application_contribution::WorthQueryOutputReadinessContractDenial,
    > {
        let contract = operation_contract(declaration)?;
        let binding = Self::conditional_binding(contract.definition());
        Ok(contract.into_package_contract(binding.portable().clone()))
    }

    fn install(
        _: (),
        _: &application_contribution::WorthQueryApplicationConditionalProducerAccess<'_, Schema>,
        installation: &mut WorthQueryConditionalApplicationRuntimeInstallation<Schema>,
    ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
        let contract = operation_contract(installation.installed_schema().installed_declaration())?;
        let binding = Self::conditional_binding(contract.definition());

        let operation = installation
            .installed_schema()
            .installed_operation(dependent_publication::PublishDependent::reference())
            .unwrap();
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &binding)
            .unwrap()
            .bind_node(RegionOutputReadyNode::reference())
            .unwrap();
        installation.bind_output_readiness::<counted_producer::Producer, _, _, _, _, _, _>(node, 0)
    }
}

fn operation_contract(
    declaration: &worth_query_decl::facade::application_schema::ErasedApplicationSchemaDeclaration,
) -> Result<
    application_contribution::WorthQueryOutputReadinessContract<
        RegionOutputDomain,
        RegionOutputReadinessOperation,
        RegionOutputReadinessFamily,
    >,
    application_contribution::WorthQueryOutputReadinessContractDenial,
> {
    application_contribution::WorthQueryOutputReadinessContractBuilder::for_field(
        declaration,
        crate::readiness::delivery_field(),
        domain::WorthQueryDomainOperationIdentity::new("region-output-readiness", 1),
        "region-output-ready",
        crate::readiness::canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("anchor").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("region-output-readiness").unwrap(),
        "region-output-readiness-v1",
    )?
    .field_dependency(
        declaration,
        crate::readiness::delivery_field(),
        application_contribution::WorthQueryReadinessFieldUse::DeliveryAndEligibility,
    )?
    .build_typed()
}
