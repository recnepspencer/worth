use crate::facade::BridgeCommittedPatchEnvelope;

use super::super::fixtures::{
    InMemoryRelationalBridgeSource, RecordingSignalBridgeSink, RecordingTruthWritebackAuthority,
    SnapshotFixture,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeHarnessMutation {
    PublishCommittedPatch(BridgeCommittedPatchEnvelope),
    PublishSnapshot(SnapshotFixture),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum SourceAdapterShape {
    #[default]
    Direct,
    Wrapped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum SourceBuilderLoadOrder {
    #[default]
    AdapterBeforeSources,
    SourcesBeforeAdapter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum PolicyBuilderLoadOrder {
    #[default]
    WholePolicy,
    SectionsCanonical,
    SectionsReverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum SourceAdapterBehavior {
    #[default]
    Honest,
    RejectOpenSnapshot,
    DriftSnapshotIdentity,
}

#[derive(Debug, Clone)]
pub struct BridgeHarnessSession {
    pub(crate) runtime: Option<crate::facade::RuntimeBridge>,
    pub(crate) source: InMemoryRelationalBridgeSource,
    pub(crate) sink: RecordingSignalBridgeSink,
    pub(crate) writeback_authority: RecordingTruthWritebackAuthority,
    pub(crate) source_adapter_shape: SourceAdapterShape,
    pub(crate) source_builder_load_order: SourceBuilderLoadOrder,
    pub(crate) policy_builder_load_order: PolicyBuilderLoadOrder,
    pub(crate) source_adapter_behavior: SourceAdapterBehavior,
}

impl Default for BridgeHarnessSession {
    fn default() -> Self {
        Self {
            runtime: None,
            source: InMemoryRelationalBridgeSource::default(),
            sink: RecordingSignalBridgeSink::default(),
            writeback_authority: RecordingTruthWritebackAuthority::default(),
            source_adapter_shape: SourceAdapterShape::Direct,
            source_builder_load_order: SourceBuilderLoadOrder::AdapterBeforeSources,
            policy_builder_load_order: PolicyBuilderLoadOrder::WholePolicy,
            source_adapter_behavior: SourceAdapterBehavior::Honest,
        }
    }
}

#[derive(Debug)]
pub struct BridgeHarnessError {
    message: String,
    preview: Option<
        Box<(
            crate::facade::BridgeSpeculativeSessionHandle,
            crate::facade::BridgeSpeculationError,
        )>,
    >,
}

impl BridgeHarnessError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            preview: None,
        }
    }

    pub(crate) fn preview_refused(
        preview: crate::facade::BridgeSpeculativeSessionHandle,
        denial: crate::facade::BridgeSpeculationError,
    ) -> Self {
        Self {
            message: format!("speculation transition refused: {denial}"),
            preview: Some(Box::new((preview, denial))),
        }
    }

    /// Recover the actual active preview after a certification transition refuses.
    pub fn into_preview(
        self,
    ) -> Result<
        (
            crate::facade::BridgeSpeculativeSessionHandle,
            crate::facade::BridgeSpeculationError,
        ),
        Self,
    > {
        match self.preview {
            Some(preview) => Ok(*preview),
            None => Err(self),
        }
    }
}

impl std::fmt::Display for BridgeHarnessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for BridgeHarnessError {}

#[derive(Debug, Clone, Copy, Default)]
pub struct BridgeHarnessAdapter;
