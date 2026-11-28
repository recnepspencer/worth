//! The runtime protects actual terminal sources before custody demotes receipt observations.
use super::*;
use worth_query_execution_query::facade::ports::ApplicationRuntimeCoreInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductRuntimeInternalPort as _;
impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(crate) fn preflight_inbound_home_retirement(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), WorthQueryHomeRetirementDenial> {
        use WorthQueryHomeRetirementDenial as Denial;
        use WorthQueryHomeRetirementOwner as Owner;
        let mut custody = self
            .inner
            .runtime
            .inbound_custody_ref()
            .try_lock()
            .map_err(|_| Denial::Busy(Owner::PublicationReceipts))?;
        let history_custody = self
            .inner
            .runtime
            .product_runtime_ref()
            .owner_ref()
            .history_custody_port();
        for terminal in custody.terminal_sources_for_home_retirement() {
            request.consult().map_err(Denial::Execution)?;
            if !terminal.settle_fresh_delivery() {
                return Err(Denial::Busy(Owner::PublicationReceipts));
            }
            let protection = history_custody
                .protect_performed_publication(
                    terminal.publication().publication().commit().identity(),
                )
                .map_err(|_| Denial::Busy(Owner::PublicationReceipts))?;
            self.inner
                .runtime
                .primary_provider_ref()
                .retain_canonical_inbound_terminal(terminal, protection)
                .map_err(|_| Denial::Busy(Owner::PublicationReceipts))?;
        }
        custody
            .preflight_home_retirement(request)
            .map_err(Into::into)
    }
}
