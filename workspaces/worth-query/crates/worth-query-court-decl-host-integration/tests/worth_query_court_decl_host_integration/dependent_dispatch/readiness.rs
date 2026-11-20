//! Every fixture producer has genuine installed field-based readiness.
use super::schema::Facts;
use worth_query_decl::facade::{
    authoring::*, binding::QueryBindingDescriptor, canonicalization::*,
};
pub fn canonical_query() -> CanonicalQueryBundle {
    let query = DetailQueryBuilder::new(RootEntityKey::new("Node").unwrap())
        .project(AspectFieldSelector::new(Facts::reference().name(), "Value").unwrap())
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(AuthoredResultShapeField::new(Facts::reference().name(), "Value", "value").unwrap())
        .build()
        .unwrap()
        .into_raw();
    canonicalize_request(
        AuthoredQueryBundleRequest::for_ordinary_read(query, shape, QueryBindingDescriptor::new())
            .unwrap(),
    )
    .unwrap()
}
macro_rules! readiness {
    ($identity:literal) => { readiness!($identity, Value, crate::dependent_dispatch::readiness::canonical_query, "node"); };
    ($identity:literal, $field:ident, $query:path, $role:literal) => {
        use worth_query_host::facade::domain;
        #[derive(Clone, Copy, Debug, Eq, PartialEq)] pub struct ReadyDomain;
        #[derive(Clone, Copy, Debug, Eq, PartialEq)] pub struct ReadyOperation;
        #[derive(Clone, Copy, Debug, Eq, PartialEq)] pub struct ReadyFamily;
        worth_query_host::facade::worth_query_conditional_node!(pub ReadyNode in ReadyDomain, ReadyOperation, ReadyFamily => operation $identity);
        pub struct Readiness;
        type Contract = WorthQueryOutputReadinessContract<ReadyDomain, ReadyOperation, ReadyFamily>;
        fn readiness_contract(declaration: &ErasedApplicationSchemaDeclaration) -> Result<Contract, WorthQueryOutputReadinessContractDenial> {
            WorthQueryOutputReadinessContractBuilder::for_field(declaration, $field::at(stringify!($field)),
                domain::WorthQueryDomainOperationIdentity::new($identity, 1), $identity,
                $query(),
                domain::WorthQueryOperationProjectionRole::new($role).unwrap(),
                domain::WorthQueryExecutionStrategyName::new($identity).unwrap(), $identity)?
                .field_dependency(declaration, $field::at(stringify!($field)), WorthQueryReadinessFieldUse::DeliveryAndEligibility)?.build_typed()
        }
        fn conditional_binding(contract: &Contract) -> domain::WorthQueryApplicationConditionalOperationBinding<Schema, Operation, Input, ReadyDomain, ReadyOperation, ReadyFamily> {
            domain::WorthQueryApplicationConditionalOperationBinding::declare(Operation::reference(), contract.definition().reference())
        }
        impl WorthQueryApplicationConditionalBinding<Schema> for Readiness {
            type Configuration = (); type Installed = (); type Operation = Operation;
            const IDENTITY: &'static str = $identity;
            const REQUIRED_PRODUCERS: &'static [&'static str] = &[$identity];
            fn package_contract(declaration: &ErasedApplicationSchemaDeclaration) -> Result<WorthQueryApplicationConditionalPackageContract, WorthQueryOutputReadinessContractDenial> {
                let contract = readiness_contract(declaration)?;
                let binding = conditional_binding(&contract);
                Ok(contract.into_package_contract(binding.portable().clone()))
            }
            fn install(_: (), _: &WorthQueryApplicationConditionalProducerAccess<'_, Schema>, installation: &mut WorthQueryConditionalApplicationRuntimeInstallation<Schema>) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
                let contract = readiness_contract(Schema::declaration().unwrap().erased()).unwrap();
                let binding = conditional_binding(&contract);
                let operation = installation.installed_schema().installed_operation(Operation::reference()).unwrap();
                let node = installation.installed_packages().bind_conditional_application_operation(operation, &binding).unwrap().bind_node(ReadyNode::reference()).unwrap();
                let ordinal = contract.delivery_ordinal(installation.installed_schema(), node.declaration(), $field::at(stringify!($field))).unwrap();
                installation.bind_output_readiness::<Producer, _, _, _, _, _, _>(node, ordinal)
            }
        }
    };
}
