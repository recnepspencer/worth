//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandDenial {
    #[doc(hidden)]
    pub fn readmission_failure_for_test(&self) -> &Option<&'static str> {
        &self.readmission_failure
    }
}

impl WorthQueryOutputDemandDenial {
    #[doc(hidden)]
    pub fn readmission_failure_mut_for_test(&mut self) -> &mut Option<&'static str> {
        &mut self.readmission_failure
    }
}

impl WorthQueryOutputDemandDenial {
    #[doc(hidden)]
    pub fn new_for_test(
        kind: WorthQueryOutputDemandDenialKind,
        subject: impl Into<std::borrow::Cow<'static, str>>,
    ) -> Self {
        Self::new(kind, subject)
    }
}

impl WorthQueryOutputDemandDenial {
    #[doc(hidden)]
    pub fn producer_domain_denied_for_test(
        subject: impl Into<std::borrow::Cow<'static, str>>,
        reason: Option<&'static str>,
        value: crate::domain_computation::primary_graph::vocabulary::application_contribution::denial_value::WorthQueryProducerDenialValue,
    ) -> Self {
        Self::producer_domain_denied(subject, reason, value)
    }
}

impl WorthQueryOutputDemandDenial {
    #[doc(hidden)]
    pub fn take_requested_output_for_test(
        &mut self,
    ) -> Option<crate::domain_computation::primary_graph::invariant_projection::RequestedOutputRead>
    {
        Self::take_requested_output(self)
    }
}

impl WorthQueryOutputDemandDenial {
    #[doc(hidden)]
    pub fn with_recovery_posture_for_test(
        self,
        recovery_posture: WorthQueryOutputDemandRecoveryPosture,
    ) -> Self {
        Self::with_recovery_posture(self, recovery_posture)
    }
}
