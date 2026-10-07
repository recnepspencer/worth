use super::*;
use crate::domain_computation::primary_graph::application_attempt::provider_compare_denial::{
    provider_session_control_denied, provider_session_denied, provider_session_kind_denied,
};
use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
use crate::domain_computation::WorthQueryProviderSessionDenialKind as Session;

fn resources() -> Vec<Resource> {
    let mut causes = vec![
        Resource::WorkExhausted,
        Resource::WorkCounterOverflow,
        Resource::ResultCapacityExceeded,
        Resource::ScratchCapacityExceeded,
        Resource::CapacityOverflow,
        Resource::ChargedBytesOverflow,
        Resource::NestedLeaseMisuse,
        Resource::NoActiveExecutionScope,
    ];
    for level in [Level::Policy, Level::Process, Level::Declared] {
        causes.push(Resource::MemoryLimit {
            requested: 31,
            admitted: 17,
            level,
        });
    }
    causes
}

fn kinds() -> Vec<Session> {
    let mut kinds = vec![
        Session::ExecutionWorkerPanicked {
            partition_identity: Some(7),
        },
        Session::ExecutionNestedPatternStopped {
            partition_identity: Some(7),
        },
        Session::ExecutionUncheckedCustomKernel {
            partition_identity: Some(7),
        },
        Session::ExecutionIdentitiesNotCanonical {
            partition_identity: Some(7),
        },
    ];
    kinds.extend(
        resources()
            .into_iter()
            .map(|denial| Session::ExecutionResource {
                denial,
                partition_identity: Some(7),
                policy_ancestor: None,
            }),
    );
    kinds
}

#[test]
fn every_head_reachable_preparation_category_retries_without_folding() {
    for session in kinds() {
        let denial = provider_session_denied(
            crate::domain_computation::WorthQueryProviderSessionFailure::new(
                session,
                crate::domain_computation::WorthQueryProviderSessionProtocolStage::Commit,
                "owner refusal",
                Default::default(),
            ),
        );
        assert!(
            matches!(classify_denial(&denial), Outcome::RetryableCommitFailure(kind) if kind == denial.kind()),
            "{session:?}"
        );
    }
}

#[test]
fn unreachable_child_policy_causes_and_non_relational_retained_state_stay_terminal() {
    for denial in [
        Resource::WorkerLimit,
        Resource::PolicyMemoryLimit,
        Resource::WorkLimit,
        Resource::EquivalenceContractUnavailable,
        Resource::RetainedBytesExhausted,
    ] {
        let kind = Kind::ExecutionResource {
            denial,
            partition_identity: Some(7),
            policy_ancestor: None,
        };
        assert!(
            matches!(classify_kind(kind), Outcome::TerminalFailure(Terminal::ApplicationCommit(observed)) if observed == kind)
        );
    }
}

#[test]
fn pending_and_invariant_keep_head_category_stage_and_terminal_posture() {
    for stage in [Stage::Idempotency, Stage::InvariantExecution] {
        for kind in kinds() {
            let denial = provider_session_kind_denied(kind, stage, "owner supplied detail");
            assert_eq!(denial.kind(), Kind::ProviderRejected);
            assert_eq!(denial.stage(), stage);
            assert_eq!(denial.detail(), Some("owner supplied detail"));
            assert_eq!(denial.execution_denial_cause(), Some(Ok(kind)));
            assert!(
                matches!(classify_denial(&denial), Outcome::TerminalFailure(Terminal::ApplicationExecution { stage: observed, cause: Ok(cause) }) if observed == stage && cause == kind)
            );
        }
        for kind in [Control::Cancelled, Control::TimedOut] {
            let denial =
                provider_session_control_denied(kind, stage, "invariant owner control stop");
            assert_eq!(denial.kind(), Kind::ProviderRejected);
            assert_eq!(denial.stage(), stage);
            assert!(
                matches!(classify_denial(&denial), Outcome::TerminalFailure(Terminal::ApplicationExecution { stage: observed, cause: Err(cause) }) if observed == stage && cause == kind)
            );
        }
    }
}

pub(in crate::domain_computation::primary_graph) fn assert_preparation_retry(denial: &Denial) {
    match classify_denial(denial) {
        Outcome::RetryableCommitFailure(kind) => assert_eq!(kind, denial.kind()),
        Outcome::RetryableExecutionControlStopped(kind) => {
            assert_eq!(denial.execution_denial_cause(), Some(Err(kind)))
        }
        _ => panic!("HEAD-reachable preparation refusal changed retry posture: {denial:?}"),
    }
}
