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
        admission.admit_home_owner(self.session_home_owner()?)?;
        self.application_attempt_work
            .observe_provider_session_readmission();
        admission.admit("primary-application-relational-session")
    }

    fn prepare_provider_session(
        &self,
        session: &WorthQueryProviderSessionView<'_>,
    ) -> Result<(), WorthQueryProviderSessionFailure> {
        session.admit_home_owner(
            self.session_home_owner()?,
            WorthQueryProviderSessionProtocolStage::SessionPreparation,
        )?;
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
        session.admit_home_owner(
            self.session_home_owner()?,
            WorthQueryProviderSessionProtocolStage::StagedPreparation,
        )?;
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
        crate::domain_computation::WorthQueryProviderCommitCompletion,
        crate::domain_computation::WorthQueryProviderSessionCommitStop,
    > {
        session.admit_home_owner(
            self.session_home_owner()?,
            WorthQueryProviderSessionProtocolStage::Commit,
        )?;
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
        session.admit_home_owner(
            self.session_home_owner()?,
            WorthQueryProviderSessionProtocolStage::Abort,
        )?;
        self.application_attempt_work.observe_attempt_abort();
        let retired = self
            .attempts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .abort(*session);
        drop(retired);
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

impl WorthQueryPrimaryGraphProvider {
    fn session_home_owner(
        &self,
    ) -> Result<
        Option<worth_runtime_world::facade::RuntimeWorldOwnerIdentity>,
        WorthQueryProviderSessionFailure,
    > {
        self.world_history
            .get()
            .map(|world| {
                world.owner_identity().map_err(|_| {
                    super::session_commit::provider_failure(
                        WorthQueryProviderSessionProtocolStage::PlanReadmission,
                        "session World owner is unavailable",
                    )
                })
            })
            .transpose()
    }
}
