//! Owned refusal and last-owner custody for the new material receiver.
use super::execution::ProviderExecutionDenial;
use crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation;
use std::sync::Arc;
use worth_query_installation::facade::WorthQueryInstalledQueryBindingAdmissionStop;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as ExecutionCause;

mod causes;
pub use causes::{WorthQueryManagedIndexDenialCause, WorthQueryManagedIndexRefusal};

pub(super) enum ReceiverCause {
    Provider(ProviderExecutionDenial),
    Execution(ExecutionCause),
    Read(crate::domain_computation::primary_graph::vocabulary::application_query::one_shot_denial::WorthQueryApplicationOneShotDenial),
    LiveAdmission(super::super::WorthQueryApplicationLiveOpenDenial),
    Installation(WorthQueryInstalledQueryBindingAdmissionStop<ExecutionCause>),
    Product(crate::basis::WorthQueryProductBranchAdmissionDenial),
    Refused(WorthQueryManagedIndexRefusal),
}
impl From<crate::domain_computation::primary_graph::vocabulary::application_query::one_shot_denial::WorthQueryApplicationOneShotDenial> for ReceiverCause {
    fn from(cause: crate::domain_computation::primary_graph::vocabulary::application_query::one_shot_denial::WorthQueryApplicationOneShotDenial) -> Self {
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

/// A failed managed-index operation with its exact owned cause and material custody.
pub struct WorthQueryManagedIndexDenial {
    cause: ReceiverCause,
    // Owned diagnostic backing drops before the actual original grant.
    _allocation: Option<Arc<SourceFactsAllocation>>,
}
impl WorthQueryManagedIndexDenial {
    pub(super) fn new(
        cause: ReceiverCause,
        allocation: Option<Arc<SourceFactsAllocation>>,
    ) -> Self {
        Self {
            cause,
            _allocation: allocation,
        }
    }
    /// Borrows the original typed cause; no session or root authority is exposed.
    pub fn cause(&self) -> WorthQueryManagedIndexDenialCause<'_> {
        use WorthQueryManagedIndexDenialCause as Cause;
        match &self.cause {
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
impl std::fmt::Debug for WorthQueryManagedIndexDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ManagedIndexDenial")
            .field(&self.cause())
            .finish()
    }
}
impl std::fmt::Display for WorthQueryManagedIndexDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "managed index refused: {:?}", self.cause())
    }
}
impl std::error::Error for WorthQueryManagedIndexDenial {}

impl PartialEq for WorthQueryManagedIndexDenial {
    fn eq(&self, other: &Self) -> bool {
        self.cause() == other.cause()
    }
}
impl Eq for WorthQueryManagedIndexDenial {}

impl WorthQueryManagedIndexDenial {
    pub(in crate::domain_computation::primary_graph) fn retain(
        mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, ExecutionCause> {
        let allocation = match &self._allocation {
            Some(value) => Arc::clone(value),
            None => SourceFactsAllocation::open(execution)?,
        };
        crate::domain_computation::primary_graph::application_query::one_shot::outcome::material::MaterialCompletion::new(execution, &allocation).arc::<Self>()?;
        self._allocation = Some(allocation);
        Ok(Arc::new(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use {
        crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial,
        crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind as Kind,
        crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandRecoveryPosture as Posture,
        crate::domain_computation::primary_graph::application_query::SourceQueryObservationStop,
    };

    #[test]
    fn source_verification_preserves_request_causes_and_intrinsic_mismatch() {
        use {crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind as Authorization, crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationRefusalKind as Refusal, crate::domain_computation::primary_graph::vocabulary::invariant_projection::verification::ConsumedOutputVerificationStop as Stop, crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial as Attempt};
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
            let cause = Arc::new(WorthQueryManagedIndexDenial::new(
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
        let cause = Arc::new(WorthQueryManagedIndexDenial::new(
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
