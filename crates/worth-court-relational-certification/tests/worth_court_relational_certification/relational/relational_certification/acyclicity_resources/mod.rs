//! Native cycle detection and resource denials through the public transaction path.
mod fixture;
mod graph_semantics;
mod interruptions;
mod native_prerequisites;

pub(super) use fixture::authority as request_authority;
use fixture::*;
use worth_execution::{CancellationToken, ExecutionRequest};
use worth_relational::facade::transactions::CommitExecutionDenialKind;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
use worth_relational::facade::transactions::TransactionCommitError;

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
    let setup = serial_request();
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
            worth_relational::facade::transactions::ConflictClass::InvariantViolation {
                fields:
                    worth_relational::facade::transactions::InvariantViolationFields::Acyclicity {
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
                    worth_relational::facade::transactions::EntityReference::Existing(source)
                );
                assert_eq!(
                    actual_target,
                    worth_relational::facade::transactions::EntityReference::Existing(target)
                );
            }
            other => panic!("expected precise cycle diagnostic, got {other:?}"),
        },
        other => panic!("expected actual cycle refusal, got {other:?}"),
    }
    assert_eq!(fixture.head(), before);
    assert_eq!(lease.charged_memory_bytes(), 0);
}
