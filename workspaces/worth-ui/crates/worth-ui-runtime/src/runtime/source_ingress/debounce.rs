use std::time::Duration;

use crate::runtime::source_ingress::counters::WorthUiSourceIngressCounters;
use crate::runtime::source_ingress::denial::{
    WorthUiSourceIngressDenial, WorthUiSourceIngressDenialReason,
};
use crate::runtime::source_ingress::digest::fold_texts;
use crate::runtime::source_ingress::event::{event_burst_digest, WorthUiWatcherEvent};
use crate::runtime::source_ingress::ordering_receipt::WorthUiCandidateOrderingReceipt;
use crate::runtime::source_ingress::provider::WorthUiSourceProvider;
use crate::runtime::source_ingress::revision::WorthUiSourcePackageRevision;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthUiReloadDebounce {
    stable_window_millis: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiSettledSourceSnapshot {
    provider: WorthUiSourceProvider,
    revision: WorthUiSourcePackageRevision,
    ordering_receipt: WorthUiCandidateOrderingReceipt,
    counters: WorthUiSourceIngressCounters,
}

impl WorthUiReloadDebounce {
    pub fn stable_window(window: Duration) -> Self {
        Self {
            stable_window_millis: u64::try_from(window.as_millis()).unwrap_or(u64::MAX),
        }
    }

    pub(crate) fn policy_digest(&self) -> u64 {
        fold_texts([format!("stable-window-ms:{}", self.stable_window_millis)])
    }

    pub(crate) fn settlement_window(&self) -> Duration {
        Duration::from_millis(self.stable_window_millis)
    }

    pub(crate) fn debounce(
        &self,
        provider: WorthUiSourceProvider,
        events: &[WorthUiWatcherEvent],
        sequence: u64,
    ) -> Result<WorthUiSettledSourceSnapshot, WorthUiSourceIngressDenial> {
        if provider.is_empty() {
            return Err(WorthUiSourceIngressDenial::new(
                WorthUiSourceIngressDenialReason::EmptyProvider,
            ));
        }
        if has_unstable_partial_write(events) {
            return Err(WorthUiSourceIngressDenial::new(
                WorthUiSourceIngressDenialReason::PartialWriteWithoutStableSnapshot,
            ));
        }

        let mut counters = WorthUiSourceIngressCounters::default();
        for _event in events {
            counters.observe_event();
        }
        counters.coalesce_event();
        counters.record_provider_read();
        counters.emit_revision();

        let revision = WorthUiSourcePackageRevision::new(
            provider.id(),
            provider.final_package_digest(),
            event_burst_digest(events),
            sequence,
        );
        let ordering_receipt =
            WorthUiCandidateOrderingReceipt::from_revision(&revision, self.policy_digest());
        Ok(WorthUiSettledSourceSnapshot {
            provider,
            revision,
            ordering_receipt,
            counters,
        })
    }
}

impl Default for WorthUiReloadDebounce {
    fn default() -> Self {
        Self {
            stable_window_millis: 35,
        }
    }
}

impl WorthUiSettledSourceSnapshot {
    pub fn source_revision(&self) -> &WorthUiSourcePackageRevision {
        &self.revision
    }

    pub fn ordering_receipt(&self) -> &WorthUiCandidateOrderingReceipt {
        &self.ordering_receipt
    }

    pub fn counters(&self) -> WorthUiSourceIngressCounters {
        self.counters
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        WorthUiSourceProvider,
        WorthUiSourcePackageRevision,
        WorthUiCandidateOrderingReceipt,
        WorthUiSourceIngressCounters,
    ) {
        (
            self.provider,
            self.revision,
            self.ordering_receipt,
            self.counters,
        )
    }

    #[cfg(test)]
    pub(crate) fn with_ordering_receipt_for_test(
        mut self,
        ordering_receipt: WorthUiCandidateOrderingReceipt,
    ) -> Self {
        self.ordering_receipt = ordering_receipt;
        self
    }
}

fn has_unstable_partial_write(events: &[WorthUiWatcherEvent]) -> bool {
    events
        .iter()
        .any(WorthUiWatcherEvent::is_partial_without_completion)
        && !events
            .iter()
            .any(|event| !event.is_partial_without_completion())
}
