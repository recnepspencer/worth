#![allow(dead_code)] // This fixture is compiled by several independent certification targets.

pub use std::sync::atomic::{AtomicUsize, Ordering};

pub use worth_query_host::facade::primary_graph::{
    WorthQueryExternalDispatchRequest, WorthQueryExternalEffectTransport,
    WorthQueryExternalTransportOutcome,
};

#[derive(Default)]
pub struct CompletingExternalTransport {
    pub contacts: AtomicUsize,
}

impl CompletingExternalTransport {
    pub fn contact_count(&self) -> usize {
        self.contacts.load(Ordering::Acquire)
    }
}

impl WorthQueryExternalEffectTransport for CompletingExternalTransport {
    fn dispatch(
        &self,
        _request: WorthQueryExternalDispatchRequest<'_>,
    ) -> WorthQueryExternalTransportOutcome {
        self.contacts.fetch_add(1, Ordering::AcqRel);
        WorthQueryExternalTransportOutcome::Completed
    }
}
