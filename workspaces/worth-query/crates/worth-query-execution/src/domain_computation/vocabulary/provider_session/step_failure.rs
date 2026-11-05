//! Moved whole from `domain_computation::provider_session::graph_provider::bounded_step::step_failure`: the items below and every impl they own.
use crate::domain_computation::vocabulary::provider_session::denial::WorthQueryGraphProviderStepDenial;
use crate::domain_computation::vocabulary::provider_session::denial::WorthQueryGraphProviderStepDenialKind;
use crate::domain_computation::vocabulary::provider_session::failure::WorthQueryGraphProviderFailure;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryGraphProviderStepInvocationDisposition {
    Returned,
    Rejected,
    Panicked,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphProviderStepFailureEvidence {
    invocation: WorthQueryGraphProviderStepInvocationDisposition,
    invocation_failure_detail: Option<Arc<str>>,
    latched_provider_failure_detail: Option<Arc<str>>,
    governed_denial: Option<WorthQueryGraphProviderStepDenial>,
}

impl WorthQueryGraphProviderStepFailureEvidence {
    pub(in crate::domain_computation) fn returned(
        governed_denial: Option<WorthQueryGraphProviderStepDenial>,
        provider_failure: Option<WorthQueryGraphProviderFailure>,
    ) -> Self {
        Self {
            invocation: WorthQueryGraphProviderStepInvocationDisposition::Returned,
            invocation_failure_detail: None,
            latched_provider_failure_detail: provider_failure
                .map(|failure| Arc::from(failure.detail())),
            governed_denial,
        }
    }

    pub(in crate::domain_computation) fn rejected(
        failure: WorthQueryGraphProviderFailure,
        governed_denial: Option<WorthQueryGraphProviderStepDenial>,
        latched_provider_failure: Option<WorthQueryGraphProviderFailure>,
    ) -> Self {
        Self {
            invocation: WorthQueryGraphProviderStepInvocationDisposition::Rejected,
            invocation_failure_detail: Some(Arc::from(failure.detail())),
            latched_provider_failure_detail: latched_provider_failure
                .map(|failure| Arc::from(failure.detail())),
            governed_denial,
        }
    }

    pub(in crate::domain_computation) fn panicked(
        governed_denial: Option<WorthQueryGraphProviderStepDenial>,
        provider_failure: Option<WorthQueryGraphProviderFailure>,
    ) -> Self {
        Self {
            invocation: WorthQueryGraphProviderStepInvocationDisposition::Panicked,
            invocation_failure_detail: None,
            latched_provider_failure_detail: provider_failure
                .map(|failure| Arc::from(failure.detail())),
            governed_denial,
        }
    }

    pub const fn invocation(&self) -> WorthQueryGraphProviderStepInvocationDisposition {
        self.invocation
    }

    pub fn provider_failure_detail(&self) -> Option<&str> {
        self.invocation_failure_detail
            .as_deref()
            .or(self.latched_provider_failure_detail.as_deref())
    }

    pub fn invocation_failure_detail(&self) -> Option<&str> {
        self.invocation_failure_detail.as_deref()
    }

    pub fn latched_provider_failure_detail(&self) -> Option<&str> {
        self.latched_provider_failure_detail.as_deref()
    }

    pub const fn governed_denial_kind(&self) -> Option<WorthQueryGraphProviderStepDenialKind> {
        match &self.governed_denial {
            Some(denial) => Some(denial.kind()),
            None => None,
        }
    }

    pub fn governed_denial_detail(&self) -> Option<&str> {
        self.governed_denial
            .as_ref()
            .map(WorthQueryGraphProviderStepDenial::detail)
    }
}
