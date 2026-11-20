use super::{WorthQueryGraphProviderStepDenial, WorthQueryGraphProviderStepDenialKind};
use crate::domain_computation::WorthQueryGraphProviderFailure;

#[derive(Default)]
pub(super) struct WorthQueryGraphProviderStepState {
    governed_denial: Option<WorthQueryGraphProviderStepDenial>,
    provider_failure: Option<WorthQueryGraphProviderFailure>,
}

impl WorthQueryGraphProviderStepState {
    pub(super) fn ensure_active(&self) -> Result<(), WorthQueryGraphProviderStepDenial> {
        if let Some(denial) = &self.governed_denial {
            return Err(denial.clone());
        }
        if self.provider_failure.is_some() {
            return Err(WorthQueryGraphProviderStepDenial::new(
                WorthQueryGraphProviderStepDenialKind::ProviderFailureLatched,
                "provider step already rejected a governed operation",
            ));
        }
        Ok(())
    }

    pub(super) fn admit<Output>(
        &mut self,
        result: Result<Output, WorthQueryGraphProviderStepDenial>,
    ) -> Result<Output, WorthQueryGraphProviderStepDenial> {
        self.ensure_active()?;
        match result {
            Ok(output) => Ok(output),
            Err(denial) => Err(self.deny(denial)),
        }
    }

    pub(super) fn deny(
        &mut self,
        denial: WorthQueryGraphProviderStepDenial,
    ) -> WorthQueryGraphProviderStepDenial {
        if let Err(existing) = self.ensure_active() {
            return existing;
        }
        self.governed_denial = Some(denial);
        self.governed_denial
            .as_ref()
            .expect("a denied provider step retains its first denial")
            .clone()
    }

    pub(super) fn reject_provider(
        &mut self,
        failure: WorthQueryGraphProviderFailure,
    ) -> WorthQueryGraphProviderFailure {
        if self.governed_denial.is_some() {
            return failure;
        }
        if self.provider_failure.is_none() {
            self.provider_failure = Some(failure);
        }
        self.provider_failure
            .as_ref()
            .expect("a rejected provider operation retains its first failure")
            .clone()
    }

    pub(super) const fn has_failure(&self) -> bool {
        self.governed_denial.is_some() || self.provider_failure.is_some()
    }

    pub(super) fn governed_denial(&self) -> Option<&WorthQueryGraphProviderStepDenial> {
        self.governed_denial.as_ref()
    }

    pub(super) fn provider_failure(&self) -> Option<&WorthQueryGraphProviderFailure> {
        self.provider_failure.as_ref()
    }
}
