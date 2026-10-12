use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::time::Instant;

use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

/// Query request authority derived from one admitted transport request.
///
/// The identity participates in request-context equality. The live scope is
/// deliberately opaque to equality because its cancellation state may change
/// while the admitted request is executing.
#[derive(Clone)]
pub struct WorthServerQueryRequestScope {
    identity: String,
    scope: WorthQueryRequestScope,
}

impl WorthServerQueryRequestScope {
    pub(crate) fn new(identity: String, scope: WorthQueryRequestScope) -> Self {
        Self { identity, scope }
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub fn request_scope(&self) -> &WorthQueryRequestScope {
        &self.scope
    }
}

impl std::fmt::Debug for WorthServerQueryRequestScope {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthServerQueryRequestScope")
            .field("identity", &self.identity)
            .field("scope", &self.scope)
            .finish()
    }
}

impl PartialEq for WorthServerQueryRequestScope {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
    }
}

impl Eq for WorthServerQueryRequestScope {}

impl PartialOrd for WorthServerQueryRequestScope {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WorthServerQueryRequestScope {
    fn cmp(&self, other: &Self) -> Ordering {
        self.identity.cmp(&other.identity)
    }
}

impl Hash for WorthServerQueryRequestScope {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.identity.hash(state);
    }
}

#[derive(Clone, Default)]
pub(crate) struct WorthServerTransportRequestCancellation {
    source: WorthQueryCancellationSource,
}

impl WorthServerTransportRequestCancellation {
    pub(crate) fn query_scope(
        &self,
        identity: String,
        deadline: Instant,
        initially_cancelled: bool,
    ) -> WorthServerQueryRequestScope {
        if initially_cancelled {
            self.source.cancel();
        }
        WorthServerQueryRequestScope::new(
            identity,
            WorthQueryRequestScope::new(Some(deadline), self.source.token()),
        )
    }

    pub(crate) fn cancel(&self) {
        self.source.cancel();
    }
}

pub(crate) struct WorthServerTransportRequestCancellationGuard {
    cancellation: WorthServerTransportRequestCancellation,
}

impl WorthServerTransportRequestCancellationGuard {
    pub(crate) fn new() -> Self {
        Self {
            cancellation: WorthServerTransportRequestCancellation::default(),
        }
    }

    pub(crate) fn cancellation(&self) -> WorthServerTransportRequestCancellation {
        self.cancellation.clone()
    }
}

impl Drop for WorthServerTransportRequestCancellationGuard {
    fn drop(&mut self) {
        self.cancellation.cancel();
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestInterruption;

    use super::WorthServerTransportRequestCancellationGuard;

    #[test]
    fn transport_lifecycle_cancels_its_carried_query_scope() {
        let guard = WorthServerTransportRequestCancellationGuard::new();
        let scope = guard.cancellation().query_scope(
            "request-1".to_string(),
            Instant::now() + Duration::from_secs(30),
            false,
        );
        assert_eq!(scope.request_scope().interruption(), None);

        drop(guard);

        assert_eq!(
            scope.request_scope().interruption(),
            Some(WorthQueryRequestInterruption::Cancelled)
        );
    }

    #[test]
    fn pre_cancelled_transport_scope_arrives_cancelled() {
        let guard = WorthServerTransportRequestCancellationGuard::new();
        let scope = guard.cancellation().query_scope(
            "request-2".to_string(),
            Instant::now() + Duration::from_secs(30),
            true,
        );

        assert_eq!(
            scope.request_scope().interruption(),
            Some(WorthQueryRequestInterruption::Cancelled)
        );
    }
}
