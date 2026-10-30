use worth_signal::facade::core::{
    ResourceNodeDeclaration, ResourceNodeId, ResourcePayloadContract, ResourcePayloadContractId,
    SignalGraph, ValidatedResourcePolicyDeclaration, ValidatedResourcePolicyReference,
};

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let declaration = ResourceNodeDeclaration::new(
        ResourceNodeId::from_node(node),
        ResourcePayloadContract::new(ResourcePayloadContractId::new(7)),
    );

    let _validated = ValidatedResourcePolicyDeclaration {
        declaration,
        retry: fake::<ValidatedResourcePolicyReference>(),
        timeout: fake::<ValidatedResourcePolicyReference>(),
        cancellation: fake::<ValidatedResourcePolicyReference>(),
        stale_after: fake::<ValidatedResourcePolicyReference>(),
        supersession: fake::<ValidatedResourcePolicyReference>(),
        revalidation: fake::<ValidatedResourcePolicyReference>(),
        observation: fake::<ValidatedResourcePolicyReference>(),
        output_continuity: fake::<ValidatedResourcePolicyReference>(),
        retention: fake::<ValidatedResourcePolicyReference>(),
        diagnostics: fake::<ValidatedResourcePolicyReference>(),
        registry_digest: fake(),
    };
}
