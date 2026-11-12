//! Model ownership through real Native cold recovery and request boundaries.
mod fixture;
use fixture::*;
mod entity_boundary;
mod graph_admission;
mod graph_boundary;
mod graph_integrity;
mod graph_persistence;
mod graph_publication;
mod graph_refusals;
mod graph_resources;
mod historical_absence;
mod historical_foreign;
mod historical_publication;
mod historical_read_chain;
mod historical_read_evidence;
mod historical_source_ancestry;
mod historical_source_presence;
mod historical_worker;
mod ownership_transfer;
mod ownership_transfer_refusals;
mod prospective_publication;
mod publication_fixture;
mod publication_membership;
mod read_evidence_fixture;
mod transfer_fixture;
mod transfer_publication_replacements;
mod transfer_publications;

#[test]
fn cold_model_ownership_demands_cross_partition_endpoints_and_edges() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let graph = graph(execution);
    let (_, basis) = graph
        .runtime
        .observe_branch(&graph.runtime.main_branch_identity())
        .unwrap();
    let before = basis.descriptor().clone();
    assert_eq!(
        graph
            .runtime
            .observe_model_owner(execution, &basis, &ownership(), graph.nested)
            .unwrap(),
        Some(graph.model),
    );
    assert!(graph.source.region_read_count() > 0);
    assert_eq!(
        graph
            .runtime
            .observe_branch(&graph.runtime.main_branch_identity())
            .unwrap()
            .0,
        before
    );
}

#[test]
fn ownership_observation_refuses_a_basis_from_another_runtime() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let graph = graph(execution);
    let before_reads = graph.source.reads.load(Ordering::Relaxed);
    let foreign = runtime();
    let (_, basis) = graph
        .runtime
        .observe_branch(&graph.runtime.main_branch_identity())
        .unwrap();
    assert!(matches!(
        foreign.observe_model_owner(execution, &basis, &ownership(), graph.nested),
        Err(Denial::ForeignBasis)
    ));
    assert_eq!(
        graph.source.reads.load(Ordering::Relaxed),
        before_reads,
        "foreign admission must precede material access"
    );
}

mod historical_progressed_source;

mod current_publications;
mod current_successor;

mod publication_appendix;
