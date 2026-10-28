use super::{
    WorthQueryBridgeBackedRuntimeBackend, WorthQueryRuntimeBackend, WorthQueryRuntimeBuilder,
    WorthQueryRuntimeError,
};

impl WorthQueryRuntimeBuilder {
    pub(super) fn lower_queued_invariant_registrations_into_backend_parts(
        &mut self,
    ) -> Result<(), WorthQueryRuntimeError> {
        if self.queued_invariant_registrations.is_empty() {
            return Ok(());
        }
        if self.backend_parts.has_relational_runtime() {
            return Err(WorthQueryRuntimeError::InvariantRegistration {
                stage: "relational_runtime_authority_selection",
                message: "queued Query-owned invariant registrations conflict with an explicitly supplied relational runtime; choose one authority path".to_string(),
            });
        }
        let queued = std::mem::take(&mut self.queued_invariant_registrations);
        self.backend_parts = std::mem::take(&mut self.backend_parts)
            .relational_runtime(queued.lower_into_relational_runtime()?);
        Ok(())
    }

    pub(super) fn lower_bridge_backed_backend_from_parts(
        &mut self,
    ) -> Result<Box<dyn WorthQueryRuntimeBackend>, WorthQueryRuntimeError> {
        let bootstrap = std::mem::take(&mut self.backend_parts).lower_bridge_backed_bootstrap()?;
        Ok(Box::new(
            WorthQueryBridgeBackedRuntimeBackend::from_validated_bootstrap(bootstrap),
        ))
    }
}
