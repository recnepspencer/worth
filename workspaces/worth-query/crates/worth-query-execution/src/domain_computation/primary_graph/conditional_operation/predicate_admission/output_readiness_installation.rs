use std::sync::Arc;

use worth_query_installation::facade::{
    WorthQueryConditionalConditionClass, WorthQueryInstalledApplicationConditionalNode,
    WorthQueryInstalledGraphParticipationAuthority,
};
use worth_runtime_bridge::facade::{
    BridgeConditionalProviderSet, BridgeOwnedConditionalInstallationRequest,
};

use super::{
    bridge_denial, QueryConditionalComputeProvider, QueryConditionalComputeSemanticContract,
    QueryOutputReadinessPredicate, WorthQueryConditionalRuntimeInstallationDenial,
};

pub(in crate::domain_computation::primary_graph) fn prepare_dependency_conditional_installation<
    Schema,
    ApplicationOperation,
    Input,
    D,
    O,
    F,
    Node,
>(
    node: &WorthQueryInstalledApplicationConditionalNode<
        Schema,
        ApplicationOperation,
        Input,
        D,
        O,
        F,
        Node,
    >,
    graph: &WorthQueryInstalledGraphParticipationAuthority,
) -> Result<BridgeOwnedConditionalInstallationRequest, WorthQueryConditionalRuntimeInstallationDenial>
where
    Node: 'static,
{
    if node.declaration().condition().class() != WorthQueryConditionalConditionClass::AspectFiltered
    {
        return Err(bridge_denial(
            "output readiness requires an aspect-filtered conditional node",
        ));
    }
    let contract = super::lower_dependency_contract(node.declaration())?;
    let dependencies = super::dependency_candidates(node, graph)?;
    let node_authority: Arc<str> = Arc::from(node.authority_identity());
    Ok(BridgeOwnedConditionalInstallationRequest {
        contract,
        location: super::lower_location(node.location()),
        dependencies,
        providers: BridgeConditionalProviderSet::new()
            .condition(QueryOutputReadinessPredicate::new(
                Arc::clone(&node_authority),
                node.declaration().condition().dependencies().len(),
            ))
            .compute(QueryConditionalComputeProvider::<Node> {
                semantics: QueryConditionalComputeSemanticContract(node_authority),
                output_version: None,
            }),
    })
}
