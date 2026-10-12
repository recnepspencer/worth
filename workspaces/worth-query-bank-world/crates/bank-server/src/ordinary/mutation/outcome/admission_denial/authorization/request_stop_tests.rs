//! Bank adaptation retains the request's typed cause.
use super::*;
#[test]
fn authorization_request_causes_survive_bank_adaptation() {
    use RelationalExecutionDenialCause as Cause;
    for cause in [
        Cause::NoActiveExecutionScope,
        Cause::WorkerFailed,
        Cause::NestedStopped,
        Cause::UncheckedCustomKernel,
        Cause::EquivalenceContractUnavailable,
        Cause::ExpectedIdentitiesNotCanonical,
        Cause::WorkExhausted,
        Cause::DeclaredMemoryExhausted {
            requested: 7,
            admitted: 3,
        },
    ] {
        let denial = BankAuthorizationDenial::from_kind(QueryKind::RelationalExecution(cause), 1);
        assert_eq!(
            denial.kind(),
            BankAuthorizationDenialKind::RelationalExecution(cause)
        );
        assert_eq!(denial.code(), "relational-execution-denied");
        assert_eq!(denial.contributing_cause_count(), 1);
    }
    let cause = WorthQueryProviderSessionDenialKind::ExecutionWorkerPanicked {
        partition_identity: None,
    };
    let denial = BankAuthorizationDenial::from_kind(QueryKind::ExecutionRequest(cause), 1);
    assert_eq!(
        denial.kind(),
        BankAuthorizationDenialKind::ExecutionRequest(cause)
    );
    assert_eq!(denial.code(), "execution-request-denied");
}
