use worth_signal::facade::core::{
    AsyncNodeCapabilityDeclaration, AsyncNodePayloadContract, AsyncNodePayloadContractId,
    SignalGraph, ValidatedAsyncNodeCapabilityDeclaration, ValidatedResourcePolicyDeclaration,
};

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let declaration = AsyncNodeCapabilityDeclaration::new(
        node,
        AsyncNodePayloadContract::new(AsyncNodePayloadContractId::new(7)),
    );

    let _validated = ValidatedAsyncNodeCapabilityDeclaration {
        declaration,
        validated: fake::<ValidatedResourcePolicyDeclaration>(),
    };
}
