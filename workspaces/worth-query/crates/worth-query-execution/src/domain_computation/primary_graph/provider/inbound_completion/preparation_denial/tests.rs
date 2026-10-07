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
fn inbound_completion_preserves_real_execution_refusals_at_each_phase() {
    isolated(
        concat!(
            module_path!(),
            "::inbound_completion_preserves_real_execution_refusals_at_each_phase"
        ),
        || {
            let error = work_refusal();
            let expected = Kind::ExecutionResource {
                denial: Resource::WorkExhausted,
                partition_identity: Some(1),
                policy_ancestor: None,
            };
            assert_eq!(
                validation_denial(&error),
                Denial::ExecutionDenied {
                    stage: Stage::InvariantExecution,
                    kind: expected
                }
            );
            assert_eq!(
                preparation_denial(&error),
                Denial::ExecutionDenied {
                    stage: Stage::ProviderCommit,
                    kind: expected
                }
            );
            let source = CancellationSource::new();
            source.cancel();
            let mut canceled = request();
            canceled.cancellation = source.token();
            let mut expired = request();
            expired.deadline = Some(Instant::now());
            for (request, kind) in [(canceled, Control::Cancelled), (expired, Control::TimedOut)] {
                let error = refused_commit(request);
                assert_eq!(
                    validation_denial(&error),
                    Denial::ExecutionControlStopped {
                        stage: Stage::InvariantExecution,
                        kind
                    }
                );
                assert_eq!(
                    preparation_denial(&error),
                    Denial::ExecutionControlStopped {
                        stage: Stage::ProviderCommit,
                        kind
                    }
                );
            }
        },
    );
}
