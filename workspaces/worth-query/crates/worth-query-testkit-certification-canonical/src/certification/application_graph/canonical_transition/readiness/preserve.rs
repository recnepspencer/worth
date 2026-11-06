//! Conditional readiness bound to the real existing-family producer.
pub use super::super::producer::preserve::{Input, Preserve, Producer};
pub use super::*;

worth_query_host::facade::worth_query_conditional_node!(
    pub PreserveReadyNode in DerivedReadinessDomain,
    DerivedReadinessOperation,
    DerivedReadinessFamily => operation "canonical-derived-preserve-ready"
);

pub struct PreserveReadiness;
impl PreserveReadiness {
    pub fn conditional_binding() -> domain::WorthQueryApplicationConditionalOperationBinding<
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
        _declaration: &worth_query_host::facade::declaration::application_schema::ErasedApplicationSchemaDeclaration,
    ) -> Result<
        application_contribution::WorthQueryApplicationConditionalPackageContract,
        worth_query_host::facade::application_contribution::WorthQueryOutputReadinessContractDenial,
    > {
        let contract = readiness_contract(_declaration)?;
        let binding = Self::conditional_binding();
        Ok(contract.into_package_contract(binding.portable().clone()))
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
        let contract =
            readiness_contract(installation.installed_schema().installed_declaration()).unwrap();
        let ordinal = contract
            .delivery_ordinal(
                installation.installed_schema(),
                node.declaration(),
                DerivedValueField::at("DerivedValueField"),
            )
            .unwrap();
        installation.bind_output_readiness::<Producer, _, _, _, _, _, _>(node, ordinal)
    }
}
pub fn readiness_contract(
    declaration: &worth_query_host::facade::declaration::application_schema::ErasedApplicationSchemaDeclaration,
) -> Result<Contract, application_contribution::WorthQueryOutputReadinessContractDenial> {
    application_contribution::WorthQueryOutputReadinessContractBuilder::for_field(
        declaration,
        DerivedValueField::at("DerivedValueField"),
        domain::WorthQueryDomainOperationIdentity::new("canonical-derived-preserve-readiness", 1),
        "canonical-derived-preserve-ready",
        canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("published-output").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("canonical-derived-preserve-readiness")
            .unwrap(),
        "canonical-derived-preserve-readiness-v1",
    )?
    .field_dependency(
        declaration,
        DerivedValueField::at("DerivedValueField"),
        application_contribution::WorthQueryReadinessFieldUse::DeliveryAndEligibility,
    )?
    .build_typed()
}
pub fn operation_definition() -> domain::WorthQueryDomainOperationDefinition<
    DerivedReadinessDomain,
    DerivedReadinessOperation,
    DerivedReadinessFamily,
> {
    readiness_contract(CanonicalSchema::declaration().unwrap().erased())
        .unwrap()
        .definition()
        .clone()
}
