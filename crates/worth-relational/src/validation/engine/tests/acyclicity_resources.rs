//! Real transaction validation under one finite caller request.
mod fixture;
mod native_prerequisites;
pub(crate) use worth_relational_mutation::internal_port::validation::engine::tests::acyclicity_resources::packet_capacity;

use fixture::DagFixture;
use packet_capacity::paid_dag_request_for_test;
use worth_execution::{CancellationToken, ExecutionRequest};

#[test]
fn repeated_dag_reachability_reuses_scratch_under_the_same_finite_policy() {
    let fixture = DagFixture::new(512);
    let before = fixture.head();
    // Deliberate ordinary finite policy, independent of an observed operation
    // total. Validation retains original-request funding that grows linearly
    // with the graph (the proposal and its relation-source witness, about
    // 8 KiB per node: 4 MiB here). The old per-visit allocation accumulator
    // grows with the square of the node count: past 1 MiB at 128 nodes, so
    // past 16 MiB here. 8 MiB funds the linear law and refuses the quadratic.
    // Work is fixture funding here, not the claim.
    let lease = paid_dag_request_for_test(
        worth_harness::facade::execution_fixture::test_execution_authority(),
        8 * 1024 * 1024,
        400_000_000,
        CancellationToken::new(),
        None,
    );
    let candidate = fixture
        .validate(&ExecutionRequest::leased(&lease), fixture.chain_batch(2))
        .expect("a DAG needs live scratch, not the sum of every freed successor buffer");
    let summary = candidate.validation_summary();
    assert!(
        summary.commit_boundary_seen
            && summary.mutation_sensitive_seen
            && summary.snapshot_publication_seen
    );
    assert!(candidate
        .state
        .invariants
        .results()
        .iter()
        .all(|result| matches!(
            result.verdict,
            crate::validation::data::execution::InvariantVerdict::Pass
                | crate::validation::data::execution::InvariantVerdict::NotApplicable
        )));
    assert_eq!(fixture.head(), before, "validation grants no publication");
    drop(candidate);
    assert_eq!(lease.charged_memory_bytes(), 0);
}
