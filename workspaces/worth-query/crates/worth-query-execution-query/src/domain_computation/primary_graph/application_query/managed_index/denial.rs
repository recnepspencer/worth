//! Owned refusal and last-owner custody for the new material receiver.
use super::execution::ProviderExecutionDenial;
use crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation;
use std::sync::Arc;
use worth_query_installation::facade::WorthQueryInstalledQueryBindingAdmissionStop;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as ExecutionCause;

mod causes;
pub use causes::{WorthQueryManagedIndexDenialCause, WorthQueryManagedIndexRefusal};

pub enum ReceiverCause {
    Provider(ProviderExecutionDenial),
    Execution(ExecutionCause),
    Read(super::super::WorthQueryApplicationOneShotDenial),
    LiveAdmission(super::super::WorthQueryApplicationLiveOpenDenial),
    Installation(WorthQueryInstalledQueryBindingAdmissionStop<ExecutionCause>),
    Product(crate::basis::WorthQueryProductBranchAdmissionDenial),
    Refused(WorthQueryManagedIndexRefusal),
}
impl From<super::super::WorthQueryApplicationOneShotDenial> for ReceiverCause {
    fn from(cause: super::super::WorthQueryApplicationOneShotDenial) -> Self {
        Self::Read(cause)
    }
}
impl From<ExecutionCause> for ReceiverCause {
    fn from(cause: ExecutionCause) -> Self {
        Self::Execution(cause)
    }
}
impl From<crate::basis::WorthQueryProductBranchAdmissionDenial> for ReceiverCause {
    fn from(cause: crate::basis::WorthQueryProductBranchAdmissionDenial) -> Self {
        Self::Product(cause)
    }
}

pub use worth_query_execution_lineage::facade::primary_graph::application_query::managed_index::denial::WorthQueryManagedIndexDenial;

impl worth_query_execution_lineage::facade::ports::ManagedIndexCausePort for ReceiverCause {
    fn cause(&self) -> WorthQueryManagedIndexDenialCause<'_> {
        use WorthQueryManagedIndexDenialCause as Cause;
        match self {
            ReceiverCause::Provider(cause) => causes::provider(cause),
            ReceiverCause::Execution(cause) => Cause::Execution(cause),
            ReceiverCause::Read(cause) => Cause::QueryRead(cause),
            ReceiverCause::LiveAdmission(cause) => Cause::LiveAdmission(cause),
            ReceiverCause::Product(cause) => Cause::Product(cause),
            ReceiverCause::Refused(cause) => Cause::Refused(*cause),
            ReceiverCause::Installation(cause) => match cause {
                WorthQueryInstalledQueryBindingAdmissionStop::Installation(cause) => {
                    Cause::Installation(cause)
                }
                WorthQueryInstalledQueryBindingAdmissionStop::Admission(cause) => {
                    Cause::Execution(cause)
                }
                WorthQueryInstalledQueryBindingAdmissionStop::AccountingOverflow => {
                    Cause::Refused(WorthQueryManagedIndexRefusal::InstallationAccountingOverflow)
                }
            },
        }
    }
}

pub fn managed_index_denial(
    cause: ReceiverCause,
    allocation: Option<Arc<SourceFactsAllocation>>,
) -> WorthQueryManagedIndexDenial {
    WorthQueryManagedIndexDenial::from_cause(Box::new(cause), allocation)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_computation::primary_graph::{
        application_query::SourceQueryObservationStop, WorthQueryOutputDemandDenial,
        WorthQueryOutputDemandDenialKind as Kind, WorthQueryOutputDemandRecoveryPosture as Posture,
    };

    #[test]
    fn source_verification_preserves_request_causes_and_intrinsic_mismatch() {
        use crate::domain_computation::{
            authorization::{
                WorthQueryOperationAuthorizationDenialKind as Authorization,
                WorthQueryOperationAuthorizationRefusalKind as Refusal,
            },
            primary_graph::{
                invariant_projection::ConsumedOutputVerificationStop as Stop,
                WorthQueryApplicationAttemptDenial as Attempt,
            },
        };
        let authorization = Authorization::Refused(Refusal::CapabilityGrantMissing);
        for (stop, kind, posture) in [
            (
                Stop::Execution(ExecutionCause::Cancelled),
                Kind::Cancelled,
                Posture::Retryable,
            ),
            (
                Stop::Execution(ExecutionCause::DeadlineElapsed),
                Kind::TimedOut,
                Posture::Retryable,
            ),
            (
                Stop::SourceQueryAuthorization(authorization),
                Kind::of_request_authorization(authorization),
                Posture::Retryable,
            ),
            (
                Stop::WorkExhausted,
                Kind::WorkBudgetExceeded,
                Posture::Retryable,
            ),
            (
                Stop::Unavailable,
                Kind::ManagedIndexSourceDenied,
                Posture::Terminal,
            ),
        ] {
            let cause = Arc::new(managed_index_denial(
                ReceiverCause::Provider(ProviderExecutionDenial::SourceVerification(
                    Attempt::output_observation(stop),
                )),
                None,
            ));
            let projected = WorthQueryOutputDemandDenial::managed_source_query(
                Arc::clone(&cause),
                "managed Source",
            );
            assert_eq!(projected.kind(), kind);
            assert_eq!(projected.recovery_posture(), posture);
            assert_eq!(
                projected.managed_source_query_denial().unwrap().cause(),
                cause.cause()
            );
        }
    }

    #[test]
    fn current_control_keeps_its_original_kind_and_retryability_in_output_demand() {
        let original =
            WorthQueryOutputDemandDenial::new(Kind::CurrentControlBusy, "current control");
        let cause = Arc::new(managed_index_denial(
            ReceiverCause::Provider(ProviderExecutionDenial::SourceRead(
                SourceQueryObservationStop::CurrentControl(original.clone()),
            )),
            None,
        ));
        let projected = WorthQueryOutputDemandDenial::managed_source_query(cause, "managed Source");
        assert_eq!(projected.kind(), Kind::CurrentControlBusy);
        assert_eq!(projected.recovery_posture(), Posture::Retryable);
        assert!(
            matches!(projected.managed_source_query_denial().unwrap().cause(),
            WorthQueryManagedIndexDenialCause::CurrentControl(stop)
                if stop.kind() == original.kind() && stop.recovery_posture() == Posture::Retryable)
        );
    }
}
