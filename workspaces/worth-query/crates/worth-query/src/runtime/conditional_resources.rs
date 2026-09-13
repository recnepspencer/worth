use worth_signal::facade::runtime::SignalConditionalEvaluationBudget;

/// Query-owned limits for retained conditional evaluation lookup state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorthQueryConditionalEvaluationCacheBudget {
    maximum_retained_entries: usize,
    maximum_retained_bytes: u64,
}

impl WorthQueryConditionalEvaluationCacheBudget {
    pub fn bounded(
        maximum_retained_entries: usize,
        maximum_retained_bytes: u64,
    ) -> Result<Self, WorthQueryConditionalEvaluationCacheBudgetDenial> {
        if maximum_retained_entries == 0 {
            return Err(WorthQueryConditionalEvaluationCacheBudgetDenial::EmptyEntryCapacity);
        }
        if maximum_retained_bytes == 0 {
            return Err(WorthQueryConditionalEvaluationCacheBudgetDenial::EmptyByteCapacity);
        }
        Ok(Self {
            maximum_retained_entries,
            maximum_retained_bytes,
        })
    }

    pub const fn maximum_retained_entries(self) -> usize {
        self.maximum_retained_entries
    }

    pub const fn maximum_retained_bytes(self) -> u64 {
        self.maximum_retained_bytes
    }

    pub(crate) fn require_bytes(
        self,
        required: u64,
    ) -> Result<(), WorthQueryConditionalEvaluationCacheBudgetDenial> {
        if required > self.maximum_retained_bytes {
            return Err(
                WorthQueryConditionalEvaluationCacheBudgetDenial::InsufficientRetainedBytes {
                    required,
                    available: self.maximum_retained_bytes,
                },
            );
        }
        Ok(())
    }
}

/// Required conditional resource inputs for one installed Query runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorthQueryConditionalExecutionResources {
    query_cache: WorthQueryConditionalEvaluationCacheBudget,
    signal_evaluations: SignalConditionalEvaluationBudget,
}

impl WorthQueryConditionalExecutionResources {
    pub const fn new(
        query_cache: WorthQueryConditionalEvaluationCacheBudget,
        signal_evaluations: SignalConditionalEvaluationBudget,
    ) -> Self {
        Self {
            query_cache,
            signal_evaluations,
        }
    }

    pub fn development() -> Self {
        Self {
            query_cache: WorthQueryConditionalEvaluationCacheBudget {
                maximum_retained_entries: 128,
                maximum_retained_bytes: 8 * 1024 * 1024,
            },
            signal_evaluations: SignalConditionalEvaluationBudget::development(),
        }
    }

    pub const fn query_cache(self) -> WorthQueryConditionalEvaluationCacheBudget {
        self.query_cache
    }

    pub const fn signal_evaluations(self) -> SignalConditionalEvaluationBudget {
        self.signal_evaluations
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorthQueryConditionalEvaluationCacheBudgetDenial {
    EmptyEntryCapacity,
    EmptyByteCapacity,
    InsufficientRetainedBytes { required: u64, available: u64 },
    RetainedByteChargeOverflow,
    RetainedAllocationUnavailable { requested_entries: usize },
}

impl std::fmt::Display for WorthQueryConditionalEvaluationCacheBudgetDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyEntryCapacity => {
                formatter.write_str("conditional evaluation cache requires at least one entry")
            }
            Self::EmptyByteCapacity => formatter
                .write_str("conditional evaluation cache requires a positive retained-byte limit"),
            Self::InsufficientRetainedBytes {
                required,
                available,
            } => write!(
                formatter,
                "conditional evaluation cache requires {required} retained bytes, but {available} were configured"
            ),
            Self::RetainedByteChargeOverflow => formatter.write_str(
                "conditional evaluation cache retained-byte charge exceeds the supported range",
            ),
            Self::RetainedAllocationUnavailable { requested_entries } => write!(
                formatter,
                "conditional evaluation cache could not allocate backing for {requested_entries} entries"
            ),
        }
    }
}

impl std::error::Error for WorthQueryConditionalEvaluationCacheBudgetDenial {}

/// Non-authorizing snapshot of the two owners' conditional retention state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorthQueryConditionalEvaluationResourceObservation {
    pub(crate) query_retained_entries: usize,
    pub(crate) query_retained_bytes: u64,
    pub(crate) query_cache_hits: u64,
    pub(crate) query_cache_misses: u64,
    pub(crate) query_cache_evictions: u64,
    pub(crate) signal_retained_slots: usize,
    pub(crate) signal_retained_bytes: u64,
}

impl WorthQueryConditionalEvaluationResourceObservation {
    pub const fn query_retained_entries(self) -> usize {
        self.query_retained_entries
    }

    pub const fn query_retained_bytes(self) -> u64 {
        self.query_retained_bytes
    }

    pub const fn query_cache_hits(self) -> u64 {
        self.query_cache_hits
    }

    pub const fn query_cache_misses(self) -> u64 {
        self.query_cache_misses
    }

    pub const fn query_cache_evictions(self) -> u64 {
        self.query_cache_evictions
    }

    pub const fn signal_retained_slots(self) -> usize {
        self.signal_retained_slots
    }

    pub const fn signal_retained_bytes(self) -> u64 {
        self.signal_retained_bytes
    }
}
