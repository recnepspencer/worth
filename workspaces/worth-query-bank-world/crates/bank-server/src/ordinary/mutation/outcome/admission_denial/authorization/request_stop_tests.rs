//! Bank adaptation retains the request's typed cause.
use super::*;
#[test]
fn authorization_request_causes_survive_bank_adaptation() {
    use worth_query_host::facade::application_contribution::{
        WorthQueryManagedComputationResourceDenial as Resource, WorthQueryMemoryLimitLevel as Level,
    };
    use WorthQueryProviderSessionDenialKind as Session;
    let resource = |denial| Session::ExecutionResource {
        denial,
        partition_identity: None,
        policy_ancestor: None,
    };
    for cause in [
        resource(Resource::NoActiveExecutionScope),
        Session::ExecutionWorkerPanicked {
            partition_identity: None,
        },
        Session::ExecutionNestedPatternStopped {
            partition_identity: None,
        },
        Session::ExecutionUncheckedCustomKernel {
            partition_identity: None,
        },
        resource(Resource::EquivalenceContractUnavailable),
        Session::ExecutionIdentitiesNotCanonical {
            partition_identity: None,
        },
        resource(Resource::WorkExhausted),
        resource(Resource::MemoryLimit {
            level: Level::Declared,
            requested: 7,
            admitted: 3,
        }),
    ] {
        let denial = BankAuthorizationDenial::from_kind(QueryKind::Stopped(Ok(cause)), 1);
        assert_eq!(
            denial.kind(),
            BankAuthorizationDenialKind::ExecutionRequest(cause)
        );
        assert_eq!(denial.code(), "execution-request");
        assert_eq!(denial.contributing_cause_count(), 1);
    }
}

#[test]
fn workflow_actor_refusals_keep_their_exact_bank_cause_and_code() {
    for (cause, expected, code) in [
        (
            AuthorizationRefusal::WorkflowActorContextRequired,
            BankAuthorizationDenialKind::WorkflowActorContextRequired,
            "workflow-actor-context-required",
        ),
        (
            AuthorizationRefusal::WorkflowActorAbsent,
            BankAuthorizationDenialKind::WorkflowActorAbsent,
            "workflow-actor-absent",
        ),
        (
            AuthorizationRefusal::WorkflowActorComparisonMismatch,
            BankAuthorizationDenialKind::WorkflowActorComparisonMismatch,
            "workflow-actor-comparison-mismatch",
        ),
        (
            AuthorizationRefusal::WorkflowActorSourceInvalid,
            BankAuthorizationDenialKind::WorkflowActorSourceInvalid,
            "workflow-actor-source-invalid",
        ),
        (
            AuthorizationRefusal::WorkflowActorEvidenceUnavailable,
            BankAuthorizationDenialKind::WorkflowActorEvidenceUnavailable,
            "workflow-actor-evidence-unavailable",
        ),
    ] {
        let denial = BankAuthorizationDenial::from_kind(QueryKind::Refused(cause), 3);
        assert_eq!(denial.kind(), expected);
        assert_eq!(denial.code(), code);
        assert_eq!(denial.contributing_cause_count(), 3);
    }
}
