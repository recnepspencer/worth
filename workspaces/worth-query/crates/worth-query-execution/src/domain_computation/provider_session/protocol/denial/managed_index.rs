//! Original managed consistency causes remain owned through commit protocols.
use super::WorthQueryProviderSessionFailure;
use crate::domain_computation::vocabulary::primary_graph::invocation::WorthQueryManagedIndexWorkDenial as Cause;
impl WorthQueryProviderSessionFailure {
    pub(in crate::domain_computation) fn with_managed_index_denial(mut self, cause: Cause) -> Self {
        self.managed_index = Some(cause);
        self
    }
    pub fn managed_index_denial(&self) -> Option<&Cause> {
        self.managed_index.as_ref()
    }
}
