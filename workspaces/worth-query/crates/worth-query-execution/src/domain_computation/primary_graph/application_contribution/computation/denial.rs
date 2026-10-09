//! Why a managed computation, or one checkpoint of it, refused to go on.

/// A limit the computation met. Each variant is one cause: the computation's
/// own declarations, the request's execution policy, or the process
/// authority it leases from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedComputationResourceDenial {
    /// The computation's declared work, or the policy's work ceiling, is
    /// spent.
    WorkExhausted,
    /// A work count did not fit its counter.
    WorkCounterOverflow,
    /// What the computation retains would pass its declared bytes.
    RetainedBytesExhausted,
    /// Temporary kernel scratch passed its declared ceiling.
    ScratchCapacityExceeded,
    /// A result passed the bytes its pattern declared for one result.
    ResultCapacityExceeded,
    /// A declared byte count does not fit a byte count.
    CapacityOverflow,
    /// The authority could not add a charged reservation to its byte counter.
    ChargedBytesOverflow,
    /// A memory limit refused a reservation. `level` names the limit, the
    /// innermost one that refused, and `admitted` is the room it left.
    MemoryLimit {
        requested: u64,
        admitted: u64,
        level: WorthQueryMemoryLimitLevel,
    },
    /// The policy asks for more workers than the authority admits.
    WorkerLimit,
    /// The policy asks for more memory than the authority admits.
    PolicyMemoryLimit,
    /// The policy's work ceiling passes the one it runs under.
    WorkLimit,
    /// A pattern ran under a lease that is not its run's.
    NestedLeaseMisuse,
    /// Query refused a second public advancement on the same thread.
    NestedAdvancementOpening,
    /// Query refused custody lent by a different installed runtime.
    ForeignAdvancementPhase,
    /// Bytes were reserved where no request's execution was running.
    NoActiveExecutionScope,
    /// The policy names an equivalence contract the authority does not hold.
    EquivalenceContractUnavailable,
}

/// Which limit refused a memory reservation. Limits are checked innermost
/// first, so a reservation over its request's policy is refused the same way
/// whatever other requests hold; only the process's refusal can clear when
/// they release.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryMemoryLimitLevel {
    /// The request's policy. Every run of a request runs under that one
    /// policy, leased or serial, so which lease of the request refused is
    /// not a cause.
    Policy,
    /// The process's memory, which every request shares.
    Process,
    /// The bytes the pattern itself declared for what it holds.
    Declared,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedComputationInterruption {
    Cancelled,
    DeadlineExceeded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedComputationCheckpointDenial {
    Resource(WorthQueryManagedComputationResourceDenial),
    Interrupted(WorthQueryManagedComputationInterruption),
    /// A pattern the computation ran stopped; its run reports why.
    NestedPatternStopped,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorthQueryManagedComputationDenial<Stopped> {
    Owner(Stopped),
    Resource(WorthQueryManagedComputationResourceDenial),
    Interrupted(WorthQueryManagedComputationInterruption),
    /// A pattern the computation ran stopped; its run reports why.
    NestedPatternStopped,
}

impl<Stopped> From<WorthQueryManagedComputationCheckpointDenial>
    for WorthQueryManagedComputationDenial<Stopped>
{
    fn from(denial: WorthQueryManagedComputationCheckpointDenial) -> Self {
        match denial {
            WorthQueryManagedComputationCheckpointDenial::Resource(denial) => {
                Self::Resource(denial)
            }
            WorthQueryManagedComputationCheckpointDenial::Interrupted(interruption) => {
                Self::Interrupted(interruption)
            }
            WorthQueryManagedComputationCheckpointDenial::NestedPatternStopped => {
                Self::NestedPatternStopped
            }
        }
    }
}
