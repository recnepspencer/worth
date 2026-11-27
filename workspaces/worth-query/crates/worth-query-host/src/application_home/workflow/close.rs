//! The original Home close owner retains the complete installed configuration.
use super::*;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::ExecutionRequestPolicy;

/// Closed Home resources and workflow configuration retire only at explicit reclaim.
pub struct WorthQueryWorkflowHomeClosed<Schema, Spec>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    configuration: Arc<WorthQueryWorkflowRuntimeConfiguration<Schema, Spec>>,
    home: WorthQueryApplicationHomeClosed,
}
impl<Schema, Spec> WorthQueryWorkflowHomeClosed<Schema, Spec>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    pub fn is_closed(&self) -> bool {
        self.home.is_closed()
    }
    pub fn outcome(&self) -> &worth_store_application::ApplicationHomeCloseOutcome {
        self.home.outcome()
    }
    pub fn reclaim(self) {
        let Self {
            configuration,
            home,
        } = self;
        drop(configuration);
        home.reclaim();
    }
}
impl<Schema, Spec> std::fmt::Debug for WorthQueryWorkflowHomeClosed<Schema, Spec>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryWorkflowHomeClosed")
            .field("closed", &self.home.is_closed())
            .finish_non_exhaustive()
    }
}

#[must_use = "retain the original close posture and workflow configuration"]
pub struct WorthQueryWorkflowHomeCloseRefused<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    configuration: Arc<WorthQueryWorkflowRuntimeConfiguration<Schema, Spec>>,
    home: WorthQueryApplicationHomeCloseRefused<Schema, Program>,
}
impl<Schema, Spec, Program> WorthQueryWorkflowHomeCloseRefused<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    pub fn reason(&self) -> &WorthQueryApplicationHomeCloseRefused<Schema, Program> {
        &self.home
    }
    /// Resume only when the original close owner permits service again.
    pub fn resume(self) -> Result<WorthQueryWorkflowApplicationHome<Schema, Spec, Program>, Self> {
        let Self {
            configuration,
            home,
        } = self;
        match home.resume() {
            Ok(home) => Ok(WorthQueryWorkflowApplicationHome {
                configuration,
                home,
            }),
            Err(home) => Err(Self {
                configuration,
                home,
            }),
        }
    }
}
impl<Schema, Spec, Program> std::fmt::Debug
    for WorthQueryWorkflowHomeCloseRefused<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryWorkflowHomeCloseRefused")
            .field("home", &self.home)
            .finish_non_exhaustive()
    }
}

impl<Schema, Spec, Program: 'static> WorthQueryWorkflowApplicationHome<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    pub fn close(
        self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        WorthQueryWorkflowHomeClosed<Schema, Spec>,
        WorthQueryWorkflowHomeCloseRefused<Schema, Spec, Program>,
    > {
        let Self {
            configuration,
            home,
        } = self;
        match home.close(request) {
            Ok(home) => Ok(WorthQueryWorkflowHomeClosed {
                configuration,
                home,
            }),
            Err(home) => Err(WorthQueryWorkflowHomeCloseRefused {
                configuration,
                home,
            }),
        }
    }
}
impl<Schema, Spec, Program: 'static> WorthQueryWorkflowHomeCloseRefused<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    pub fn try_close(
        self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryWorkflowHomeClosed<Schema, Spec>, Self> {
        let Self {
            configuration,
            home,
        } = self;
        match home.try_close(request) {
            Ok(home) => Ok(WorthQueryWorkflowHomeClosed {
                configuration,
                home,
            }),
            Err(home) => Err(Self {
                configuration,
                home,
            }),
        }
    }
}

/// Desktop-default retirement: [`WorthQueryApplicationHome::close_clean`] parity.
impl<Schema, Spec, Program: 'static> WorthQueryWorkflowApplicationHome<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    /// Closes this Home using an owned serial lease with the requested budgets.
    pub fn close_with_policy(
        self,
        policy: ExecutionRequestPolicy,
    ) -> Result<
        WorthQueryWorkflowHomeClosed<Schema, Spec>,
        super::super::WorthQueryHomeClosePolicyRefused<
            Self,
            WorthQueryWorkflowHomeCloseRefused<Schema, Spec, Program>,
        >,
    > {
        super::super::close_policy::with_close_policy(self, policy, |home, request| {
            home.close(request)
        })
    }

    /// Cleanly retires this Home under
    /// [`DEFAULT_HOST_EXECUTION_POLICY`](super::super::DEFAULT_HOST_EXECUTION_POLICY),
    /// then retires the workflow configuration and reclaims the inert retired
    /// Query resources. A refusal returns this Home or its close continuation.
    pub fn close_clean(
        self,
    ) -> Result<
        worth_store_application::ApplicationHomeCloseOutcome,
        super::super::WorthQueryHomeClosePolicyRefused<
            Self,
            WorthQueryWorkflowHomeCloseRefused<Schema, Spec, Program>,
        >,
    > {
        self.close_clean_with_policy(super::super::DEFAULT_HOST_EXECUTION_POLICY)
    }

    /// [`Self::close_clean`] under an explicit execution policy.
    pub fn close_clean_with_policy(
        self,
        policy: ExecutionRequestPolicy,
    ) -> Result<
        worth_store_application::ApplicationHomeCloseOutcome,
        super::super::WorthQueryHomeClosePolicyRefused<
            Self,
            WorthQueryWorkflowHomeCloseRefused<Schema, Spec, Program>,
        >,
    > {
        let WorthQueryWorkflowHomeClosed {
            configuration,
            home,
        } = self.close_with_policy(policy)?;
        drop(configuration);
        let (outcome, retired_query) = home.into_parts();
        if let Some(retired_query) = retired_query {
            retired_query.reclaim();
        }
        Ok(outcome)
    }
}
