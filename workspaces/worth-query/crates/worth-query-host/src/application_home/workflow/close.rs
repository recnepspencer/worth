//! The original Home close owner retains the complete installed configuration.
use super::*;
use worth_execution::ExecutionRequest;

/// Closed Home resources and workflow configuration retire only at explicit reclaim.
pub struct WorthQueryWorkflowHomeClosed<Schema, Spec>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    configuration: WorthQueryWorkflowRuntimeConfiguration<Schema, Spec>,
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
    configuration: WorthQueryWorkflowRuntimeConfiguration<Schema, Spec>,
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
