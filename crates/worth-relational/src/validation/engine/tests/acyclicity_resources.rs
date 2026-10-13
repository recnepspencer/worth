//! Real transaction validation under one finite caller request.
mod fixture;
mod graph_semantics;
mod interruptions;
mod packet_capacity;

use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::transactions::data::{CommitExecutionDenialKind, TransactionCommitError};
use fixture::*;
use worth_execution::{CancellationToken, ExecutionRequest};

#[test]
fn repeated_dag_reachability_reuses_scratch_under_the_same_finite_policy() {
    let fixture = DagFixture::new(128);
    let before = fixture.head();
    // Deliberate ordinary finite policy, independent of an observed operation
    // total. The old per-visit allocation accumulator exceeds this allowance.
    let lease = request(1024 * 1024, 4_000_000, CancellationToken::new(), None);
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
        .commit_boundary
        .results()
        .iter()
        .all(|result| matches!(
            result.verdict,
            crate::validation::data::InvariantVerdict::Pass
                | crate::validation::data::InvariantVerdict::NotApplicable
        )));
    assert_eq!(fixture.head(), before, "validation grants no publication");
    drop(candidate);
    assert_eq!(lease.charged_memory_bytes(), 0);
}

#[test]
fn unfunded_native_validation_preserves_the_branch_and_releases_request_memory() {
    for (bytes, work) in [(0, 4_000_000), (1024 * 1024, 0)] {
        let fixture = DagFixture::new(8);
        let before = fixture.head();
        let lease = request(bytes, work, CancellationToken::new(), None);
        let failure = fixture
            .validate(&ExecutionRequest::leased(&lease), fixture.chain_batch(2))
            .expect_err("unfunded validation cannot issue candidate authority");
        match failure {
            TransactionCommitError::Execution { denial, .. } => {
                assert!(
                    matches!(
                        denial.kind,
                        CommitExecutionDenialKind::Cause(
                            Cause::WorkExhausted
                                | Cause::PolicyMemoryExhausted { .. }
                                | Cause::DeclaredMemoryExhausted { .. }
                        )
                    ),
                    "{denial:?}"
                );
            }
            other => panic!("expected native execution refusal, got {other:?}"),
        }
        assert_eq!(fixture.head(), before);
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
}

#[test]
fn late_visible_plus_planned_cycle_retains_the_actual_planned_edge() {
    let fixture = DagFixture::new(12);
    let setup = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&setup);
    fixture.publish(&execution, fixture.chain_batch(1)).unwrap();
    let before = fixture.head();
    let source = *fixture.nodes.last().unwrap();
    let target = fixture.nodes[0];
    let lease = request(1024 * 1024, 4_000_000, CancellationToken::new(), None);
    let failure = fixture
        .validate(
            &ExecutionRequest::leased(&lease),
            fixture.edge_batch(source, target, "close-late-cycle"),
        )
        .expect_err("the last-to-first edge closes the real committed chain");
    match failure {
        TransactionCommitError::Conflict { error, .. } => match error.class {
            crate::transactions::data::ConflictClass::InvariantViolation {
                fields:
                    crate::validation::data::InvariantViolationFields::Acyclicity {
                        contract_id,
                        source: actual_source,
                        target: actual_target,
                        ..
                    },
                ..
            } => {
                assert_eq!(contract_id.as_str(), "no_cycles");
                assert_eq!(
                    actual_source,
                    crate::transactions::data::EntityReference::Existing(source)
                );
                assert_eq!(
                    actual_target,
                    crate::transactions::data::EntityReference::Existing(target)
                );
            }
            other => panic!("expected precise cycle diagnostic, got {other:?}"),
        },
        other => panic!("expected actual cycle refusal, got {other:?}"),
    }
    assert_eq!(fixture.head(), before);
    assert_eq!(lease.charged_memory_bytes(), 0);
}
