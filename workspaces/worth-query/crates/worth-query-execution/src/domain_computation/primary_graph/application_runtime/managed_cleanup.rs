//! Ordinary inspection and logical discharge after the live release handoff.
use super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::{query_obligations::*, WorthQueryAdvancementPhase};
use worth_foundational::facade::ManagedCleanupObligationIdentity;
use worth_query_installation::facade::ApplicationSchema;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn managed_cleanup_obligations(&self) -> Vec<QueryManagedCleanupObligation> {
        self.installed_program_support()
            .map(|support| support.lifecycle().managed_cleanup_obligations())
            .unwrap_or_default()
    }
    /// Discharges only the logical obligation. This never changes World's business fate.
    pub fn discharge_managed_cleanup(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        identity: ManagedCleanupObligationIdentity,
    ) -> Result<(), QueryObligationDenial> {
        let request = phase
            .checked_execution_request_for(&self.runtime.product_runtime)
            .map_err(|_| QueryObligationDenial::StaleIncarnation)?;
        self.installed_program_support()
            .ok_or(QueryObligationDenial::SupportUnavailable)?
            .lifecycle()
            .discharge_managed_cleanup(&request, identity)
    }
}
