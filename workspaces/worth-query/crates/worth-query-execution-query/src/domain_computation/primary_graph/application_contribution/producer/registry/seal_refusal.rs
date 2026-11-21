//! A refused batch retains its completed prefix and exact unconsumed executors.
use super::*;
pub struct RefusedProducerSeal<Schema> {
    _pending: PendingProducerRegistry<Schema>,
    _installed: BTreeMap<String, Arc<InstalledProducerProvider<Schema>>>,
    cause: WorthQueryPrimaryGraphInstallationDenial,
}
impl<Schema> RefusedProducerSeal<Schema> {
    pub fn new(
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
    pub fn reason(
        &self,
    ) -> &WorthQueryPrimaryGraphInstallationDenial {
        &self.cause
    }
    pub fn into_cause(
        self,
    ) -> WorthQueryPrimaryGraphInstallationDenial {
        self.cause
    }
}
