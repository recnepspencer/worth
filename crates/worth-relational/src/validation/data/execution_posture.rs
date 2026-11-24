use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvariantPlanScopeClass {
    TouchedScope,
    PartitionScope,
    BroaderScope,
}

impl InvariantPlanScopeClass {
    pub const fn diagnostic_label(self) -> &'static str {
        match self {
            Self::TouchedScope => "touched_scope",
            Self::PartitionScope => "partition_scope",
            Self::BroaderScope => "broader_scope",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvariantScopeWideningCause {
    AllObservedPartitionScope,
    FullObservedReadSet,
}

impl InvariantScopeWideningCause {
    pub const fn diagnostic_label(self) -> &'static str {
        match self {
            Self::AllObservedPartitionScope => "all_observed_partition_scope",
            Self::FullObservedReadSet => "full_observed_read_set",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvariantExecutionDisposition {
    Executed,
    SkippedByPlanContract,
    SkippedByMayBreakMask,
}
