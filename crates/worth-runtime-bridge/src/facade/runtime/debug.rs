use super::*;

impl RuntimeBridge {
    /// Returns the admitted source registry bound into this runtime.
    ///
    /// This is a debug/specialist inspection surface, not part of the everyday
    /// bridge memory model.
    pub fn source_registry(&self) -> &AdmittedSourceRegistry {
        &self.source_registry
    }

    /// Returns the configured source adapter, if one is bound.
    ///
    /// This exists for host-adapter and specialist inspection workflows.
    pub fn source_adapter(&self) -> Option<&Arc<dyn BridgeSourceAdapter>> {
        self.source_adapter.as_ref()
    }

    /// Returns the configured writeback authority, if one is bound.
    ///
    /// This is intentionally secondary to the standard path and writeback
    /// facade flows.
    pub fn writeback_authority(&self) -> Option<&Arc<dyn TruthWritebackAuthority>> {
        self.writeback_authority.as_ref()
    }
}
