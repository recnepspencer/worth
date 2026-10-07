use super::*;
use crate::domain_computation::primary_graph::bootstrap_publication::{
    commit_refusals::{refused_commit, work_refusal},
    execution_refusals::{isolated, request},
};
use crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial as Resource;
use crate::domain_computation::{
    WorthQueryProviderSessionControlStopKind as Control,
    WorthQueryProviderSessionDenialKind as Kind,
};
use std::time::Instant;
use worth_execution::CancellationSource;

#[test]
fn restoration_preserves_real_execution_refusals() {
    isolated(
        concat!(
            module_path!(),
            "::restoration_preserves_real_execution_refusals"
        ),
        || {
            let error = RelationalMaterializationError::Commit(work_refusal());
            let WorthQueryGeneratedOutputRestorationFailureCause::ExecutionDenied(kind) =
                preparation_failure_cause(&error)
            else {
                panic!("restoration folded a real execution refusal");
            };
            assert_eq!(
                kind,
                Kind::ExecutionResource {
                    denial: Resource::WorkExhausted,
                    partition_identity: Some(1),
                    policy_ancestor: None
                }
            );
            let source = CancellationSource::new();
            source.cancel();
            let mut canceled = request();
            canceled.cancellation = source.token();
            let mut expired = request();
            expired.deadline = Some(Instant::now());
            for (request, expected) in
                [(canceled, Control::Cancelled), (expired, Control::TimedOut)]
            {
                let error = RelationalMaterializationError::Commit(refused_commit(request));
                let WorthQueryGeneratedOutputRestorationFailureCause::ExecutionControlStopped(kind) =
                    preparation_failure_cause(&error)
                else {
                    panic!("restoration folded control stop");
                };
                assert_eq!(kind, expected);
            }
        },
    );
}
