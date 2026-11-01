//! Refusals preserve the original frame or operation allocation owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalResidencyRetirementDenial {
    Busy,
    PoisonedOwner,
    PoolClosed,
    FrameIdentityMismatch,
    ActiveAllocationUse,
    AccountingMismatch,
    EventSequenceExhausted,
    Execution(worth_execution::WorkCeilingDenial),
}

/// Inert original pool custody; no allocation, frame or read authority escapes.
/// Final destruction is unbudgeted; retain through close, then reclaim outside requests.
#[derive(Debug)]
pub struct RetiredOperationAllocation {
    pub(crate) _owner: std::sync::Arc<super::pool::PoolInner>,
}

/// Inert original pool custody after this pin's byte share was relinquished.
/// Final destruction is unbudgeted; retain through close, then reclaim outside requests.
#[derive(Debug)]
pub struct RetiredPhysicalFrame {
    pub(crate) _owner: std::sync::Arc<super::pool::PoolInner>,
}
