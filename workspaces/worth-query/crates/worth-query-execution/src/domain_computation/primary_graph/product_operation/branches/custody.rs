//! How a head, candidate or document reaches the runtime that owns its branch.
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use std::sync::Arc;

/// An installed allocation that keeps its runtime alive for the owners sharing it.
pub(in crate::domain_computation::primary_graph) trait SharedPrimaryRuntime<Schema>:
    Send + Sync
{
    fn runtime(&self) -> &WorthQueryPrimaryGraphApplicationRuntime<Schema>;
}

/// The runtime behind a branch owner. A borrowed owner cannot outlive its
/// caller's runtime; a shared owner keeps the installed allocation alive itself.
pub(in crate::domain_computation::primary_graph) enum WorthQueryRuntimeCustody<'runtime, Schema> {
    Borrowed(&'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>),
    Shared(Arc<dyn SharedPrimaryRuntime<Schema>>),
}

impl<Schema> WorthQueryRuntimeCustody<'_, Schema> {
    /// Whether this owner belongs to exactly that runtime allocation.
    pub(in crate::domain_computation::primary_graph) fn same_runtime(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    ) -> bool {
        std::ptr::eq::<WorthQueryPrimaryGraphApplicationRuntime<Schema>>(&**self, runtime)
    }
}

impl<Schema> Clone for WorthQueryRuntimeCustody<'_, Schema> {
    fn clone(&self) -> Self {
        match self {
            Self::Borrowed(runtime) => Self::Borrowed(runtime),
            Self::Shared(backing) => Self::Shared(Arc::clone(backing)),
        }
    }
}

impl<Schema> std::ops::Deref for WorthQueryRuntimeCustody<'_, Schema> {
    type Target = WorthQueryPrimaryGraphApplicationRuntime<Schema>;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(runtime) => runtime,
            Self::Shared(backing) => backing.runtime(),
        }
    }
}
