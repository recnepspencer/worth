//! Shared lifetime of a Bridge-owned, thread-affine Signal runtime.
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BridgeSignalRuntimeCustody(Arc<RuntimeLifetime>);

#[derive(Debug, PartialEq, Eq)]
struct RuntimeLifetime(u64);

impl BridgeSignalRuntimeCustody {
    pub(crate) fn new(key: u64) -> Self {
        Self(Arc::new(RuntimeLifetime(key)))
    }

    pub(crate) fn key(&self) -> u64 {
        self.0 .0
    }
}

impl Drop for RuntimeLifetime {
    fn drop(&mut self) {
        super::state::release_runtime(self.0);
    }
}
