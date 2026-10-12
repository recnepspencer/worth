//! Admission-owned access to actual retained observations for cancellation proofs.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenialKind as Kind,
    WorthQueryPrimaryGraphApplicationDecisionFact as Fact,
};
use worth_execution::{CancellationSource, ExecutionRequest};
use worth_relational::facade::{runtime::RelationalRuntime, snapshots::SnapshotHandle};
impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation) fn assert_cancelled_fact_dispatch(
        &self,
        runtime: &RelationalRuntime,
        execution: &ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
        source: &CancellationSource,
    ) {
        let Some(WorthQueryRetainedAuthorizationDecisionFacts::Abilities {
            principal,
            decisions,
        }) = &self.authorization
        else {
            panic!("ViewAccount must retain conventional observations");
        };
        assert_eq!(decisions.len(), 1);
        let principal = Fact::principal(principal.clone());
        let authorization = Fact::authorization(0, decisions[0].clone());
        for fact in [&principal, &authorization] {
            assert!(fact.remains_equal_in(execution, runtime, snapshot).unwrap());
        }
        crate::test_read_stops::every_consult(
            |request| decisions[0].remains_equal_in(request, runtime, snapshot),
            worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled,
            worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted,
        );
        source.cancel();
        for fact in [&principal, &authorization] {
            assert_eq!(
                fact.remains_equal_in(execution, runtime, snapshot)
                    .unwrap_err()
                    .kind(),
                Kind::Cancelled
            );
        }
    }
    pub(in crate::domain_computation) fn assert_cancelled_retained_activation_read(
        &self,
        runtime: &RelationalRuntime,
        execution: &ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
        source: &CancellationSource,
    ) {
        let Some(WorthQueryRetainedAuthorizationDecisionFacts::Abilities { decisions, .. }) =
            &self.authorization
        else {
            panic!("ViewAccount must retain conventional observations");
        };
        assert_eq!(decisions.len(), 1);
        decisions[0]
            .assert_retained_activation_read_cancelled(runtime, execution, snapshot, source);
    }
}
