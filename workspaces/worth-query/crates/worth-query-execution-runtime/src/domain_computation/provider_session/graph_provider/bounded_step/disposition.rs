use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryGraphProviderStepDispositionKind {
    Continue,
    Complete,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorthQueryGraphProviderStepDisposition {
    kind: WorthQueryGraphProviderStepDispositionKind,
    provider_receipt: Option<Arc<str>>,
}

impl WorthQueryGraphProviderStepDisposition {
    pub const fn continue_work() -> Self {
        Self {
            kind: WorthQueryGraphProviderStepDispositionKind::Continue,
            provider_receipt: None,
        }
    }

    pub fn complete(provider_receipt: impl Into<Arc<str>>) -> Result<Self, &'static str> {
        let provider_receipt = provider_receipt.into();
        if provider_receipt.trim().is_empty()
            || provider_receipt.trim() != provider_receipt.as_ref()
        {
            return Err("invalid-provider-step-receipt");
        }
        Ok(Self {
            kind: WorthQueryGraphProviderStepDispositionKind::Complete,
            provider_receipt: Some(provider_receipt),
        })
    }

    pub const fn kind(&self) -> WorthQueryGraphProviderStepDispositionKind {
        self.kind
    }

    pub fn provider_receipt(&self) -> Option<&str> {
        self.provider_receipt.as_deref()
    }

    pub(super) fn into_provider_receipt(self) -> Option<Arc<str>> {
        self.provider_receipt
    }
}
