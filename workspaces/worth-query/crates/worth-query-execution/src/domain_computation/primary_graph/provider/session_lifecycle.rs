use std::sync::Arc;

use super::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::{
    WorthQueryProviderExecutionPlanView, WorthQueryProviderSessionFailure,
    WorthQueryProviderSessionLifecycle, WorthQueryProviderSessionProtocolStage,
    WorthQueryProviderSessionToken, WorthQueryProviderSessionTokenAdmission,
    WorthQueryProviderSessionView,
};

impl WorthQueryProviderSessionLifecycle for Arc<WorthQueryPrimaryGraphProvider> {
    fn readmit_provider_plan(
        &self,
        _plan: &WorthQueryProviderExecutionPlanView<'_>,
        admission: WorthQueryProviderSessionTokenAdmission,
    ) -> Result<WorthQueryProviderSessionToken, WorthQueryProviderSessionFailure> {
        self.application_attempt_work
            .observe_provider_session_readmission();
        admission.admit("primary-application-relational-session")
    }

    fn prepare_provider_session(
        &self,
        _session: &WorthQueryProviderSessionView<'_>,
    ) -> Result<(), WorthQueryProviderSessionFailure> {
        self.application_attempt_work
            .observe_provider_session_preparation();
        if self.take_rejected_session_prepare() {
            return Err(super::session_commit::provider_failure(
                WorthQueryProviderSessionProtocolStage::SessionPreparation,
                "injected primary graph session preparation rejection",
            ));
        }
        Ok(())
    }

    fn prepare_staged_session(
        &self,
        session: &WorthQueryProviderSessionView<'_>,
    ) -> Result<(), WorthQueryProviderSessionFailure> {
        self.application_attempt_work
            .observe_staged_session_preparation();
        let attempts = self
            .attempts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if attempts.is_staged_session_preparable(*session) {
            Ok(())
        } else {
            Err(super::session_commit::provider_failure(
                WorthQueryProviderSessionProtocolStage::StagedPreparation,
                "primary graph session has no exact staged overlay",
            ))
        }
    }

    fn commit_prepared_session(
        &self,
        session: &WorthQueryProviderSessionView<'_>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        crate::domain_computation::WorthQueryProviderTerminalDescription,
        crate::domain_computation::WorthQueryProviderSessionCommitStop,
    > {
        self.application_attempt_work.observe_prepared_commit();
        super::session_commit::commit_prepared_session(self, *session, allocation_policy)
    }

    fn abort_provider_session(
        &self,
        session: &WorthQueryProviderSessionView<'_>,
    ) -> Result<
        crate::domain_computation::WorthQueryProviderTerminalDescription,
        WorthQueryProviderSessionFailure,
    > {
        self.application_attempt_work.observe_attempt_abort();
        self.attempts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .abort(*session);
        crate::domain_computation::WorthQueryProviderTerminalDescription::new(format!(
            "primary application abort completed for {}",
            session.identity()
        ))
        .map_err(|_| {
            super::session_commit::provider_failure(
                WorthQueryProviderSessionProtocolStage::Abort,
                "primary application abort description is invalid",
            )
        })
    }
}
