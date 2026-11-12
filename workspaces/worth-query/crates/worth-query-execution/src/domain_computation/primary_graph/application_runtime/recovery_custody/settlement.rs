//! Same original terminal publication precedes release of its mandatory material.
use super::*;
use worth_execution::ExecutionRequest;
impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn settle_recovery_custody(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        if !self.runtime.dispatch_journal.is_durable() {
            return Ok(());
        }
        let admission = self
            .runtime
            .dispatch_journal
            .begin_recovery_admission(execution)
            .map_err(Denial::journal)?;
        for debt in self.runtime.recovery_handles.deferred_dispositions() {
            let current = admission
                .claim(&debt.recorded.claim.original)
                .map_err(Denial::journal)?
                .ok_or_else(|| Denial::new(Kind::AlreadyTerminal))?;
            if current.identity != debt.recorded.identity {
                return Err(Denial::new(Kind::StaleIncarnation));
            }
            let terminal = current.terminal.unwrap_or(debt.terminal);
            admission
                .dispose(
                    &current.claim.original,
                    current.identity,
                    terminal,
                    execution,
                )
                .map_err(Denial::journal)?;
            self.runtime
                .primary_provider
                .release_recovery_material(
                    current.claim.material_pin,
                    current.claim.attempt,
                    execution,
                )
                .map_err(Denial::material)?;
            self.runtime
                .recovery_handles
                .acknowledge_deferred(current.identity);
        }
        let claims = self
            .runtime
            .dispatch_journal
            .recorded_recovery_claims()
            .map_err(Denial::journal)?;
        let pins = self
            .runtime
            .primary_provider
            .settled_recovery_material_pins(execution)
            .map_err(Denial::material)?;
        let actual_owner = self
            .runtime
            .product_runtime
            .owner
            .owner_identity()
            .durable()
            .ok_or_else(|| Denial::new(Kind::ForeignRuntime))?;
        for claim in &claims {
            execution.consult().map_err(|cause| Denial::journal(crate::domain_computation::application_aftermath::dispatch_journal::QueryDispatchJournalDenial::Execution(cause)))?;
            if claim.claim.original.owner() != actual_owner {
                return Err(Denial::new(Kind::ForeignRuntime));
            }
            if claim.terminal.is_none()
                && !pins.iter().any(|(pin, original)| {
                    *pin == claim.claim.material_pin && *original == claim.claim.attempt
                })
            {
                return Err(Denial::material(crate::facade::correction_material::QueryCorrectionMaterialDenial::PinOwnerMismatch));
            }
        }
        for (pin, original) in pins {
            if !claims.iter().any(|claim| {
                claim.terminal.is_none()
                    && claim.claim.material_pin == pin
                    && claim.claim.attempt == original
            }) {
                self.runtime
                    .primary_provider
                    .release_recovery_material(pin, original, execution)
                    .map_err(Denial::material)?;
            }
        }
        Ok(())
    }
}
