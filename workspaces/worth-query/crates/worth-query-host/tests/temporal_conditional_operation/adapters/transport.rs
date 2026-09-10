#![allow(dead_code)] // This fixture is compiled by several independent certification targets.

use std::sync::atomic::{AtomicUsize, Ordering};

use worth_query_host::facade::primary_graph::{
    WorthQueryExternalDispatchRequest, WorthQueryExternalEffectTransport,
    WorthQueryExternalTransportOutcome,
};

#[derive(Default)]
pub struct CompletingExternalTransport {
    contacts: AtomicUsize,
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
