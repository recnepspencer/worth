//! A refused batch retains its completed prefix and exact unconsumed executors.
use super::*;
pub(in crate::domain_computation::primary_graph) struct RefusedProducerSeal<Schema> {
    _pending: PendingProducerRegistry<Schema>,
    _installed: BTreeMap<String, Arc<InstalledProducerProvider<Schema>>>,
    cause: WorthQueryPrimaryGraphInstallationDenial,
}
impl<Schema> RefusedProducerSeal<Schema> {
    pub(super) fn new(
        _pending: PendingProducerRegistry<Schema>,
        _installed: BTreeMap<String, Arc<InstalledProducerProvider<Schema>>>,
        cause: WorthQueryPrimaryGraphInstallationDenial,
    ) -> Self {
        Self {
            _pending,
            _installed,
            cause,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn reason(
        &self,
    ) -> &WorthQueryPrimaryGraphInstallationDenial {
        &self.cause
    }
    pub(in crate::domain_computation::primary_graph) fn into_cause(
        self,
    ) -> WorthQueryPrimaryGraphInstallationDenial {
        self.cause
    }
}
