use super::*;
use crate::domain_computation::primary_graph::application_attempt::provider_compare_denial::provider_session_denied;
use crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial as Resource;

#[test]
fn cleanup_failure_keeps_heads_indeterminate_posture_for_execution_refusals() {
    crate::domain_computation::primary_graph::with_test_advancement(|_active_phase| {
        let cases = [
            Session::ExecutionResource {
                denial: Resource::ScratchCapacityExceeded,
                partition_identity: Some(3),
                policy_ancestor: None,
            },
            Session::ExecutionNestedPatternStopped {
                partition_identity: Some(3),
            },
            Session::ExecutionWorkerPanicked {
                partition_identity: Some(3),
            },
            Session::ExecutionUncheckedCustomKernel {
                partition_identity: Some(3),
            },
            Session::ExecutionIdentitiesNotCanonical {
                partition_identity: Some(3),
            },
        ];
        for kind in cases {
            let denial = provider_session_denied(WorthQueryProviderSessionFailure::new(
                kind,
                WorthQueryProviderSessionProtocolStage::Commit,
                "commit preparation execution refused",
                Default::default(),
            ));
            let Outcome::Indeterminate(evidence) =
                cleanup_failed_outcome(Progression::Denied(denial))
            else {
                panic!("failed cleanup changed HEAD's indeterminate posture: {kind:?}");
            };
            assert_eq!(evidence.denial_kind(), kind);
            assert_eq!(evidence.recovery(),
            crate::domain_computation::primary_graph::WorthQueryApplicationCommitRecoveryKind::CommitRecoveryRequired);
        }
    });
}
