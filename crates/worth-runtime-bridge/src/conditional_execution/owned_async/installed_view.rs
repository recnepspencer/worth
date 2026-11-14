//! Installed declaration lookup against the actual receiver, never a diagnostic reconstruction.
use super::super::BridgeConditionalExecutionView;
use crate::facade::LoweredBridgeAsyncSourceDeclaration;
use std::sync::PoisonError;
impl BridgeConditionalExecutionView<'_> {
    pub fn require_process_local_owned_async_declaration(
        &self,
        declaration: &LoweredBridgeAsyncSourceDeclaration,
    ) -> Result<(), crate::facade::BridgeAsyncRequestIdentityRejection> {
        if self.owned_async_uses_process_local_retention(declaration) {
            Ok(())
        } else {
            Err(super::admission::request_denial(
                "durable resource handoff requires an installed process-local declaration",
            ))
        }
    }

    pub fn require_process_local_owned_async_owner(
        &self,
        owner: &super::BridgeOwnedAsyncManagedRequestAdmission,
    ) -> Result<(), super::BridgeOwnedAsyncManagedDenial> {
        owner.require_owner(self.async_observation_authority)?;
        if owner.native.uses_process_local_retention() {
            Ok(())
        } else {
            Err(super::BridgeOwnedAsyncManagedDenial::ProgressDenied)
        }
    }

    pub fn installed_owned_async_declaration(
        &self,
        identity: &str,
    ) -> Option<LoweredBridgeAsyncSourceDeclaration> {
        self.async_declarations
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(identity)
            .map(|installed| installed.lowered.clone())
    }

    pub(super) fn owned_async_queue_capacity(
        &self,
        lowered: &LoweredBridgeAsyncSourceDeclaration,
    ) -> Option<std::num::NonZeroU64> {
        self.async_declarations
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(lowered.declaration_identity().as_str())
            .filter(|installed| installed.lowered.digest() == lowered.digest())
            .and_then(|installed| installed.source.description.managed_queue_capacity)
            .or_else(|| {
                self.async_definitions?
                    .get(lowered.declaration_identity().as_str())?
                    .description
                    .managed_queue_capacity
            })
    }

    pub fn owned_async_declaration_count(&self) -> usize {
        self.async_declarations
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .len()
            .max(self.async_definitions.map_or(0, |rows| rows.len()))
    }

    pub(super) fn owned_async_uses_process_local_retention(
        &self,
        lowered: &LoweredBridgeAsyncSourceDeclaration,
    ) -> bool {
        let catalog = self
            .async_declarations
            .read()
            .unwrap_or_else(PoisonError::into_inner);
        if let Some(row) = catalog.get(lowered.declaration_identity().as_str()) {
            return row.lowered.digest() == lowered.digest()
                && row.source.description.retention
                    == super::BridgeOwnedAsyncRetention::ProcessLocal;
        }
        self.async_definitions
            .and_then(|rows| rows.get(lowered.declaration_identity().as_str()))
            .is_some_and(|row| {
                row.description.retention == super::BridgeOwnedAsyncRetention::ProcessLocal
                    && row.description.managed_queue_capacity.is_some()
            })
    }
}
