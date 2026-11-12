//! The runtime protects actual terminal sources before custody demotes receipt observations.
use super::*;
impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(super) fn preflight_inbound_home_retirement(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), WorthQueryHomeRetirementDenial> {
        use WorthQueryHomeRetirementDenial as Denial;
        use WorthQueryHomeRetirementOwner as Owner;
        let mut custody = self
            .runtime
            .inbound_custody
            .try_lock()
            .map_err(|_| Denial::Busy(Owner::PublicationReceipts))?;
        let inspection = self.runtime.product_runtime.owner.inspection_port();
        for terminal in custody.terminal_sources_for_home_retirement() {
            request.consult().map_err(Denial::Execution)?;
            if !terminal.settle_fresh_delivery() {
                return Err(Denial::Busy(Owner::PublicationReceipts));
            }
            let protection = inspection
                .protect_performed_publication(
                    terminal.publication().publication().commit().identity(),
                )
                .map_err(|_| Denial::Busy(Owner::PublicationReceipts))?;
            self.runtime
                .primary_provider
                .retain_canonical_inbound_terminal(terminal, protection)
                .map_err(|_| Denial::Busy(Owner::PublicationReceipts))?;
        }
        custody.preflight_home_retirement(request)
    }
}
