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
    fn conditional_binding(
        definition: &domain::WorthQueryDomainOperationDefinition<
            RootDomain,
            RootReadinessOperation,
            RootReadinessFamily,
        >,
    ) -> domain::WorthQueryApplicationConditionalOperationBinding<
        Schema,
        publication::PublishRoot,
        publication::Input,
        RootDomain,
        RootReadinessOperation,
        RootReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            publication::PublishRoot::reference::<Schema>(),
            definition.reference(),
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
            .installed_operation(publication::PublishRoot::reference::<Schema>())
            .unwrap();
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &binding)
            .unwrap()
            .bind_node(RootReadyNode::reference())
            .unwrap();
        installation.bind_output_readiness::<Producer, _, _, _, _, _, _>(node, 0)
    }
}

fn operation_contract(
    declaration: &worth_query_decl::facade::application_schema::ErasedApplicationSchemaDeclaration,
) -> Result<
    application_contribution::WorthQueryOutputReadinessContract<
        RootDomain,
        RootReadinessOperation,
        RootReadinessFamily,
    >,
    application_contribution::WorthQueryOutputReadinessContractDenial,
> {
    application_contribution::WorthQueryOutputReadinessContractBuilder::for_field(
        declaration,
        crate::readiness::delivery_field(),
        domain::WorthQueryDomainOperationIdentity::new("courtroom-equal-root-readiness", 1),
        "courtroom-equal-root-ready",
        crate::readiness::canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("anchor").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("courtroom-equal-root-readiness").unwrap(),
        "courtroom-equal-root-readiness-v1",
    )?
    .field_dependency(
        declaration,
        crate::readiness::delivery_field(),
        application_contribution::WorthQueryReadinessFieldUse::DeliveryAndEligibility,
    )?
    .build_typed()
}
