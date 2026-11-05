//! The runtime composes `application_aftermath`.
use worth_query_installation::facade::ApplicationSchema;

use super::super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::application_aftermath::host::ApplicationAftermathHost;

impl<Schema: ApplicationSchema> ApplicationAftermathHost<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn application_aftermath_host(&self) -> &dyn ApplicationAftermathHost<Schema> {
        self
    }
    fn recovery_handles(
        &self,
    ) -> &std::sync::Arc<crate::domain_computation::managed_run::WorthQueryRecoveryHandleRegistry>
    {
        &self.runtime.recovery_handles
    }
}
