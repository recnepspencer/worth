//! The adapter preserves request stops even when native recovery nests them.
use super::*;
use worth_query_host::facade::application_contribution::{
    WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
};

#[test]
fn command_preparation_and_recovery_keep_native_refusals() {
    for (interruption, relational, expected) in [
        (
            WorthQueryManagedComputationInterruption::Cancelled,
            RelationalExecutionDenialCause::Cancelled,
            Kind::Cancelled,
        ),
        (
            WorthQueryManagedComputationInterruption::DeadlineExceeded,
            RelationalExecutionDenialCause::DeadlineElapsed,
            Kind::DeadlineExceeded,
        ),
    ] {
        let expected = BankHttpDenial::new(expected, Next::Retry);
        assert_eq!(
            command_denial(WorthQueryCapabilityCommandDenial::Execution(
                WorthQueryAdvancementDenial::Interrupted(interruption)
            )),
            expected
        );
        assert_eq!(
            native_denial(NativeDenial::Recovery(
                WorthQueryManagedApplicationRecoveryDenial::ProviderAdmission(
                    CompanionPreflightStop::Execution(relational)
                )
            )),
            expected
        );
        assert_eq!(
            native_denial(NativeDenial::Preparation(
                bank_server::BankEstateProgressionDenial::Projection(
                    bank_server::BankEstateOperationProjectionDenial::InvariantAdmission(
                        WorthQueryInvariantProjectionDenialKind::ExecutionRequest(relational),
                    ),
                ),
            )),
            expected,
        );
        let kind = match interruption {
            WorthQueryManagedComputationInterruption::Cancelled => {
                bank_server::BankApplicationAttemptDenialKind::Cancelled
            }
            WorthQueryManagedComputationInterruption::DeadlineExceeded => {
                bank_server::BankApplicationAttemptDenialKind::DeadlineExceeded
            }
        };
        assert_eq!(
            native_denial(NativeDenial::Preparation(
                bank_server::BankEstateProgressionDenial::Attempt(
                    bank_server::BankCommitPreparationDenial::Application {
                        kind,
                        allocation: None
                    },
                ),
            )),
            expected,
        );
    }
    assert_eq!(
        command_denial(WorthQueryCapabilityCommandDenial::FreshRequestMismatch),
        BankHttpDenial::new(Kind::PermissionDenied, Next::None)
    );
    assert_eq!(
        native_denial(NativeDenial::Preparation(
            bank_server::BankEstateProgressionDenial::Attempt(
                bank_server::BankCommitPreparationDenial::Application {
                    kind: bank_server::BankApplicationAttemptDenialKind::ExecutionRequest(
                        WorthQueryProviderSessionDenialKind::ExecutionWorkerPanicked {
                            partition_identity: None,
                        },
                    ),
                    allocation: None,
                },
            ),
        )),
        BankHttpDenial::new(Kind::InternalDenied, Next::ContactOperator),
    );
}
