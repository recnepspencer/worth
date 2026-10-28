//! A successful native prefix preserves the real subsequent preparation cause.
use super::*;
use crate::validation::data::{CustomInvariantFailurePhase, InvariantVerdict};
use worth_execution::CancellationSource;

#[test]
fn native_pass_preserves_custom_preparation_refusal_and_original_cancellation() {
    for cancel in [false, true] {
        let (fixture, hooks) = fixture(2);
        let before = fixture.head();
        let cancellation = CancellationSource::new();
        *hooks.preparation.lock().unwrap() = if cancel {
            Preparation::Cancel(cancellation.clone())
        } else {
            Preparation::Refuse
        };
        let plan = fixture.merged_plan(fixture.chain_batch(1));
        let request = InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &fixture.runtime,
            InvariantObservation::committed(fixture.runtime.storage_access().current_edition()),
            fixture.runtime.current_version_id(),
            Some(&plan),
            None,
        );
        let lease =
            super::super::fixture::request(8 * 1024 * 1024, 4_000_000, cancellation.token(), None);
        let outcome = InvariantEngine::new(&fixture.runtime)
            .execute(&ExecutionRequest::leased(&lease), request);
        if cancel {
            let Err(TransactionCommitError::Execution { denial, .. }) = outcome else {
                panic!("original cancellation required");
            };
            assert_eq!(
                denial.kind,
                crate::transactions::data::CommitExecutionDenialKind::Cause(
                    crate::execution::RelationalExecutionDenialCause::Cancelled,
                )
            );
        } else {
            let result = outcome.unwrap();
            assert_eq!(result.results().len(), 2);
            assert!(matches!(
                result.results()[0].verdict,
                InvariantVerdict::Pass
            ));
            let InvariantVerdict::Violation(violation) = &result.results()[1].verdict else {
                panic!("preparation failure must remain a violation");
            };
            assert!(
                matches!(&violation.fields, InvariantViolationFields::CustomInvariantFailure {
                identity, phase: CustomInvariantFailurePhase::Preparation, ..
            } if identity.semantic_identity().rule_id.as_str() == "test.native-prerequisite.scope")
            );
            assert_eq!(result.decision_log().len(), 2);
        }
        assert_eq!(hooks.observed(), (1, 0));
        assert_eq!(fixture.head(), before);
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
}
