use super::{WorthUiActiveApplicationSession, WorthUiPendingApplicationCutover};

impl WorthUiActiveApplicationSession {
    /// Lets transaction tests retain the exact production-staged pending
    /// authority while inspecting denial behavior below the public cutover.
    pub(crate) fn into_runtime_and_pending_after_staging_for_test(
        self,
        pending: WorthUiPendingApplicationCutover,
    ) -> (
        crate::runtime::WorthUiRuntime,
        crate::runtime::WorthUiPendingActivation,
    ) {
        assert!(pending.basis.admits_session(self.session_identity()));
        (
            self.application.into_runtime_for_test(),
            pending.pending_activation,
        )
    }
}
