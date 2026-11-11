//! The graph phase keeps its own result funding and spends the original meter.
use super::*;

#[test]
fn graph_selection_preserves_same_basis_relations_without_publication() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let graph = graph(execution);
    let (_, basis) = graph
        .runtime
        .observe_branch(&graph.runtime.main_branch_identity())
        .unwrap();
    assert_eq!(
        graph
            .runtime
            .observe_model_owner(execution, &basis, &ownership(), graph.nested)
            .unwrap(),
        Some(graph.model)
    );
    let selected = graph
        .runtime
        .prepare_owned_model_graph_selection(
            execution,
            &basis,
            &basis,
            &ownership(),
            &ownership(),
            graph.model,
        )
        .unwrap();
    assert_eq!(selected.current_relations().len(), 2);
    assert_eq!(selected.selected_relations().len(), 2);
    assert_eq!(
        graph
            .runtime
            .observe_branch(&graph.runtime.main_branch_identity())
            .unwrap()
            .0,
        basis.descriptor().clone()
    );
}
