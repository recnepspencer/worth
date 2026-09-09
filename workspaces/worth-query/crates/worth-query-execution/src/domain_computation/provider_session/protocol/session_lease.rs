use std::sync::Arc;

use super::{WorthQueryGraphProviderAnchor, WorthQueryProviderSessionToken};

/// Framework-owned cleanup guard for one physical provider session.
///
/// Every live protocol state owns this guard. A successful terminal transition
/// disarms it; every other return, panic boundary, or caller abandonment makes
/// one best-effort provider abort before the physical session can be orphaned.
pub(crate) struct WorthQueryProviderSessionLease {
    provider: Arc<WorthQueryGraphProviderAnchor>,
    token: WorthQueryProviderSessionToken,
    active: bool,
}

impl WorthQueryProviderSessionLease {
    pub(super) fn new(
        provider: Arc<WorthQueryGraphProviderAnchor>,
        token: WorthQueryProviderSessionToken,
    ) -> Self {
        Self {
            provider,
            token,
            active: true,
        }
    }

    pub(super) fn provider(&self) -> &WorthQueryGraphProviderAnchor {
        &self.provider
    }

    pub(super) fn provider_arc(&self) -> Arc<WorthQueryGraphProviderAnchor> {
        Arc::clone(&self.provider)
    }

    pub(super) fn token(&self) -> &WorthQueryProviderSessionToken {
        &self.token
    }

    pub(super) fn close(&mut self) {
        self.active = false;
    }

    pub(super) fn abort(
        &mut self,
    ) -> Result<super::WorthQueryProviderTerminalDescription, super::WorthQueryProviderSessionFailure>
    {
        let result = self.provider.abort_session(&self.token.view());
        if result.is_ok() {
            self.close();
        }
        result
    }

    pub(super) fn commit(
        &mut self,
    ) -> Result<
        super::WorthQueryProviderTerminalDescription,
        super::WorthQueryProviderSessionCommitStop,
    > {
        let result = self.provider.commit_session(&self.token.view());
        if result.is_ok()
            || matches!(
                &result,
                Err(super::WorthQueryProviderSessionCommitStop::SettlementDeferred(_))
                    | Err(super::WorthQueryProviderSessionCommitStop::ProductUnpublished(_))
                    | Err(super::WorthQueryProviderSessionCommitStop::ControlStopped(
                        _
                    ))
            )
        {
            self.close();
        }
        result
    }

    pub(super) fn abort_after_failure(
        &mut self,
    ) -> super::WorthQueryProviderSessionRecoveryPosture {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.abort())) {
            Ok(Ok(_)) => super::WorthQueryProviderSessionRecoveryPosture::Closed,
            Ok(Err(_)) | Err(_) => {
                super::WorthQueryProviderSessionRecoveryPosture::RecoveryRequired
            }
        }
    }
}

impl Drop for WorthQueryProviderSessionLease {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = self.provider.abort_session(&self.token.view());
        }));
        self.active = false;
    }
}
