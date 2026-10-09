use std::sync::Arc;

use super::{
    WorthQueryProvisionalDenialKind, WorthQueryProvisionalFailure,
    WorthQueryProvisionalOverlayEvidence,
};
use crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor;

/// Owns one physical provisional overlay until it is explicitly discarded.
///
/// The field is ordered before the provider session in the enclosing attempt,
/// so abandonment discards provisional state before the session guard aborts.
pub(crate) struct WorthQueryProvisionalOverlayLease<'scope> {
    execution: worth_execution::ExecutionRequest<'scope, 'scope>,
    provider: Arc<WorthQueryGraphProviderAnchor>,
    evidence: WorthQueryProvisionalOverlayEvidence,
    active: bool,
}

impl<'scope> WorthQueryProvisionalOverlayLease<'scope> {
    pub(crate) fn new(
        execution: worth_execution::ExecutionRequest<'scope, 'scope>,
        provider: Arc<WorthQueryGraphProviderAnchor>,
        evidence: WorthQueryProvisionalOverlayEvidence,
    ) -> Self {
        Self {
            execution,
            provider,
            evidence,
            active: true,
        }
    }

    pub(crate) fn evidence(&self) -> &WorthQueryProvisionalOverlayEvidence {
        &self.evidence
    }

    pub(crate) fn discard(&mut self) -> Result<(), WorthQueryProvisionalFailure> {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.provider
                .discard_provisional_overlay(self.execution, self.evidence.view())
        }))
        .unwrap_or_else(|_| {
            Err(WorthQueryProvisionalFailure::new(
                WorthQueryProvisionalDenialKind::ProviderPanicked,
                "provider panicked while discarding a provisional overlay",
            ))
        });
        if result.is_ok() {
            self.active = false;
        }
        result
    }

    pub(crate) fn release_to_provider_resolution(&mut self) {
        self.active = false;
    }
}

impl Drop for WorthQueryProvisionalOverlayLease<'_> {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = self
                .provider
                .discard_provisional_overlay(self.execution, self.evidence.view());
        }));
        self.active = false;
    }
}
