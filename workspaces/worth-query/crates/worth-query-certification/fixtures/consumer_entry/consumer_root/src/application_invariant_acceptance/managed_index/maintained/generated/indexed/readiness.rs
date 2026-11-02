//! Fresh runtime readiness route reuses the exact portable Generate contract.
use super::super::super::schema::*;
use super::super::{
    operation::GenerateItem,
    readiness::{binding, contract, ReadyNode},
};
use super::producer::Producer;
use worth_query_host::facade::{application_contribution::*, declaration::application_schema::*};
pub struct Readiness;
impl WorthQueryApplicationConditionalBinding<Catalog> for Readiness {
    type Configuration = ();
    type Installed = ();
    type Operation = GenerateItem;
    const IDENTITY: &'static str = "maintained-generated.indexed.readiness.v1";
    const REQUIRED_PRODUCERS: &'static [&'static str] =
        &[<Producer as WorthQueryApplicationProducerBinding<Catalog>>::IDENTITY];
    fn package_contract(
        declaration: &ErasedApplicationSchemaDeclaration,
    ) -> Result<
        WorthQueryApplicationConditionalPackageContract,
        WorthQueryOutputReadinessContractDenial,
    > {
        let contract = contract(declaration)?;
        let binding = binding(&contract);
        Ok(contract.into_package_contract(binding.portable().clone()))
    }
    fn install(
        _: (),
        _: &WorthQueryApplicationConditionalProducerAccess<'_, Catalog>,
        installation: &mut worth_query_host::facade::primary_graph::WorthQueryConditionalApplicationRuntimeInstallation<Catalog>,
    ) -> Result<
        (),
        worth_query_host::facade::primary_graph::WorthQueryConditionalRuntimeInstallationDenial,
    > {
        let contract = contract(installation.installed_schema().installed_declaration()).unwrap();
        let operation = installation
            .installed_schema()
            .installed_operation(GenerateItem::reference())
            .unwrap();
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &binding(&contract))
            .unwrap()
            .bind_node(ReadyNode::reference())
            .unwrap();
        let ordinal = contract
            .delivery_ordinal(
                installation.installed_schema(),
                node.declaration(),
                ItemName::reference(),
            )
            .unwrap();
        installation.bind_output_readiness::<Producer, _, _, _, _, _, _>(node, ordinal)
    }
}
