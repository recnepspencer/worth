use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorthQuerySemanticScaleAxis {
    ModelSize,
    SourceRows,
    SourceEdges,
    TouchedRegion,
    GraphValence,
    VisitedState,
    FrontierState,
    CandidateItems,
    WorkItems,
    OutputRows,
    OutputWidth,
    BatchWidth,
}

impl WorthQuerySemanticScaleAxis {
    pub const ALL: [Self; 12] = [
        Self::ModelSize,
        Self::SourceRows,
        Self::SourceEdges,
        Self::TouchedRegion,
        Self::GraphValence,
        Self::VisitedState,
        Self::FrontierState,
        Self::CandidateItems,
        Self::WorkItems,
        Self::OutputRows,
        Self::OutputWidth,
        Self::BatchWidth,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ModelSize => "model-size",
            Self::SourceRows => "source-rows",
            Self::SourceEdges => "source-edges",
            Self::TouchedRegion => "touched-region",
            Self::GraphValence => "graph-valence",
            Self::VisitedState => "visited-state",
            Self::FrontierState => "frontier-state",
            Self::CandidateItems => "candidate-items",
            Self::WorkItems => "work-items",
            Self::OutputRows => "output-rows",
            Self::OutputWidth => "output-width",
            Self::BatchWidth => "batch-width",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorthQueryResourceDimension {
    ScratchBytes,
    TransientBytes,
    CandidateRetainedRepresentationBytes,
    PeakResidentBytes,
    RetainedBytes,
    ReclaimableBytes,
    OutputBytes,
    AllocationCount,
    ProviderContacts,
    ProviderMessages,
    ProviderRetries,
    ProviderBarriers,
    SynchronizationBudget,
    QueueDepth,
    ConcurrencyWidth,
    ChunkWidth,
    FanOut,
    DeadlineNanos,
    CancellationPollingInterval,
    CleanupBudget,
}

impl WorthQueryResourceDimension {
    pub const ALL: [Self; 20] = [
        Self::ScratchBytes,
        Self::TransientBytes,
        Self::CandidateRetainedRepresentationBytes,
        Self::PeakResidentBytes,
        Self::RetainedBytes,
        Self::ReclaimableBytes,
        Self::OutputBytes,
        Self::AllocationCount,
        Self::ProviderContacts,
        Self::ProviderMessages,
        Self::ProviderRetries,
        Self::ProviderBarriers,
        Self::SynchronizationBudget,
        Self::QueueDepth,
        Self::ConcurrencyWidth,
        Self::ChunkWidth,
        Self::FanOut,
        Self::DeadlineNanos,
        Self::CancellationPollingInterval,
        Self::CleanupBudget,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ScratchBytes => "scratch-bytes",
            Self::TransientBytes => "transient-bytes",
            Self::CandidateRetainedRepresentationBytes => "candidate-retained-representation-bytes",
            Self::PeakResidentBytes => "peak-resident-bytes",
            Self::RetainedBytes => "retained-bytes",
            Self::ReclaimableBytes => "reclaimable-bytes",
            Self::OutputBytes => "output-bytes",
            Self::AllocationCount => "allocation-count",
            Self::ProviderContacts => "provider-contacts",
            Self::ProviderMessages => "provider-messages",
            Self::ProviderRetries => "provider-retries",
            Self::ProviderBarriers => "provider-barriers",
            Self::SynchronizationBudget => "synchronization-budget",
            Self::QueueDepth => "queue-depth",
            Self::ConcurrencyWidth => "concurrency-width",
            Self::ChunkWidth => "chunk-width",
            Self::FanOut => "fan-out",
            Self::DeadlineNanos => "deadline-nanos",
            Self::CancellationPollingInterval => "cancellation-polling-interval",
            Self::CleanupBudget => "cleanup-budget",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySemanticScaleRequest {
    values: BTreeMap<WorthQuerySemanticScaleAxis, u64>,
}

impl WorthQuerySemanticScaleRequest {
    pub fn bounded(value: u64) -> Self {
        Self {
            values: WorthQuerySemanticScaleAxis::ALL
                .into_iter()
                .map(|axis| (axis, value))
                .collect(),
        }
    }

    pub fn with(mut self, axis: WorthQuerySemanticScaleAxis, value: u64) -> Self {
        self.values.insert(axis, value);
        self
    }

    pub fn get(&self, axis: WorthQuerySemanticScaleAxis) -> Option<u64> {
        self.values.get(&axis).copied()
    }

    pub fn iter(&self) -> impl Iterator<Item = (WorthQuerySemanticScaleAxis, u64)> + '_ {
        self.values.iter().map(|(axis, value)| (*axis, *value))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryResourceLimitRequest {
    values: BTreeMap<WorthQueryResourceDimension, u64>,
}

impl WorthQueryResourceLimitRequest {
    pub fn bounded(value: u64) -> Self {
        Self {
            values: WorthQueryResourceDimension::ALL
                .into_iter()
                .map(|dimension| (dimension, value))
                .collect(),
        }
    }

    pub fn with(mut self, dimension: WorthQueryResourceDimension, value: u64) -> Self {
        self.values.insert(dimension, value);
        self
    }

    pub fn get(&self, dimension: WorthQueryResourceDimension) -> Option<u64> {
        self.values.get(&dimension).copied()
    }

    pub fn iter(&self) -> impl Iterator<Item = (WorthQueryResourceDimension, u64)> + '_ {
        self.values
            .iter()
            .map(|(dimension, value)| (*dimension, *value))
    }
}
