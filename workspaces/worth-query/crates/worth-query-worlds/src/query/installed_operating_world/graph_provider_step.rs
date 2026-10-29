pub use std::sync::Arc;

pub use worth_query::facade::domain;

pub struct FixtureGraphProviderExecution {
    pub provider_receipt: Arc<str>,
    pub projection: Option<domain::WorthQueryGraphReadMaterial>,
    pub applied_effect: bool,
    pub failure: Option<domain::WorthQueryGraphProviderFailure>,
    pub advanced: bool,
}

impl FixtureGraphProviderExecution {
    pub fn read(provider_receipt: impl Into<Arc<str>>) -> Self {
        Self::new(provider_receipt, None, false)
    }

    pub fn projection(
        provider_receipt: impl Into<Arc<str>>,
        projection: domain::WorthQueryGraphReadMaterial,
    ) -> Self {
        Self::new(provider_receipt, Some(projection), false)
    }

    pub fn effect(provider_receipt: impl Into<Arc<str>>) -> Self {
        Self::new(provider_receipt, None, true)
    }

    pub fn failed(detail: impl Into<String>) -> Self {
        Self {
            provider_receipt: Arc::from("failed-provider-step"),
            projection: None,
            applied_effect: false,
            failure: Some(domain::WorthQueryGraphProviderFailure::new(detail)),
            advanced: false,
        }
    }

    pub fn new(
        provider_receipt: impl Into<Arc<str>>,
        projection: Option<domain::WorthQueryGraphReadMaterial>,
        applied_effect: bool,
    ) -> Self {
        Self {
            provider_receipt: provider_receipt.into(),
            projection,
            applied_effect,
            failure: None,
            advanced: false,
        }
    }
}

impl domain::WorthQueryGraphProviderExecution for FixtureGraphProviderExecution {
    fn advance(
        &mut self,
        step: &mut domain::WorthQueryGraphProviderStep,
    ) -> Result<
        domain::WorthQueryGraphProviderStepDisposition,
        domain::WorthQueryGraphProviderFailure,
    > {
        if self.advanced {
            return Err(domain::WorthQueryGraphProviderFailure::new(
                "one-step fixture provider was advanced twice",
            ));
        }
        self.advanced = true;
        if let Some(failure) = self.failure.take() {
            return Err(failure);
        }
        if self.applied_effect {
            step.apply_effect(|| Ok(()))?;
        } else {
            step.perform_work_unit(|| Ok(()))?;
        }
        if let Some(projection) = self.projection.take() {
            step.emit_projection_chunk(projection)
                .map_err(|denial| domain::WorthQueryGraphProviderFailure::new(denial.detail()))?;
        }
        domain::WorthQueryGraphProviderStepDisposition::complete(Arc::clone(&self.provider_receipt))
            .map_err(domain::WorthQueryGraphProviderFailure::new)
    }

    fn dispose(&mut self) -> Result<(), domain::WorthQueryGraphProviderFailure> {
        Ok(())
    }
}
