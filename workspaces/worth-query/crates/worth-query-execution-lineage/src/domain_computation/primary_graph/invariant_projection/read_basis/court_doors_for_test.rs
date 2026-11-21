//! Sealed test doors over this module's private members.
use super::*;

impl NativeProjectionRead<'_> {
    #[doc(hidden)]
    pub fn committed_runtime_for_test(&self) -> Option<&RelationalRuntime> {
        Self::committed_runtime(self)
    }
}

impl NativeProjectionRead<'_> {
    #[doc(hidden)]
    pub fn origin_for_test<'a>(
        &self,
        snapshot: &'a SnapshotHandle,
    ) -> super::super::super::projection_read_origin::ProjectionReadOriginRef<'a> {
        Self::origin(self, snapshot)
    }
}
