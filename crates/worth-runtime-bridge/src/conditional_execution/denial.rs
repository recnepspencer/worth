#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeConditionalDenialKind {
    ConditionalRetentionCapacity,
    ConditionalRetentionClosed,
    ConditionalRetentionQuarantined,
    ForeignSignalGraph,
    CorrespondenceAdmission,
    EmptyCorrespondenceSet,
    MixedSignalNodes,
    DeclarationLocationMismatch,
    DeclarationCorrespondenceMismatch,
    SignalNodeAlreadyBound,
    MissingConditionProvider,
    ExtraConditionProvider,
    MissingDependencyComparator,
    ExtraDependencyComparator,
    MissingOutputComparator,
    ExtraOutputComparator,
    MissingReuseComparator,
    ExtraReuseComparator,
    MissingTriggerProvider,
    ExtraTriggerProvider,
    MissingWakeProvider,
    ExtraWakeProvider,
    MissingComputeProvider,
    ExtraComputeProvider,
    UnsupportedMaintenancePosture,
    UnsupportedArtifactPosture,
    SignalContractInstallation,
    StaleLowering,
    OperationAuthorityMismatch,
    GraphAuthorityMismatch,
    SignalContractMismatch,
    DependencyOrdinalMismatch,
    SnapshotMismatch,
    SnapshotAdmission,
    MissingSourceObservation,
    SourcePostureMismatch,
    AttemptMismatch,
    ManagedWakeMismatch,
    ManagedClockQuarantined,
    SignalExecution,
    ConditionalTransitionChainIncomplete,
    ConditionalTransitionChainMismatch,
    ConditionalPredecessorNotExecuted,
    ConditionalEvaluationBusy,
    ConditionalEvaluationPoisoned,
    ConditionalEvaluationUnwindPending,
    ConditionalEvaluationAdmissionCapacity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeConditionalDenial {
    kind: BridgeConditionalDenialKind,
    detail: String,
    lowering_counters: super::BridgeInstalledConditionalLoweringCounters,
    bridge_execution_counters: super::BridgeConditionalExecutionCounters,
    reentry_counters: super::BridgeConditionalReentryCounters,
    signal_counters: worth_signal::facade::SignalConditionalDecisionCounters,
    semantic_observation_reads: usize,
}

impl BridgeConditionalDenial {
    pub(crate) fn new(kind: BridgeConditionalDenialKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
            lowering_counters: super::BridgeInstalledConditionalLoweringCounters::default(),
            bridge_execution_counters: super::BridgeConditionalExecutionCounters::default(),
            reentry_counters: super::BridgeConditionalReentryCounters::default(),
            signal_counters: worth_signal::facade::SignalConditionalDecisionCounters::default(),
            semantic_observation_reads: 0,
        }
    }
    pub(crate) fn with_lowering_counters(
        mut self,
        counters: super::BridgeInstalledConditionalLoweringCounters,
    ) -> Self {
        self.lowering_counters = counters;
        self
    }
    pub(crate) fn with_execution_counters(
        mut self,
        signal_counters: worth_signal::facade::SignalConditionalDecisionCounters,
        semantic_observation_reads: usize,
    ) -> Self {
        self.signal_counters = signal_counters;
        self.semantic_observation_reads = semantic_observation_reads;
        self
    }
    pub(crate) fn with_bridge_execution_counters(
        mut self,
        counters: super::BridgeConditionalExecutionCounters,
    ) -> Self {
        self.bridge_execution_counters = counters;
        self
    }
    pub(crate) fn with_reentry_counters(
        mut self,
        counters: super::BridgeConditionalReentryCounters,
    ) -> Self {
        self.reentry_counters = counters;
        self
    }
    pub const fn kind(&self) -> BridgeConditionalDenialKind {
        self.kind
    }
    pub fn detail(&self) -> &str {
        &self.detail
    }
    pub const fn lowering_counters(&self) -> super::BridgeInstalledConditionalLoweringCounters {
        self.lowering_counters
    }
    pub const fn bridge_execution_counters(&self) -> super::BridgeConditionalExecutionCounters {
        self.bridge_execution_counters
    }
    pub const fn reentry_counters(&self) -> super::BridgeConditionalReentryCounters {
        self.reentry_counters
    }
    pub const fn signal_counters(&self) -> worth_signal::facade::SignalConditionalDecisionCounters {
        self.signal_counters
    }
    pub const fn semantic_observation_reads(&self) -> usize {
        self.semantic_observation_reads
    }
}
