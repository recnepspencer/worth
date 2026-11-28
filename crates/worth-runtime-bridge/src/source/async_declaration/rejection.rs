use std::sync::Arc;

use super::BridgeAsyncSourceDeclarationCounters;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeAsyncSourceDeclarationRejectionKind {
    SignalDeclarationRejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeAsyncSourceDeclarationRejection {
    signal_cause: Option<Box<worth_signal::facade::SignalError>>,
    kind: BridgeAsyncSourceDeclarationRejectionKind,
    detail: Arc<str>,
    counters: BridgeAsyncSourceDeclarationCounters,
}

impl BridgeAsyncSourceDeclarationRejection {
    pub fn new(
        kind: BridgeAsyncSourceDeclarationRejectionKind,
        detail: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            signal_cause: None,
            kind,
            detail: detail.into(),
            counters: BridgeAsyncSourceDeclarationCounters::rejected(),
        }
    }

    pub(super) fn with_signal_cause(mut self, cause: worth_signal::facade::SignalError) -> Self {
        self.signal_cause = Some(Box::new(cause));
        self
    }

    pub fn kind(&self) -> BridgeAsyncSourceDeclarationRejectionKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        self.detail.as_ref()
    }

    pub fn counters(&self) -> &BridgeAsyncSourceDeclarationCounters {
        &self.counters
    }
}

mod report;
