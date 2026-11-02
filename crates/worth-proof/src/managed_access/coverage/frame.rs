//! Correspondence of actual borrowed invocation material, with no invented image tag.
use std::{any::Any, sync::Arc};

pub struct BorrowedManagedCoverageFrame<'a, Audience: ?Sized> {
    root: &'a Arc<dyn Any + Send + Sync>,
    source: &'a dyn Any,
    probe: &'a dyn Any,
    audience: &'a Audience,
}
impl<Audience: ?Sized> Copy for BorrowedManagedCoverageFrame<'_, Audience> {}
impl<Audience: ?Sized> Clone for BorrowedManagedCoverageFrame<'_, Audience> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'a, Audience: ?Sized> BorrowedManagedCoverageFrame<'a, Audience> {
    /// Descriptive borrowed input only. No constructor here issues coverage.
    pub fn new(
        root: &'a Arc<dyn Any + Send + Sync>,
        source: &'a dyn Any,
        probe: &'a dyn Any,
        audience: &'a Audience,
    ) -> Self {
        Self {
            root,
            source,
            probe,
            audience,
        }
    }
    pub fn same_invocation(&self, expected: &Self) -> bool {
        Arc::ptr_eq(self.root, expected.root)
            && std::ptr::eq(self.source, expected.source)
            && std::ptr::eq(self.probe, expected.probe)
            && std::ptr::eq(self.audience, expected.audience)
    }
}
