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
mod historical_artifact;
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
    assert!(graph.source.reads.load(Ordering::Relaxed) > 0);
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
fn missing_cold_ownership_edge_material_refuses_with_its_real_partition() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let graph = graph(execution);
    let (_, basis) = graph
        .runtime
        .observe_branch(&graph.runtime.main_branch_identity())
        .unwrap();
    // Warm only endpoint partitions through the real transaction read door.
    let mut reader = begin(&graph.runtime);
    assert!(reader
        .read_entity(execution, graph.model)
        .unwrap()
        .effective()
        .is_some());
    assert!(reader
        .read_entity(execution, graph.nested)
        .unwrap()
        .effective()
        .is_some());
    // Lose each remaining region member at the storage port. The required cold
    // relation partition is distinct from both already admitted endpoint regions.
    let read = graph.source.member_reads.lock().unwrap().clone();
    for (&locator, bytes) in &graph.source.members {
        if bytes.starts_with(b"WORTH-REL-REG") && !read.contains(&locator) {
            graph
                .source
                .failures
                .lock()
                .unwrap()
                .insert(locator, Cause::MissingMember);
        }
    }
    let denial = graph
        .runtime
        .observe_model_owner(execution, &basis, &ownership(), graph.nested)
        .unwrap_err();
    let Denial::Native(RelationalExecutionDenialCause::StorageUnavailable(failure)) = denial else {
        panic!("missing ownership material must not become absence: {denial:?}")
    };
    assert_eq!(failure.cause, Cause::MissingMember);
    assert_eq!(
        failure.subject,
        RelationalStorageSubject::Region(PartitionId::main())
    );
    assert_eq!(
        graph
            .runtime
            .observe_branch(&graph.runtime.main_branch_identity())
            .unwrap()
            .0,
        basis.descriptor().clone()
    );
}

#[test]
fn ownership_observation_preserves_work_and_memory_refusals() {
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let graph = graph(execution);
    let (_, basis) = graph
        .runtime
        .observe_branch(&graph.runtime.main_branch_identity())
        .unwrap();
    // Warm the actual serving view so the narrow request tests traversal cost.
    assert_eq!(
        graph
            .runtime
            .observe_model_owner(execution, &basis, &ownership(), graph.nested)
            .unwrap(),
        Some(graph.model)
    );
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Automatic,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::new(1).unwrap(), None, Some(1)),
    );
    let bounded = SerialRequest::from_policy(&policy, CancellationToken::new(), None);
    let denial = graph
        .runtime
        .observe_model_owner(
            &ExecutionRequest::serial(&bounded),
            &basis,
            &ownership(),
            graph.nested,
        )
        .unwrap_err();
    assert!(
        matches!(
            denial,
            Denial::Request(worth_execution::WorkCeilingDenial::Stopped(
                worth_execution::MapKernelStop::WorkCeiling
            ))
        ),
        "{denial:?}"
    );
    let bounded = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(0)),
        CancellationToken::new(),
        None,
    );
    let denial = graph
        .runtime
        .observe_model_owner(
            &ExecutionRequest::serial(&bounded),
            &basis,
            &ownership(),
            graph.nested,
        )
        .unwrap_err();
    assert!(matches!(denial, Denial::Request(_)), "{denial:?}");
    assert_eq!(
        graph
            .runtime
            .observe_branch(&graph.runtime.main_branch_identity())
            .unwrap()
            .0,
        basis.descriptor().clone()
    );
}

#[test]
fn ownership_observation_refuses_a_basis_from_another_runtime() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let graph = graph(execution);
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
        0,
        "foreign admission must precede material access"
    );
}

mod historical_progressed_source;

mod current_publications;
