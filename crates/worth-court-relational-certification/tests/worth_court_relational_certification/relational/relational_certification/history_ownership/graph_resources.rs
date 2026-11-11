//! The graph phase keeps its own result funding and spends the original meter.
use super::*;
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::branch::RelationalOwnedModelSelectionDenial as GraphDenial;

#[test]
fn graph_relation_preparation_retains_its_funding_and_cannot_reset_the_entity_meter() {
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
    let memory = SerialMemoryBudget::new(None);
    let metered = SerialRequest::from_memory(memory.clone(), CancellationToken::new(), None);
    let request = &ExecutionRequest::serial(&metered);
    let entities = graph
        .runtime
        .prepare_owned_model_entity_boundary(request, &basis, &basis, &ownership(), graph.model)
        .unwrap();
    let entity_bytes = memory.charged_memory_bytes();
    drop(entities);
    assert_eq!(memory.charged_memory_bytes(), 0);
    let selected = graph
        .runtime
        .prepare_owned_model_graph_selection(
            request,
            &basis,
            &basis,
            &ownership(),
            &ownership(),
            graph.model,
        )
        .unwrap();
    assert_eq!(selected.current_relations().len(), 2);
    assert_eq!(selected.selected_relations().len(), 2);
    assert!(
        memory.charged_memory_bytes() > entity_bytes,
        "relation results retain their original backing beyond the computation scope"
    );
    drop(selected);
    assert_eq!(memory.charged_memory_bytes(), 0);
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Automatic,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::new(1).unwrap(), None, Some(64)),
    );
    let bounded = SerialRequest::from_policy(&policy, CancellationToken::new(), None);
    let request = &ExecutionRequest::serial(&bounded);
    drop(
        graph
            .runtime
            .prepare_owned_model_entity_boundary(request, &basis, &basis, &ownership(), graph.model)
            .unwrap(),
    );
    assert!(matches!(
        graph.runtime.prepare_owned_model_graph_selection(
            request,
            &basis,
            &basis,
            &ownership(),
            &ownership(),
            graph.model
        ),
        Err(GraphDenial::Ownership(Denial::Request(
            worth_execution::WorkCeilingDenial::Stopped(
                worth_execution::MapKernelStop::WorkCeiling
            )
        )))
    ));
    assert_eq!(
        graph
            .runtime
            .observe_branch(&graph.runtime.main_branch_identity())
            .unwrap()
            .0,
        basis.descriptor().clone()
    );
}
