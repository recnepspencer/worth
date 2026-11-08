//! Independent ordinary memory owner construction and first-demand reference.
use super::*;
pub(super) fn value(execution: &ExecutionRequest<'_, '_>) -> AspectVersion {
    let (mut memory, node) = program();
    let branch = memory.current_branch();
    let basis = memory.observe_signal_branch_basis(branch).unwrap();
    let mut result = None;
    memory
        .advance_signal_branch(*execution, &mut (), &basis, |tx| {
            result = Some(tx.target(node).read(
                &|_: &mut EvaluationContext<'_, ()>| -> Result<AspectVersion, SignalError> {
                    Ok(AspectVersion::from_updates([(Aspect::new(0), 71)]))
                },
            )?);
            Ok(())
        })
        .unwrap();
    result.unwrap()
}
fn program() -> (Box<owner_fixture::Runtime>, NodeId) {
    let mut graph = Box::new(SignalGraph::new());
    let node = graph.node().on_demand().build();
    let memory: Box<owner_fixture::Runtime> = Box::new(
        SignalRuntime::builder_from_boxed_graph(graph)
            .with_kernel_defaults()
            .build(),
    );
    (memory, node)
}
