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
            assert_eq!(evidence.denial_kind(), Ok(kind));
            assert_eq!(evidence.recovery(),
            crate::domain_computation::primary_graph::WorthQueryApplicationCommitRecoveryKind::CommitRecoveryRequired);
        }
    });
}

#[test]
fn cleanup_failure_preserves_the_complete_interrupted_commit_evidence() {
    use crate::domain_computation::primary_graph::{
        WorthQueryApplicationCommitRecoveryKind as Recovery,
        WorthQueryApplicationUnresolvedCommitEvidence as Evidence,
    };
    use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Stop;
    for stop in [Stop::Cancelled, Stop::TimedOut] {
        let failure = WorthQueryProviderSessionFailure::checked_read_stopped(
            Err(stop),
            WorthQueryProviderSessionProtocolStage::Commit,
            "request stopped during aftermath sealing",
        );
        let evidence =
            Evidence::from_provider_session_failure(Recovery::CommitRecoveryRequired, &failure);
        let Outcome::Indeterminate(retained) =
            cleanup_failed_outcome(Progression::Indeterminate(evidence.clone()))
        else {
            panic!("cleanup must retain indeterminate evidence")
        };
        assert_eq!(retained, evidence.with_cleanup_failure());
    }
}
